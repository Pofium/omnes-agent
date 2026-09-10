//! Own embedded knowledge layer for Ralph: findings into memory, KAG context assembly.
//!
//! Conforms to §4.3, §7.4, FR-A10 of specification.

use crate::journal::{IterationRecord, Journal};
use crate::tasks::Task;
use omnesagent_api::memory_traits::{Memory, MemoryCategory};
use omnesagent_kag::ProjectService;
use std::path::Path;
use std::sync::Arc;

/// Knowledge manager providing context assembly and findings recording.
#[derive(Clone, Default)]
pub struct RalphKnowledgeService {
    pub memory: Option<Arc<dyn Memory>>,
    pub kag: Option<Arc<ProjectService>>,
}

impl RalphKnowledgeService {
    pub fn new(memory: Option<Arc<dyn Memory>>, kag: Option<Arc<ProjectService>>) -> Self {
        Self { memory, kag }
    }

    /// Check whether the knowledge layer is running in degraded mode (§7.6).
    pub fn is_degraded(&self) -> bool {
        self.memory.is_none() && self.kag.is_none()
    }

    /// Asynchronously assemble task context pack (§7.4).
    ///
    /// Priority order:
    /// 1. Task specification & done_when & scenarios
    /// 2. Negative experience / failed findings
    /// 3. Reuse candidates (`search_symbols` [async])
    /// 4. Blast radius (`analyze_impact` [sync])
    /// 5. Architecture god nodes (`build_context` [sync])
    /// 6. Invariants in `openspec/specs/`
    pub async fn build_context(
        &self,
        task: &Task,
        project_id: &str,
        journal_path: &Path,
        openspec_root: &Path,
        max_tokens: usize,
    ) -> String {
        let mut sections = Vec::new();

        // 1. Task scenarios and description
        let mut task_spec = format!("### Task {} ({})\n", task.id, task.title);
        task_spec.push_str(&format!("**Criteria (done_when):** {}\n", task.done_when));
        if !task.scenarios.is_empty() {
            task_spec.push_str("**Scenarios:**\n");
            for s in &task.scenarios {
                task_spec.push_str(&format!("- {}\n", s));
            }
        }
        sections.push(task_spec);

        // 2. Failed findings & negative experience
        let mut findings_text = String::new();
        if let Some(ref mem) = self.memory {
            let _cat = MemoryCategory::Custom(format!("ralph:{}", task.id));
            if let Ok(entries) = mem.recall(&task.id, 5, None, None, None).await {
                for e in entries {
                    findings_text.push_str(&format!("- [Past Experience]: {}\n", e.content));
                }
            }
        } else if journal_path.exists() {
            // Degraded fallback: read recent failures directly from journal.jsonl
            if let Ok(iterations) = Journal::read_iterations(journal_path) {
                let failures: Vec<_> = iterations
                    .iter()
                    .filter(|it| it.task == task.id && it.verdict == "failed")
                    .rev()
                    .take(3)
                    .collect();
                for f in failures {
                    findings_text.push_str(&format!(
                        "- [Failed i{}]: Hypothesis '{}' failed with tests: {:?}\n",
                        f.n, f.hypothesis, f.tests_summary.failed_tests
                    ));
                }
            }
        }
        if !findings_text.is_empty() {
            sections.push(format!("### Past Negative Experience\n{}", findings_text));
        }

        // 3. Reuse candidates (KAG async search_symbols)
        if let Some(ref kag) = self.kag {
            if let Ok(symbols) = kag.search_symbols(project_id, &task.title, 5).await {
                if !symbols.is_empty() {
                    let mut reuse_text = String::from("### Reuse Candidates (Ladder Rung 2)\n");
                    for node in symbols {
                        reuse_text.push_str(&format!(
                            "- Symbol '{}' in {}: {}\n",
                            node.label,
                            node.file_path.as_deref().unwrap_or("unknown"),
                            node.description.as_deref().unwrap_or("")
                        ));
                    }
                    sections.push(reuse_text);
                }
            }

            // 4. Blast radius analysis
            for scope in &task.paths_scope {
                if let Ok(report) = kag.analyze_impact(project_id, scope, 2) {
                    if !report.affected_nodes.is_empty() {
                        let blast_text = format!(
                            "### Blast Radius for '{}'\nAffected symbols count: {}\n",
                            scope,
                            report.affected_nodes.len()
                        );
                        sections.push(blast_text);
                        break;
                    }
                }
            }

            // 5. Architecture context
            if let Ok(arch_ctx) = kag.build_context(project_id, Some(&task.title)) {
                if !arch_ctx.trim().is_empty() {
                    sections.push(format!("### Architecture Context\n{}", arch_ctx));
                }
            }
        }

        // 6. Invariants in `openspec/specs/`
        let specs_dir = openspec_root.join("specs");
        if specs_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(specs_dir) {
                let mut invariants = String::new();
                for entry in entries.flatten() {
                    if entry.path().extension().is_some_and(|e| e == "md") {
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            invariants.push_str(&format!(
                                "#### Spec {}\n{}\n",
                                entry.file_name().to_string_lossy(),
                                content
                            ));
                        }
                    }
                }
                if !invariants.is_empty() {
                    sections.push(format!("### System Invariants\n{}", invariants));
                }
            }
        }

        let full_text = sections.join("\n\n");

        // Truncate to max_tokens roughly (4 chars per token)
        let max_chars = max_tokens * 4;
        if full_text.len() > max_chars {
            format!("{}...\n[Context truncated at max_tokens limit]", &full_text[..max_chars])
        } else {
            full_text
        }
    }

    /// Record completed or failed iteration into agent's own memory backend (§4.3).
    pub async fn record_iteration(&self, it: &IterationRecord) {
        if let Some(ref mem) = self.memory {
            let key = format!("ralph:{}:i{}", it.task, it.n);
            let content = format!(
                "Ralph task {} i{} verdict={}: hypothesis='{}', ladder_rung='{}', what_done='{}'",
                it.task, it.n, it.verdict, it.hypothesis, it.ladder_rung, it.result.what_done
            );
            let category = MemoryCategory::Custom(format!("ralph:{}", it.task));
            let _ = mem.store(&key, &content, category, None).await;
        }
    }
}
