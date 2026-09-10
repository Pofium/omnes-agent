//! Async loop driver executing the Ralph Apply cycle.
//!
//! Conforms to §7.2, §7.3, §7.5, FR-A3 of specification.

use crate::approval::{GateAction, GateManager};
use crate::debt::DebtLedger;
use crate::events::RalphEvent;
use crate::executor::{Executor, TaskContext};
use crate::git::GitRepo;
use crate::journal::{ExecutionResult, IterationRecord, Journal, JournalEntry, TestsSummary};
use crate::knowledge::RalphKnowledgeService;
use crate::openspec::OpenSpecWorkspace;
use crate::plateau::{compute_failure_fingerprint, is_plateau};
use crate::state::{FailureFingerprint, GateDecision, RalphPhase, RalphState};
use crate::tasks::TaskList;
use chrono::Utc;
use omnesagent_config::ralph::RalphConfig;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use tokio::sync::broadcast::Sender;

#[derive(Debug, Error)]
pub enum DriverError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("State error: {0}")]
    State(#[from] crate::state::StateError),
    #[error("Tasks error: {0}")]
    Tasks(#[from] crate::tasks::TasksError),
    #[error("Executor error: {0}")]
    Executor(#[from] crate::executor::ExecutorError),
    #[error("Git error: {0}")]
    Git(#[from] crate::git::GitError),
    #[error("Journal error: {0}")]
    Journal(#[from] crate::journal::JournalError),
    #[error("OpenSpec error: {0}")]
    OpenSpec(#[from] crate::openspec::OpenSpecError),
    #[error("Loop stopped: {0}")]
    Stopped(String),
}

/// Helper to execute the test command and parse outcomes.
pub async fn run_test_command(cmd_str: &str, working_dir: &Path) -> TestsSummary {
    let output = if cfg!(windows) {
        tokio::process::Command::new("cmd")
            .args(["/C", cmd_str])
            .current_dir(working_dir)
            .stdin(std::process::Stdio::null())
            .output()
            .await
    } else {
        tokio::process::Command::new("sh")
            .args(["-c", cmd_str])
            .current_dir(working_dir)
            .stdin(std::process::Stdio::null())
            .output()
            .await
    };

    let (exit_code, stdout, stderr) = match output {
        Ok(out) => (
            out.status.code().unwrap_or(1),
            String::from_utf8_lossy(&out.stdout).to_string(),
            String::from_utf8_lossy(&out.stderr).to_string(),
        ),
        Err(e) => (1, String::new(), format!("Failed to run test command: {}", e)),
    };

    let combined = format!("{}\n{}", stdout, stderr);
    let mut failed_tests = Vec::new();
    let mut passed = 0;
    let mut failed = 0;

    for line in combined.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("test ") && trimmed.ends_with("... ok") {
            passed += 1;
        } else if trimmed.starts_with("test ") && trimmed.ends_with("... FAILED") {
            failed += 1;
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                failed_tests.push(parts[1].to_string());
            }
        }
    }

    if exit_code != 0 && failed == 0 && failed_tests.is_empty() {
        failed = 1;
        failed_tests.push("test_command_failure".to_string());
    } else if exit_code == 0 && passed == 0 {
        passed = 1; // exit code 0 assumed ok
    }

    let fp = compute_failure_fingerprint(&failed_tests, exit_code);

    TestsSummary {
        passed,
        failed,
        failed_tests,
        fingerprint: fp,
        exit_code,
    }
}

/// Drives the Ralph cycle to completion or plateau/stop.
pub async fn drive_loop(
    repo_root: PathBuf,
    slug: String,
    config: RalphConfig,
    executor: Arc<dyn Executor>,
    knowledge: RalphKnowledgeService,
    gate_mgr: Arc<GateManager>,
    event_tx: Option<Sender<RalphEvent>>,
    stop_signal: Arc<AtomicBool>,
) -> Result<(), DriverError> {
    let ws = OpenSpecWorkspace::new(&repo_root, &slug);
    ws.init()?;

    let state_path = ws.state_path();
    let journal_path = ws.journal_path();
    let git = GitRepo::new(&repo_root);

    let initial_head = git.rev_parse_head().ok();

    // Check concurrency and initialize state
    RalphState::check_concurrency(&state_path, &slug)?;

    let mut state = if state_path.exists() {
        let loaded = RalphState::read_from(&state_path)?;
        if loaded.can_resume() {
            // Resume check (§7.5.1)
            let manual_commits = loaded
                .last_known_head
                .as_ref()
                .is_some_and(|head| initial_head.as_ref() != Some(head));

            if let Some(ref tx) = event_tx {
                let _ = tx.send(RalphEvent::RunResumed {
                    run_id: loaded.run_id.clone(),
                    slug: slug.clone(),
                    manual_commits_detected: manual_commits,
                });
            }
        }
        loaded
    } else {
        let run_id = format!("run_{}", Utc::now().timestamp_millis());
        let autonomy_str = format!("{:?}", config.default_autonomy);
        let mode_str = format!("{:?}", config.mode).to_lowercase();
        let mut s = RalphState::new(run_id.clone(), slug.clone(), autonomy_str, mode_str);
        s.last_known_head = initial_head.clone();
        s.write_atomic(&state_path)?;

        if let Some(ref tx) = event_tx {
            let _ = tx.send(RalphEvent::RunStarted {
                run_id: s.run_id.clone(),
                slug: slug.clone(),
                autonomy: s.autonomy.clone(),
                mode: s.mode.clone(),
            });
        }
        s
    };

    // Human Gate L1/L2 check at proposed phase (§5.3, §7.1)
    if state.phase == RalphPhase::Explore {
        state.transition_to(RalphPhase::Proposed)?;
        state.write_atomic(&state_path)?;
    }

    if state.phase == RalphPhase::Proposed {
        if state.autonomy == "L1" || state.autonomy == "L2" {
            let gate_id = format!("gate_proposed_{}", state.run_id);
            if let Some(ref tx) = event_tx {
                let _ = tx.send(RalphEvent::GatePending {
                    gate_id: gate_id.clone(),
                    run_id: state.run_id.clone(),
                    slug: slug.clone(),
                    phase: "proposed".to_string(),
                    description: "Approve feature specification to begin Apply phase".to_string(),
                });
            }

            let (decision, by) = gate_mgr
                .request_gate(
                    &gate_id,
                    &state.run_id,
                    &slug,
                    "proposed",
                    "Approve specification to begin applying",
                )
                .await;

            state.gates.push(GateDecision {
                phase: "proposed".to_string(),
                decision: format!("{:?}", decision).to_lowercase(),
                by,
                at: Utc::now(),
            });

            if decision == GateAction::Deny {
                state.transition_to(RalphPhase::Stopped)?;
                state.write_atomic(&state_path)?;
                return Err(DriverError::Stopped("Gate denied by human operator".to_string()));
            }
        }
        state.transition_to(RalphPhase::Applying)?;
        state.write_atomic(&state_path)?;
    }

    let tasks_md = ws.read_tasks_markdown()?;
    let task_list = TaskList::parse(&tasks_md)?;

    let mut total_iterations = 0usize;

    // Apply loop (§7.2)
    while state.phase == RalphPhase::Applying {
        if stop_signal.load(Ordering::Relaxed) {
            state.transition_to(RalphPhase::Stopped)?;
            state.write_atomic(&state_path)?;
            if let Some(ref tx) = event_tx {
                let _ = tx.send(RalphEvent::RunStopped {
                    run_id: state.run_id.clone(),
                    slug: slug.clone(),
                    reason: "External stop signal received".to_string(),
                });
            }
            return Ok(());
        }

        let next_task = match task_list.next_ready(&state.done) {
            Some(t) => t.clone(),
            None => {
                // No more ready tasks. If tasks remain uncompleted, they are blocked.
                if state.done.len() < task_list.tasks.len() {
                    state.transition_to(RalphPhase::Stopped)?;
                    state.write_atomic(&state_path)?;
                    return Err(DriverError::Stopped("Remaining tasks are blocked".to_string()));
                }
                // All tasks done! Move to verifying.
                state.transition_to(RalphPhase::Verifying)?;
                state.write_atomic(&state_path)?;
                break;
            }
        };

        let n = state.increment_iteration(&next_task.id);
        total_iterations += 1;

        if n > config.max_iterations_per_task {
            state.mark_blocked(&next_task.id);
            state.write_atomic(&state_path)?;
            continue;
        }

        if total_iterations > config.max_total_iterations {
            state.transition_to(RalphPhase::Stopped)?;
            state.write_atomic(&state_path)?;
            return Err(DriverError::Stopped("Total iteration budget exceeded".to_string()));
        }

        // Build knowledge context (§7.4)
        let knowledge_ctx = knowledge
            .build_context(
                &next_task,
                &slug,
                &journal_path,
                &ws.root,
                config.context_max_tokens,
            )
            .await;

        let task_ctx = TaskContext {
            task: &next_task,
            spec_context: &tasks_md,
            knowledge_context: &knowledge_ctx,
            paths_scope: &next_task.paths_scope,
            mode: &state.mode,
        };

        let git_before = git.rev_parse_head().ok();
        let start_time = Instant::now();

        // Run executor with configured timeout (§5.2, FR-A14)
        let answer = match executor.run_task(&task_ctx).await {
            Ok(ans) => ans,
            Err(e) => {
                // If executor fails or times out, record failed iteration
                let duration_ms = start_time.elapsed().as_millis() as u64;
                let summary = TestsSummary {
                    passed: 0,
                    failed: 1,
                    failed_tests: vec![format!("executor_error: {}", e)],
                    fingerprint: compute_failure_fingerprint(
                        &[format!("executor_error: {}", e)],
                        1,
                    ),
                    exit_code: 1,
                };
                let it = IterationRecord {
                    task: next_task.id.clone(),
                    n,
                    hypothesis: "Executor failed".to_string(),
                    plan: vec![],
                    result: ExecutionResult {
                        what_done: format!("Error: {}", e),
                        errors: vec![e.to_string()],
                        self_assessment: "failed".to_string(),
                    },
                    tests_summary: summary,
                    verdict: "failed".to_string(),
                    ladder_rung: "none".to_string(),
                    markers: vec![],
                    git_before: git_before.clone(),
                    git_after: None,
                    tokens_used: 0,
                    duration_ms,
                    at: Utc::now(),
                };
                Journal::append(&journal_path, &JournalEntry::Iteration(it))?;
                continue;
            }
        };

        // Check paths scope compliance (§5.4)
        if let Err(e) = git.check_paths_scope(&next_task.paths_scope) {
            let _ = git.revert_working_copy();
            let it = IterationRecord {
                task: next_task.id.clone(),
                n,
                hypothesis: answer.hypothesis,
                plan: answer.plan,
                result: ExecutionResult {
                    what_done: "Reverted due to paths_scope violation".to_string(),
                    errors: vec![e.to_string()],
                    self_assessment: "failed".to_string(),
                },
                tests_summary: TestsSummary {
                    passed: 0,
                    failed: 1,
                    failed_tests: vec!["paths_scope_violation".to_string()],
                    fingerprint: compute_failure_fingerprint(&["paths_scope_violation".to_string()], 1),
                    exit_code: 1,
                },
                verdict: "failed".to_string(),
                ladder_rung: answer.ladder_rung,
                markers: vec![],
                git_before,
                git_after: None,
                tokens_used: 0,
                duration_ms: start_time.elapsed().as_millis() as u64,
                at: Utc::now(),
            };
            Journal::append(&journal_path, &JournalEntry::Iteration(it))?;
            continue;
        }

        // Run tests
        let test_cmd = next_task
            .test_command
            .as_deref()
            .unwrap_or(&config.test_command_fallback);

        let mut tests = run_test_command(test_cmd, &repo_root).await;
        let is_green = tests.exit_code == 0;
        let verdict = if is_green { "verified" } else { "failed" };
        let duration_ms = start_time.elapsed().as_millis() as u64;

        state.fingerprints.push(FailureFingerprint {
            task: next_task.id.clone(),
            n,
            fp: tests.fingerprint.clone(),
        });

        if is_green {
            // Minimality review if enabled and mode == full (§7.2.1)
            if config.minimality_review && state.mode == "full" {
                if let Ok(diff) = git.diff(None) {
                    if let Ok(mut dl) = executor.review_delete_list(&diff).await {
                        dl.sanitize_carve_outs();
                        if !dl.is_empty() {
                            if let Ok(applied) = dl.apply(&repo_root) {
                                if applied > 0 {
                                    // Re-run test command to verify minimality deletions didn't break tests
                                    let rerun_tests = run_test_command(test_cmd, &repo_root).await;
                                    if rerun_tests.exit_code != 0 {
                                        // Revert delete list changes if red
                                        let _ = git.revert_working_copy();
                                    } else {
                                        tests = rerun_tests;
                                        if let Some(ref tx) = event_tx {
                                            let _ = tx.send(RalphEvent::ReviewDeleteList {
                                                count: applied,
                                                files_affected: dl
                                                    .items
                                                    .into_iter()
                                                    .map(|i| i.file)
                                                    .collect(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Commit iteration
            let commit_hash = git.commit_iteration(
                &next_task.id,
                n,
                &answer.summary,
                config.commit_format,
            )?;

            state.last_known_head = Some(commit_hash.clone());
            state.mark_done(&next_task.id);
            state.write_atomic(&state_path)?;

            let it = IterationRecord {
                task: next_task.id.clone(),
                n,
                hypothesis: answer.hypothesis,
                plan: answer.plan,
                result: answer.result,
                tests_summary: tests,
                verdict: verdict.to_string(),
                ladder_rung: answer.ladder_rung.clone(),
                markers: answer.markers,
                git_before,
                git_after: Some(commit_hash),
                tokens_used: 0,
                duration_ms,
                at: Utc::now(),
            };

            Journal::append(&journal_path, &JournalEntry::Iteration(it.clone()))?;
            knowledge.record_iteration(&it).await;

            if let Some(ref tx) = event_tx {
                let _ = tx.send(RalphEvent::IterationCompleted {
                    task: next_task.id.clone(),
                    n,
                    verdict: verdict.to_string(),
                    ladder_rung: answer.ladder_rung,
                    duration_ms,
                });
            }
        } else {
            // Test failed: record iteration and check for plateau (§7.2, §7.5)
            let it = IterationRecord {
                task: next_task.id.clone(),
                n,
                hypothesis: answer.hypothesis,
                plan: answer.plan,
                result: answer.result,
                tests_summary: tests.clone(),
                verdict: verdict.to_string(),
                ladder_rung: answer.ladder_rung,
                markers: answer.markers,
                git_before,
                git_after: None,
                tokens_used: 0,
                duration_ms,
                at: Utc::now(),
            };

            Journal::append(&journal_path, &JournalEntry::Iteration(it.clone()))?;
            knowledge.record_iteration(&it).await;
            state.write_atomic(&state_path)?;

            if let Some(ref tx) = event_tx {
                let _ = tx.send(RalphEvent::IterationFailed {
                    task: next_task.id.clone(),
                    n,
                    fingerprint: tests.fingerprint.clone(),
                    exit_code: tests.exit_code,
                });
            }

            // Check plateau window
            let task_fps: Vec<&str> = state
                .fingerprints
                .iter()
                .filter(|f| f.task == next_task.id)
                .map(|f| f.fp.as_str())
                .collect();

            if is_plateau(&task_fps, config.plateau_window) {
                let reason = format!(
                    "Plateau reached on task {} with fingerprint {}",
                    next_task.id, tests.fingerprint
                );
                state.transition_to(RalphPhase::Stopped)?;
                state.write_atomic(&state_path)?;
                if let Some(ref tx) = event_tx {
                    let _ = tx.send(RalphEvent::RunStopped {
                        run_id: state.run_id.clone(),
                        slug: slug.clone(),
                        reason: reason.clone(),
                    });
                }
                return Err(DriverError::Stopped(reason));
            }
        }
    }

    // Phase: Verifying (§7.3)
    if state.phase == RalphPhase::Verifying {
        // Debt harvesting if enabled
        if config.debt_harvest {
            if let Ok(cycle_diff) = git.diff(None) {
                let ledger = DebtLedger::harvest_from_diff(&cycle_diff);
                if let Some(ref tx) = event_tx {
                    let _ = tx.send(RalphEvent::DebtHarvested {
                        count: ledger.items.len(),
                        no_trigger_count: ledger.no_trigger_count(),
                    });
                }
            }
        }

        // Gate L2 check before archiving
        if state.autonomy == "L2" {
            let gate_id = format!("gate_archive_{}", state.run_id);
            if let Some(ref tx) = event_tx {
                let _ = tx.send(RalphEvent::GatePending {
                    gate_id: gate_id.clone(),
                    run_id: state.run_id.clone(),
                    slug: slug.clone(),
                    phase: "verifying".to_string(),
                    description: "Approve feature archiving and finalization".to_string(),
                });
            }

            let (decision, by) = gate_mgr
                .request_gate(
                    &gate_id,
                    &state.run_id,
                    &slug,
                    "verifying",
                    "Approve final archiving of feature",
                )
                .await;

            state.gates.push(GateDecision {
                phase: "verifying".to_string(),
                decision: format!("{:?}", decision).to_lowercase(),
                by,
                at: Utc::now(),
            });

            if decision == GateAction::Deny {
                state.transition_to(RalphPhase::Stopped)?;
                state.write_atomic(&state_path)?;
                return Err(DriverError::Stopped("Archival gate denied by human".to_string()));
            }
        }

        state.transition_to(RalphPhase::Archived)?;
        state.write_atomic(&state_path)?;
    }

    Ok(())
}
