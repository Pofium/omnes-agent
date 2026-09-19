//! OmnesAgent Compression: Native in-process token compression layer.
//!
//! Provides zero-dependency, out-of-the-box token reduction across:
//! - MCP Tool Schemas (Atlassian Labs algorithm: -70% to -90%)
//! - Cross-turn file and output deduplication (sqz algorithm: ~92% on repeats)
//! - Structural JSON array & log crushing (Headroom SmartCrusher algorithm)
//! - AST-based function body collapsing for context files
//! - Model-family adaptive routing (DeepSeek, GLM, Claude, OpenAI, Qwen)

pub mod ccr_store;
pub mod code_ast;
pub mod mcp_schema;
pub mod model_router;
pub mod smart_crusher;
pub mod sqz_dedup;

pub use ccr_store::{CcrEntry, CcrStore};
pub use code_ast::AstCodeCompressor;
pub use mcp_schema::{McpSchemaCompressor, SchemaTransformMode};
pub use model_router::{CompressionPolicy, ModelFamily};
pub use smart_crusher::SmartCrusher;
pub use sqz_dedup::{BlockEntry, DedupRefFormat, SqzDedupEngine};

/// Unified in-process token compressor for agent sessions.
#[derive(Clone)]
pub struct TokenCompressor {
    pub dedup: SqzDedupEngine,
    pub ccr: CcrStore,
}

impl Default for TokenCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenCompressor {
    /// Creates a new token compressor instance with fresh in-memory stores.
    #[must_use]
    pub fn new() -> Self {
        let ccr = CcrStore::new();
        let dedup = SqzDedupEngine::default();
        Self { dedup, ccr }
    }

    /// Compresses a tool output string using SmartCrusher and DedupEngine according to policy.
    pub fn compress_tool_output(&self, output: &str, policy: &CompressionPolicy) -> String {
        let mut text = output.to_string();

        // 1. Structural crushing if enabled
        if policy.enable_smart_crusher {
            let crusher = SmartCrusher::new(self.ccr.clone());
            text = crusher.crush_output(&text);
        }

        // 2. Cross-turn deduplication
        if policy.enable_dedup {
            text = self.dedup.process_text(&text, policy.dedup_ref_format);
        }

        text
    }

    /// Compresses file content for background context using AST collapsing and deduplication.
    pub fn compress_file_content(&self, content: &str, policy: &CompressionPolicy) -> String {
        let mut text = content.to_string();

        if policy.enable_ast_code_collapse {
            text = AstCodeCompressor::collapse_function_bodies(&text, 4);
        }

        if policy.enable_dedup {
            text = self.dedup.process_text(&text, policy.dedup_ref_format);
        }

        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_compressor_pipeline() {
        let compressor = TokenCompressor::new();
        let policy = CompressionPolicy::for_family(ModelFamily::DeepSeek);

        let repeated_json = serde_json::json!([
            {"id": 1, "status": "active", "data": "very long data string repeated across turns to exceed byte limit"},
            {"id": 2, "status": "active", "data": "another long data string repeated across turns to exceed byte limit"},
            {"id": 3, "status": "active", "data": "third long data string repeated across turns to exceed byte limit"}
        ]).to_string();

        // First pass
        let out1 = compressor.compress_tool_output(&repeated_json, &policy);
        assert!(!out1.starts_with("§ref:"));

        // Second pass: deduplicated!
        let out2 = compressor.compress_tool_output(&repeated_json, &policy);
        assert!(out2.starts_with("§ref:"));
    }
}
