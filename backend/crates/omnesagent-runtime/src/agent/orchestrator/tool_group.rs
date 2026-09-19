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
        || lower.starts_with("ob2h:project_scan")
        || lower.starts_with("ob2h:project_context")
        || lower.starts_with("ob2h:project_report")
        || lower.starts_with("ob2h:graph_search")
        || lower.starts_with("ob2h:memory_search")
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

