use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use omnesagent_api::tool::{Tool, ToolOutput, ToolResult};
use omnesagent_config::policy::SecurityPolicy;
use omnesagent_config::policy::ToolOperation;
use omnesagent_memory::Memory;

/// Tool for explicitly merging duplicate or conflicting memory entries into a canonical one.
pub struct MemoryMergeTool {
    memory: Arc<dyn Memory>,
    security: Arc<SecurityPolicy>,
}

impl MemoryMergeTool {
    pub fn new(memory: Arc<dyn Memory>, security: Arc<SecurityPolicy>) -> Self {
        Self { memory, security }
    }
}

#[async_trait]
impl Tool for MemoryMergeTool {
    fn name(&self) -> &str {
        "memory_merge"
    }

    fn description(&self) -> &str {
        "Explicitly merge two or more memory records into a single canonical record. The canonical record preserves/combines content and importance, while absorbed records are removed."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "keys": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of memory keys to merge (must contain at least 2 keys)"
                },
                "canonical_key": {
                    "type": "string",
                    "description": "Optional key to use as canonical destination. If omitted, the record with highest importance/recency is chosen."
                },
                "merged_content": {
                    "type": "string",
                    "description": "Optional explicit content for the merged record. If omitted, contents are combined separated by '---'."
                },
                "note": {
                    "type": "string",
                    "description": "Optional note describing the reason for merging"
                }
            },
            "required": ["keys"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let keys_val = args.get("keys").and_then(|v| v.as_array()).ok_or_else(|| {
            anyhow::Error::msg("Missing 'keys' array parameter")
        })?;

        let keys: Vec<String> = keys_val
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        if keys.len() < 2 {
            return Ok(ToolResult::err("memory_merge requires at least 2 keys to merge"));
        }

        if let Err(error) = self
            .security
            .enforce_tool_operation(ToolOperation::Act, "memory_merge")
        {
            return Ok(ToolResult {
                success: false,
                output: ToolOutput::default(),
                error: Some(error),
            });
        }

        let canonical_key_req = args.get("canonical_key").and_then(|v| v.as_str());
        let note = args.get("note").and_then(|v| v.as_str());

        // Загружаем все записи
        let mut entries = Vec::new();
        for k in &keys {
            match self.memory.get(k).await? {
                Some(entry) => entries.push(entry),
                None => {
                    return Ok(ToolResult::err(format!("Memory with key '{k}' not found")));
                }
            }
        }

        // Выбираем каноническую запись
        let canonical_idx = if let Some(ck) = canonical_key_req {
            entries
                .iter()
                .position(|e| e.key == ck)
                .ok_or_else(|| anyhow::anyhow!("canonical_key '{ck}' is not in the provided keys list"))?
        } else {
            // По максимальной важности
            entries
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| {
                    a.importance
                        .unwrap_or(0.5)
                        .partial_cmp(&b.importance.unwrap_or(0.5))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(idx, _)| idx)
                .unwrap_or(0)
        };

        let canonical = entries[canonical_idx].clone();
        let absorbed: Vec<_> = entries
            .into_iter()
            .enumerate()
            .filter(|(idx, _)| *idx != canonical_idx)
            .map(|(_, e)| e)
            .collect();

        // Формируем объединенный контент
        let combined_content = if let Some(mc) = args.get("merged_content").and_then(|v| v.as_str()) {
            let mut c = mc.to_string();
            if let Some(n) = note {
                c.push_str(&format!("\n[Merged Note]: {n}"));
            }
            c
        } else {
            let mut c = canonical.content.clone();
            for a in &absorbed {
                if !c.contains(&a.content) {
                    c.push_str("\n---\n");
                    c.push_str(&a.content);
                }
            }
            if let Some(n) = note {
                c.push_str(&format!("\n[Merged Note]: {n}"));
            }
            c
        };

        // Сохраняем каноническую
        self.memory
            .store(
                &canonical.key,
                &combined_content,
                canonical.category,
                canonical.session_id.as_deref(),
            )
            .await?;

        // Удаляем поглощенные
        let mut deleted_keys = Vec::new();
        for a in &absorbed {
            let _ = self.memory.forget(&a.key).await?;
            deleted_keys.push(a.key.clone());
        }

        Ok(ToolResult::ok(json!({
            "status": "merged",
            "canonical_key": canonical.key,
            "absorbed_keys": deleted_keys,
            "note": note,
            "message": format!("Successfully merged {} records into canonical '{}'", deleted_keys.len() + 1, canonical.key)
        })))
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

    #[test]
    fn name_and_schema() {
        let (_tmp, mem) = test_mem();
        let tool = MemoryMergeTool::new(mem, test_security());
        assert_eq!(tool.name(), "memory_merge");
        let schema = tool.parameters_schema();
        assert!(schema["properties"]["keys"].is_object());
        assert!(schema["properties"]["canonical_key"].is_object());
    }

    #[tokio::test]
    async fn merge_two_memories_into_canonical() {
        let (_tmp, mem) = test_mem();
        let tool = MemoryMergeTool::new(mem.clone(), test_security());

        mem.store("fact_1", "User likes Rust", MemoryCategory::Core, None).await.unwrap();
        mem.store("fact_2", "User also likes Go", MemoryCategory::Daily, None).await.unwrap();

        let result = tool
            .execute(json!({
                "keys": ["fact_1", "fact_2"],
                "canonical_key": "fact_1",
                "merged_content": "User prefers Rust and likes Go"
            }))
            .await
            .unwrap();

        assert!(result.success);
        let parsed: serde_json::Value = serde_json::from_str(&result.output).unwrap();
        assert_eq!(parsed["canonical_key"], "fact_1");
        assert_eq!(parsed["status"], "merged");

        let canon = mem.get("fact_1").await.unwrap().expect("fact_1 exists");
        assert_eq!(canon.content, "User prefers Rust and likes Go");

        let absorbed = mem.get("fact_2").await.unwrap();
        assert!(absorbed.is_none());
    }

    #[tokio::test]
    async fn merge_requires_at_least_two_keys() {
        let (_tmp, mem) = test_mem();
        let tool = MemoryMergeTool::new(mem.clone(), test_security());

        let res = tool.execute(json!({"keys": ["only_one"]})).await.unwrap();
        assert!(!res.success);
    }
}

