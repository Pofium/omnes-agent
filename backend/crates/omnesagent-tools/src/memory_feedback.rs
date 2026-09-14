use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use omnesagent_api::tool::{Tool, ToolOutput, ToolResult};
use omnesagent_config::policy::{SecurityPolicy, ToolOperation};
use omnesagent_memory::Memory;

/// Tool to record feedback on long-term memory entries and adapt their trust score.
pub struct MemoryFeedbackTool {
    memory: Arc<dyn Memory>,
    security: Arc<SecurityPolicy>,
}

impl MemoryFeedbackTool {
    pub fn new(memory: Arc<dyn Memory>, security: Arc<SecurityPolicy>) -> Self {
        Self { memory, security }
    }
}

#[async_trait]
impl Tool for MemoryFeedbackTool {
    fn name(&self) -> &str {
        "memory_feedback"
    }

    fn description(&self) -> &str {
        "Record agent or user feedback on a retrieved memory entry (helpful, unhelpful, outdated) to adjust its trust score. Entries with high trust are prioritized; entries with degraded trust are deprioritized or marked for review."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "key": {
                    "type": "string",
                    "description": "The key of the memory to record feedback for"
                },
                "verdict": {
                    "type": "string",
                    "enum": ["helpful", "unhelpful", "outdated"],
                    "description": "The feedback verdict: 'helpful' (+0.15 trust), 'unhelpful' (-0.20 trust), or 'outdated' (-0.30 trust)"
                },
                "note": {
                    "type": "string",
                    "description": "Optional comment or context explaining why the memory was rated this way"
                }
            },
            "required": ["key", "verdict"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let key = args.get("key").and_then(|v| v.as_str()).ok_or_else(|| {
            ::omnesagent_log::record!(
                WARN,
                ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Reject)
                    .with_outcome(::omnesagent_log::EventOutcome::Failure)
                    .with_attrs(::serde_json::json!({"param": "key"})),
                "memory_feedback: missing key parameter"
            );
            anyhow::Error::msg("Missing 'key' parameter")
        })?;

        let verdict = args.get("verdict").and_then(|v| v.as_str()).ok_or_else(|| {
            ::omnesagent_log::record!(
                WARN,
                ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Reject)
                    .with_outcome(::omnesagent_log::EventOutcome::Failure)
                    .with_attrs(::serde_json::json!({"param": "verdict"})),
                "memory_feedback: missing verdict parameter"
            );
            anyhow::Error::msg("Missing 'verdict' parameter")
        })?;

        let note = args.get("note").and_then(|v| v.as_str());

        if let Err(error) = self
            .security
            .enforce_tool_operation(ToolOperation::Act, "memory_feedback")
        {
            return Ok(ToolResult {
                success: false,
                output: ToolOutput::default(),
                error: Some(error),
            });
        }

        match self.memory.record_feedback(key, verdict, note).await {
            Ok(Some(new_trust)) => Ok(ToolResult {
                success: true,
                output: format!(
                    "Recorded feedback '{verdict}' for memory '{key}'. New trust score: {new_trust:.4}"
                )
                .into(),
                error: None,
            }),
            Ok(None) => Ok(ToolResult {
                success: true,
                output: format!("No memory found with key: {key}").into(),
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: ToolOutput::default(),
                error: Some(format!("Failed to record feedback for memory '{key}': {e}")),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use omnesagent_memory::{MemoryCategory, SqliteMemory};

    fn test_security() -> Arc<SecurityPolicy> {
        Arc::new(SecurityPolicy::default())
    }

    fn test_mem() -> (TempDir, Arc<dyn Memory>) {
        let tmp = TempDir::new().unwrap();
        let mem = SqliteMemory::new("test", tmp.path()).unwrap();
        (tmp, Arc::new(mem))
    }

    #[tokio::test]
    async fn test_memory_feedback_tool() {
        let (_tmp, mem) = test_mem();
        let sec = test_security();

        mem.store("fact1", "Rust is fast", MemoryCategory::Core, None)
            .await
            .unwrap();

        let tool = MemoryFeedbackTool::new(mem.clone(), sec);
        assert_eq!(tool.name(), "memory_feedback");

        // Helpful (+0.15 => 0.65)
        let res = tool
            .execute(json!({
                "key": "fact1",
                "verdict": "helpful",
                "note": "verified in benchmark"
            }))
            .await
            .unwrap();
        assert!(res.success);
        let out = res.output.as_str();
        assert!(out.contains("New trust score: 0.6500"));

        // Non-existent key
        let res_missing = tool
            .execute(json!({
                "key": "nonexistent",
                "verdict": "unhelpful"
            }))
            .await
            .unwrap();
        assert!(res_missing.success);
        assert!(res_missing.output.as_str().contains("No memory found"));
    }
}
