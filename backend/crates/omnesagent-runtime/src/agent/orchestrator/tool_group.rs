// Tool grouping and intent-based dynamic tool pruning.

use omnesagent_api::tool::ToolSpec;
use super::intent::AgentIntent;

/// Logical classification of tools for intent-based gating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolGroup {
    /// No tools allowed (Direct conversational mode).
    None,

    /// Read-only inspection and analysis tools (Code Exploration).
    ReadOnly,

    /// Full tools: read, write, edit, shell, git, memory, etc. (Engineering).
    FullStack,

    /// DevOps-specific tools: shell, docker, kubernetes, system inspection.
    DevOps,
}

/// Filter an inventory of tools according to the target intent.
pub fn filter_tools_for_intent(all_tools: &[ToolSpec], intent: AgentIntent) -> Vec<ToolSpec> {
    filter_tools_for_intent_with_query(all_tools, intent, None, None)
}

/// Filter an inventory of tools with semantic top-K retrieval for external MCP tools.
pub fn filter_tools_for_intent_with_query(
    all_tools: &[ToolSpec],
    intent: AgentIntent,
    query: Option<&str>,
    max_mcp_tools: Option<usize>,
) -> Vec<ToolSpec> {
    match intent {
        AgentIntent::DirectChat => {
            // Zero tool definitions injected for pure chat.
            // Drastically lowers latency, eliminates tool hallucinations and saves prompt tokens.
            Vec::new()
        }
        AgentIntent::CodeExploration => {
            let read_only: Vec<ToolSpec> = all_tools
                .iter()
                .filter(|tool| is_read_only_tool(&tool.name))
                .cloned()
                .collect();
            prune_mcp_tools(&read_only, query, max_mcp_tools.unwrap_or(10))
        }
        AgentIntent::EngineeringTask | AgentIntent::SystemAdmin => {
            // Engineering and Admin modes retain full core capabilities while ranking MCP tools
            prune_mcp_tools(all_tools, query, max_mcp_tools.unwrap_or(12))
        }
    }
}

/// Determines whether a tool is an external MCP or namespaced tool.
pub fn is_mcp_or_external_tool(name: &str) -> bool {
    name.contains("__")
        || name.contains(':')
        || name.starts_with("mcp_")
        || name.starts_with("searxng_")
        || name.starts_with("figma_")
}

/// Compute semantic relevance score between a tool and query terms.
pub fn score_tool_relevance(tool: &ToolSpec, query_terms: &[&str]) -> usize {
    if query_terms.is_empty() {
        return 0;
    }
    let lower_name = tool.name.to_ascii_lowercase();
    let lower_desc = tool.description.to_ascii_lowercase();

    let mut score = 0;
    for &term in query_terms {
        if term.len() < 2 {
            continue;
        }
        let lower_term = term.to_ascii_lowercase();
        // Exact name match or token match in tool name gets highest weight
        if lower_name == lower_term {
            score += 15;
        } else if lower_name.contains(&lower_term) {
            score += 8;
        }
        // Description match gets moderate weight
        if lower_desc.contains(&lower_term) {
            score += 3;
        }
    }
    score
}

/// Prunes external MCP tools to top-K if their count exceeds the threshold,
/// while always preserving core native tools.
pub fn prune_mcp_tools(tools: &[ToolSpec], query: Option<&str>, max_mcp: usize) -> Vec<ToolSpec> {
    let mut core_tools = Vec::new();
    let mut mcp_tools = Vec::new();

    for tool in tools {
        if is_mcp_or_external_tool(&tool.name) {
            mcp_tools.push(tool.clone());
        } else {
            core_tools.push(tool.clone());
        }
    }

    if mcp_tools.len() <= max_mcp {
        // Under threshold: keep all tools
        let mut result = core_tools;
        result.extend(mcp_tools);
        return result;
    }

    // Over threshold: rank MCP tools by relevance to the query
    let terms: Vec<&str> = match query {
        Some(q) => q.split_whitespace().collect(),
        None => Vec::new(),
    };

    let mut scored: Vec<(ToolSpec, usize)> = mcp_tools
        .into_iter()
        .map(|tool| {
            let score = score_tool_relevance(&tool, &terms);
            (tool, score)
        })
        .collect();

    // Sort descending by relevance score
    scored.sort_by_key(|entry| std::cmp::Reverse(entry.1));

    let top_mcp: Vec<ToolSpec> = scored.into_iter().take(max_mcp).map(|(t, _)| t).collect();

    let mut result = core_tools;
    result.extend(top_mcp);
    result
}

/// Determines whether a tool is strictly read-only inspection.
pub fn is_read_only_tool(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();

    // Explicit mutating keywords
    if lower.contains("write")
        || lower.contains("edit")
        || lower.contains("replace")
        || lower.contains("delete")
        || lower.contains("remove")
        || lower.contains("create")
        || lower.contains("patch")
        || lower.contains("modify")
        || lower.contains("commit")
        || lower.contains("push")
        || lower.contains("execute")
        || lower.contains("run_command")
        || lower.contains("shell")
        || lower.contains("powershell")
        || lower.contains("bash")
        || lower.contains("cmd")
    {
        return false;
    }

    // Read-only inspection prefixes and names
    lower.starts_with("read_")
        || lower.starts_with("view_")
        || lower.starts_with("list_")
        || lower.starts_with("search_")
        || lower.starts_with("grep_")
        || lower.starts_with("find_")
        || lower.starts_with("get_")
        || lower.starts_with("inspect_")
        || lower.contains("explore")
        || lower.contains("status")
        || lower.contains("log")
        || lower.contains("diff")
        // ob2h MCP tools arrive namespaced as `<server>__<tool>` (double
        // underscore, see mcp_client.rs), so the read-only allowlist must use
        // `ob2h__` — a single-colon prefix never matched a real tool name.
        || lower.starts_with("ob2h__project_scan")
        || lower.starts_with("ob2h__project_context")
        || lower.starts_with("ob2h__project_report")
        || lower.starts_with("ob2h__project_impact")
        || lower.starts_with("ob2h__graph_search")
        || lower.starts_with("ob2h__graph_reason")
        || lower.starts_with("ob2h__graph_stats")
        || lower.starts_with("ob2h__memory_search")
        || lower.starts_with("ob2h__memory_context")
        || lower.starts_with("ob2h__omnes_stats")
        || lower.starts_with("ob2h__secrets_status")
        // ob2h v1.9.9 (Ф7.2): seven new read-only lifecycle tools plus the ten
        // read-only names deferred from Ф0.6. `secrets_get` intentionally stays
        // write-class (the Ф4.1 decision is not revisited).
        || lower.starts_with("ob2h__lesson_recall")
        || lower.starts_with("ob2h__lesson_list")
        || lower.starts_with("ob2h__slot_list")
        || lower.starts_with("ob2h__slot_get")
        || lower.starts_with("ob2h__memory_timeline")
        || lower.starts_with("ob2h__memory_verify")
        || lower.starts_with("ob2h__memory_audit")
        || lower.starts_with("ob2h__project_scan_status")
        || lower.starts_with("ob2h__project_call_path")
        || lower.starts_with("ob2h__project_graph_search")
        || lower.starts_with("ob2h__ast_diff")
        || lower.starts_with("ob2h__ast_history")
        || lower.starts_with("ob2h__dream_log")
        || lower.starts_with("ob2h__dream_status")
        || lower.starts_with("ob2h__ralph_context")
        || lower.starts_with("ob2h__ralph_report")
        || lower.starts_with("ob2h__session_log")
        || lower.starts_with("codegraph_")
        || lower.starts_with("searxng_")
        || lower == "web_search"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_tool(name: &str) -> ToolSpec {
        ToolSpec::new(name, "test tool", serde_json::json!({}))
    }

    #[test]
    fn test_direct_chat_has_no_tools() {
        let tools = vec![
            dummy_tool("read_file"),
            dummy_tool("write_to_file"),
            dummy_tool("run_command"),
        ];
        let filtered = filter_tools_for_intent(&tools, AgentIntent::DirectChat);
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_code_exploration_read_only() {
        let tools = vec![
            dummy_tool("read_file"),
            dummy_tool("grep_search"),
            dummy_tool("write_to_file"),
            dummy_tool("replace_file_content"),
            dummy_tool("run_command"),
        ];
        let filtered = filter_tools_for_intent(&tools, AgentIntent::CodeExploration);
        let names: Vec<_> = filtered.into_iter().map(|t| t.name).collect();
        assert_eq!(names, vec!["read_file", "grep_search"]);
    }

    #[test]
    fn test_ob2h_read_only_uses_real_double_underscore_prefix() {
        // Regression: the read-only allowlist once used `ob2h:` (single colon),
        // which never matched the `<server>__<tool>` names the MCP client emits,
        // so ob2h read tools survived only via the generic `__` rule and write
        // tools were never explicitly excluded.
        let read_only = [
            "ob2h__project_scan",
            "ob2h__project_context",
            "ob2h__project_report",
            "ob2h__project_impact",
            "ob2h__graph_search",
            "ob2h__graph_reason",
            "ob2h__graph_stats",
            "ob2h__memory_search",
            "ob2h__memory_context",
            "ob2h__omnes_stats",
            "ob2h__secrets_status",
            // ob2h v1.9.9 read-only lifecycle tools (Ф7.2).
            "ob2h__lesson_recall",
            "ob2h__lesson_list",
            "ob2h__slot_list",
            "ob2h__slot_get",
            "ob2h__memory_timeline",
            "ob2h__memory_verify",
            "ob2h__memory_audit",
            // Ф0.6 follow-up read-only names.
            "ob2h__project_scan_status",
            "ob2h__project_call_path",
            "ob2h__project_graph_search",
            "ob2h__ast_diff",
            "ob2h__ast_history",
            "ob2h__dream_log",
            "ob2h__dream_status",
            "ob2h__ralph_context",
            "ob2h__ralph_report",
            "ob2h__session_log",
        ];
        for name in read_only {
            assert!(is_read_only_tool(name), "{name} should be read-only");
        }

        let mutating = [
            "ob2h__memory_save",
            "ob2h__memory_forget",
            "ob2h__secrets_get",
            "ob2h__secrets_set",
            "ob2h__secrets_delete",
            "ob2h__secrets_lock",
            "ob2h__workspace_write",
            "ob2h__project_init",
            "ob2h__knowledge_extract",
            "ob2h__dream_run",
            "ob2h__session_ingest",
            "ob2h__omnes_backup",
            // ob2h v1.9.9 write/privileged lifecycle tools (Ф7.3): must never
            // reach CodeExploration, only EngineeringTask/SystemAdmin.
            "ob2h__lesson_save",
            "ob2h__lesson_delete",
            "ob2h__slot_create",
            "ob2h__slot_append",
            "ob2h__slot_replace",
            "ob2h__slot_delete",
            "ob2h__session_recap",
            "ob2h__viewer_open",
            // ob2h v2.0 write/HITL tools
            "ob2h__knowledge_reconcile",
            "ob2h__memory_feedback",
        ];
        for name in mutating {
            assert!(!is_read_only_tool(name), "{name} must not be read-only");
        }
    }

    #[test]
    fn test_ob2h_v199_lifecycle_classification() {
        // Ф7.3 / Appendix A: the full v1.9.9 lifecycle matrix — read-only
        // inspection tools stay available in CodeExploration, while write and
        // privileged tools (lesson/slot writes, recap, viewer spawn) reach only
        // EngineeringTask/SystemAdmin. A generous MCP budget is used so the test
        // exercises the intent filter, not the top-K pruning.
        let read_only = [
            "ob2h__lesson_recall",
            "ob2h__lesson_list",
            "ob2h__slot_list",
            "ob2h__slot_get",
            "ob2h__memory_timeline",
            "ob2h__memory_verify",
            "ob2h__memory_audit",
        ];
        let write_or_privileged = [
            "ob2h__lesson_save",
            "ob2h__lesson_delete",
            "ob2h__slot_create",
            "ob2h__slot_append",
            "ob2h__slot_replace",
            "ob2h__slot_delete",
            "ob2h__session_recap",
            "ob2h__viewer_open",
        ];

        let tools: Vec<ToolSpec> = read_only
            .iter()
            .chain(write_or_privileged.iter())
            .map(|name| dummy_tool(name))
            .collect();

        let explored = filter_tools_for_intent_with_query(
            &tools,
            AgentIntent::CodeExploration,
            None,
            Some(64),
        );
        let explored_names: Vec<&str> = explored.iter().map(|t| t.name.as_str()).collect();
        for name in read_only {
            assert!(
                explored_names.contains(&name),
                "{name} must stay in CodeExploration"
            );
        }
        for name in write_or_privileged {
            assert!(
                !explored_names.contains(&name),
                "{name} must not enter CodeExploration"
            );
        }

        for intent in [AgentIntent::EngineeringTask, AgentIntent::SystemAdmin] {
            let filtered =
                filter_tools_for_intent_with_query(&tools, intent, None, Some(64));
            assert_eq!(filtered.len(), tools.len(), "all tools retained for {intent:?}");
        }
    }

    #[test]
    fn test_ob2h_vault_tools_are_engineering_only() {
        // Ф4.1: only the read-class `secrets_status` is allowed in
        // CodeExploration; get/set/delete/lock stay EngineeringTask/SystemAdmin.
        let tools = vec![
            dummy_tool("ob2h__secrets_status"),
            dummy_tool("ob2h__secrets_get"),
            dummy_tool("ob2h__secrets_set"),
            dummy_tool("ob2h__secrets_delete"),
            dummy_tool("ob2h__secrets_lock"),
        ];

        let explored = filter_tools_for_intent(&tools, AgentIntent::CodeExploration);
        let names: Vec<_> = explored.into_iter().map(|t| t.name).collect();
        assert_eq!(names, vec!["ob2h__secrets_status"]);

        let engineering = filter_tools_for_intent(&tools, AgentIntent::EngineeringTask);
        assert_eq!(engineering.len(), tools.len());

        let admin = filter_tools_for_intent(&tools, AgentIntent::SystemAdmin);
        assert_eq!(admin.len(), tools.len());
    }

    #[test]
    fn test_code_exploration_excludes_ob2h_write_tools() {
        let tools = vec![
            dummy_tool("ob2h__memory_search"),
            dummy_tool("ob2h__graph_reason"),
            dummy_tool("ob2h__memory_save"),
            dummy_tool("ob2h__secrets_set"),
        ];
        let filtered = filter_tools_for_intent(&tools, AgentIntent::CodeExploration);
        let names: Vec<_> = filtered.into_iter().map(|t| t.name).collect();
        assert_eq!(names, vec!["ob2h__memory_search", "ob2h__graph_reason"]);
    }

    #[test]
    fn test_engineering_retains_all() {
        let tools = vec![
            dummy_tool("read_file"),
            dummy_tool("write_to_file"),
            dummy_tool("run_command"),
        ];
        let filtered = filter_tools_for_intent(&tools, AgentIntent::EngineeringTask);
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_mcp_top_k_ranking() {
        let tools = vec![
            dummy_tool("read_file"),
            dummy_tool("write_to_file"),
            // 4 MCP tools:
            dummy_tool("github__create_pull_request"),
            dummy_tool("github__list_issues"),
            dummy_tool("postgres__query_database"),
            dummy_tool("docker__run_container"),
        ];

        // Ask a GitHub-related question with max_mcp = 2
        let filtered = filter_tools_for_intent_with_query(
            &tools,
            AgentIntent::EngineeringTask,
            Some("Создай PR и проверь issues"),
            Some(2),
        );

        let names: Vec<_> = filtered.into_iter().map(|t| t.name).collect();
        // Core tools must be present:
        assert!(names.contains(&"read_file".to_string()));
        assert!(names.contains(&"write_to_file".to_string()));
        // Ranked top MCP tools should be github PR and issues:
        assert!(names.contains(&"github__create_pull_request".to_string()));
        assert!(names.contains(&"github__list_issues".to_string()));
        // Docker and Postgres should be pruned:
        assert!(!names.contains(&"postgres__query_database".to_string()));
        assert!(!names.contains(&"docker__run_container".to_string()));
    }
}

