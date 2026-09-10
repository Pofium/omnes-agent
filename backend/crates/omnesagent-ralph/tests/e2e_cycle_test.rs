use async_trait::async_trait;
use omnesagent_config::ralph::RalphConfig;
use omnesagent_ralph::approval::GateManager;
use omnesagent_ralph::debt::DebtLedger;
use omnesagent_ralph::executor::{Executor, ExecutorAnswer, ExecutorError, TaskContext};
use omnesagent_ralph::journal::{ExecutionResult, Journal};
use omnesagent_ralph::knowledge::RalphKnowledgeService;
use omnesagent_ralph::loop_driver::{drive_loop, DriverError};
use omnesagent_ralph::minimality::{DeleteItem, DeleteList};
use omnesagent_ralph::state::{RalphPhase, RalphState, StateError};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

/// Mock executor to simulate iterations without real external LLM APIs.
struct MockCycleExecutor {
    pub repo_root: std::path::PathBuf,
    pub call_count: AtomicUsize,
}

impl MockCycleExecutor {
    fn new(repo_root: std::path::PathBuf) -> Self {
        Self {
            repo_root,
            call_count: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl Executor for MockCycleExecutor {
    async fn run_task(&self, ctx: &TaskContext<'_>) -> Result<ExecutorAnswer, ExecutorError> {
        let count = self.call_count.fetch_add(1, Ordering::SeqCst) + 1;

        if count == 1 {
            Ok(ExecutorAnswer {
                task_id: ctx.task.id.clone(),
                hypothesis: "Attempt 1: Naive approach".to_string(),
                plan: vec!["Write code".to_string()],
                result: ExecutionResult {
                    what_done: "Added broken code".to_string(),
                    errors: vec![],
                    self_assessment: "partial".to_string(),
                },
                ladder_rung: "stdlib".to_string(),
                markers: vec![],
                summary: "Broken attempt 1".to_string(),
                raw_output: String::new(),
            })
        } else if count == 2 {
            Ok(ExecutorAnswer {
                task_id: ctx.task.id.clone(),
                hypothesis: "Attempt 2: Alternative approach".to_string(),
                plan: vec!["Refactor code".to_string()],
                result: ExecutionResult {
                    what_done: "Refactored broken code".to_string(),
                    errors: vec![],
                    self_assessment: "partial".to_string(),
                },
                ladder_rung: "stdlib".to_string(),
                markers: vec![],
                summary: "Broken attempt 2".to_string(),
                raw_output: String::new(),
            })
        } else {
            // Iteration 3: successful implementation with a ponytail marker and boilerplate to delete
            let src_file = ctx.task.paths_scope[0].clone();
            let code = r#"
// unused_boilerplate
pub fn dead_code() {}
// ponytail: memory buffer up to 100, upgrade to sqlite
pub fn working_fn() -> bool { true }
"#;
            // Write code into repo
            let full_path = self.repo_root.join(&src_file);
            let _ = std::fs::write(&full_path, code);

            Ok(ExecutorAnswer {
                task_id: ctx.task.id.clone(),
                hypothesis: "Attempt 3: Correct implementation".to_string(),
                plan: vec!["Apply correct fix".to_string()],
                result: ExecutionResult {
                    what_done: "Working code implemented".to_string(),
                    errors: vec![],
                    self_assessment: "ok".to_string(),
                },
                ladder_rung: "minimum".to_string(),
                markers: vec!["ponytail: memory buffer up to 100, upgrade to sqlite".to_string()],
                summary: "Working fix implemented".to_string(),
                raw_output: String::new(),
            })
        }
    }

    async fn review_delete_list(&self, _diff: &str) -> Result<DeleteList, ExecutorError> {
        Ok(DeleteList {
            items: vec![DeleteItem {
                file: "src/lib.rs".to_string(),
                target_pattern: "// unused_boilerplate\npub fn dead_code() {}\n".to_string(),
                rationale: "Unnecessary boilerplate".to_string(),
            }],
        })
    }
}

fn init_git_repo(path: &Path) {
    let _ = Command::new("git").args(["init"]).current_dir(path).stdin(std::process::Stdio::null()).output();
    let _ = Command::new("git")
        .args(["config", "user.name", "Ralph Test"])
        .current_dir(path)
        .stdin(std::process::Stdio::null())
        .output();
    let _ = Command::new("git")
        .args(["config", "user.email", "ralph@example.com"])
        .current_dir(path)
        .stdin(std::process::Stdio::null())
        .output();

    let src_dir = path.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), "// initial\n").unwrap();

    let _ = Command::new("git").args(["add", "."]).current_dir(path).stdin(std::process::Stdio::null()).output();
    let _ = Command::new("git")
        .args(["commit", "-m", "chore: initial commit"])
        .current_dir(path)
        .stdin(std::process::Stdio::null())
        .output();
}

#[tokio::test]
async fn test_e2e_two_red_one_green_review_archive() {
    let repo = tempdir().unwrap();
    let repo_path = repo.path().to_path_buf();
    init_git_repo(&repo_path);

    let slug = "test-feature";
    let change_dir = repo_path.join("openspec").join("changes").join(slug);
    std::fs::create_dir_all(&change_dir).unwrap();

    // Create tasks.md
    // Using a custom test script: fails on count < 3, succeeds on count >= 3
    let test_script = if cfg!(windows) {
        // Windows batch test that checks if src/lib.rs contains working_fn
        "findstr working_fn src\\lib.rs"
    } else {
        "grep working_fn src/lib.rs"
    };

    let tasks_md = format!(
        r#"
## T-001: Implement working function
- **depends:**
- **done_when:** working_fn exists in src/lib.rs
- **scenarios:**
  - Verify working_fn exists
- **paths_scope:** src/lib.rs
- **test_command:** {}
"#,
        test_script
    );
    std::fs::write(change_dir.join("tasks.md"), tasks_md).unwrap();

    let mut config = RalphConfig::default();
    config.default_autonomy = omnesagent_config::ralph::RalphAutonomy::L0; // autonomous
    config.plateau_window = 3;
    config.minimality_review = true;
    config.debt_harvest = true;

    let executor = Arc::new(MockCycleExecutor::new(repo_path.clone()));
    let knowledge = RalphKnowledgeService::default();
    let gate_mgr = Arc::new(GateManager::new());
    let stop_signal = Arc::new(AtomicBool::new(false));

    // Run Apply cycle
    let res = drive_loop(
        repo_path.clone(),
        slug.to_string(),
        config,
        executor,
        knowledge,
        gate_mgr,
        None,
        stop_signal,
    )
    .await;

    assert!(res.is_ok(), "Loop should complete successfully: {:?}", res);

    // Verify state.json
    let state_file = change_dir.join("state.json");
    assert!(state_file.exists());
    let state = RalphState::read_from(&state_file).unwrap();
    assert_eq!(state.phase, RalphPhase::Archived);
    assert_eq!(state.done, vec!["T-001"]);
    assert_eq!(state.get_iterations("T-001"), 3);

    // Verify journal.jsonl
    let journal_file = change_dir.join("journal.jsonl");
    assert!(journal_file.exists());
    let iterations = Journal::read_iterations(&journal_file).unwrap();
    assert_eq!(iterations.len(), 3);
    assert_eq!(iterations[0].verdict, "failed");
    assert_eq!(iterations[1].verdict, "failed");
    assert_eq!(iterations[2].verdict, "verified");
    assert_eq!(iterations[2].ladder_rung, "minimum");

    // Verify minimality review removed dead_code
    let content = std::fs::read_to_string(repo_path.join("src").join("lib.rs")).unwrap();
    assert!(!content.contains("dead_code"), "Minimality review should have removed dead_code");
    assert!(content.contains("working_fn"), "Working code must remain");

    // Verify git commit was created with CC format
    let git_log = Command::new("git")
        .args(["log", "-n", "1", "--oneline"])
        .current_dir(&repo_path)
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    let log_msg = String::from_utf8_lossy(&git_log.stdout);
    assert!(log_msg.contains("feat(ralph/T-001): i3"));

    // Verify debt harvester finds the ponytail marker
    let cycle_diff = Command::new("git")
        .args(["diff", "HEAD~1"])
        .current_dir(&repo_path)
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    let diff_text = String::from_utf8_lossy(&cycle_diff.stdout);
    let ledger = DebtLedger::harvest_from_diff(&diff_text);
    assert_eq!(ledger.items.len(), 1);
    assert_eq!(ledger.items[0].ceiling, "memory buffer up to 100");
    assert_eq!(
        ledger.items[0].upgrade_trigger.as_deref(),
        Some("upgrade to sqlite")
    );
}

#[tokio::test]
async fn test_concurrency_error_on_active_run() {
    let repo = tempdir().unwrap();
    let repo_path = repo.path().to_path_buf();
    init_git_repo(&repo_path);

    let slug = "active-feature";
    let change_dir = repo_path.join("openspec").join("changes").join(slug);
    std::fs::create_dir_all(&change_dir).unwrap();

    let mut state = RalphState::new(
        "run-1".to_string(),
        slug.to_string(),
        "L0".to_string(),
        "full".to_string(),
    );
    state.phase = RalphPhase::Applying;
    state.write_atomic(&change_dir.join("state.json")).unwrap();

    // Concurrency check should error
    let err = RalphState::check_concurrency(&change_dir.join("state.json"), slug).unwrap_err();
    match err {
        StateError::RunActive(s) => assert_eq!(s, slug),
        _ => panic!("Expected RunActive"),
    }
}

/// Mock executor that always produces identical failure to test plateau stopping.
struct FailingPlateauExecutor;

#[async_trait]
impl Executor for FailingPlateauExecutor {
    async fn run_task(&self, ctx: &TaskContext<'_>) -> Result<ExecutorAnswer, ExecutorError> {
        Ok(ExecutorAnswer {
            task_id: ctx.task.id.clone(),
            hypothesis: "Repeat hypothesis".to_string(),
            plan: vec![],
            result: ExecutionResult::default(),
            ladder_rung: "minimum".to_string(),
            markers: vec![],
            summary: "Failed".to_string(),
            raw_output: String::new(),
        })
    }

    async fn review_delete_list(&self, _diff: &str) -> Result<DeleteList, ExecutorError> {
        Ok(DeleteList::default())
    }
}

#[tokio::test]
async fn test_plateau_stop() {
    let repo = tempdir().unwrap();
    let repo_path = repo.path().to_path_buf();
    init_git_repo(&repo_path);

    let slug = "plateau-feature";
    let change_dir = repo_path.join("openspec").join("changes").join(slug);
    std::fs::create_dir_all(&change_dir).unwrap();

    // Test command that always fails identically with exit code 1
    let fail_cmd = if cfg!(windows) { "exit 1" } else { "exit 1" };

    let tasks_md = format!(
        r#"
## T-001: Stubborn task
- **depends:**
- **done_when:** never
- **scenarios:**
- **paths_scope:**
- **test_command:** {}
"#,
        fail_cmd
    );
    std::fs::write(change_dir.join("tasks.md"), tasks_md).unwrap();

    let mut config = RalphConfig::default();
    config.default_autonomy = omnesagent_config::ralph::RalphAutonomy::L0;
    config.plateau_window = 3;

    let executor = Arc::new(FailingPlateauExecutor);
    let knowledge = RalphKnowledgeService::default();
    let gate_mgr = Arc::new(GateManager::new());
    let stop_signal = Arc::new(AtomicBool::new(false));

    let res = drive_loop(
        repo_path,
        slug.to_string(),
        config,
        executor,
        knowledge,
        gate_mgr,
        None,
        stop_signal,
    )
    .await;

    assert!(res.is_err());
    match res.unwrap_err() {
        DriverError::Stopped(reason) => {
            assert!(reason.contains("Plateau reached"));
        }
        other => panic!("Expected DriverError::Stopped, got: {:?}", other),
    }

    let state = RalphState::read_from(&change_dir.join("state.json")).unwrap();
    assert_eq!(state.phase, RalphPhase::Stopped);
}
