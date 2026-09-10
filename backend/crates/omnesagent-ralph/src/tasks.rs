//! Parser and dependency graph resolver for `tasks.md`.
//!
//! Conforms to §5.4, FR-A2, FR-A12 of specification.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TasksError {
    #[error("Parse error on line {0}: {1}")]
    Parse(usize, String),
    #[error("Cyclic dependency detected involving tasks: {0}")]
    CyclicDependency(String),
    #[error("Unknown dependency '{0}' referenced in task '{1}'")]
    UnknownDependency(String, String),
    #[error("Empty tasks list")]
    Empty,
}

/// A parsed task definition from `tasks.md`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub depends: Vec<String>,
    pub done_when: String,
    pub scenarios: Vec<String>,
    pub paths_scope: Vec<String>,
    pub test_command: Option<String>,
}

/// Task list with dependency analysis.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TaskList {
    pub tasks: Vec<Task>,
}

impl TaskList {
    /// Parse markdown text of `tasks.md` into a validated TaskList.
    pub fn parse(markdown: &str) -> Result<Self, TasksError> {
        let mut tasks = Vec::new();
        let mut current_task: Option<Task> = None;
        let mut in_scenarios = false;

        for (idx, line) in markdown.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("## ") {
                if let Some(task) = current_task.take() {
                    tasks.push(task);
                }
                in_scenarios = false;

                // Format: "## T-001: Name" or "## T-001 Name"
                let header = trimmed.trim_start_matches("## ").trim();
                let parts: Vec<&str> = header.splitn(2, ':').collect();
                if parts.is_empty() || parts[0].trim().is_empty() {
                    return Err(TasksError::Parse(line_num, "Invalid task header".to_string()));
                }
                let id = parts[0].trim().to_string();
                let title = if parts.len() > 1 {
                    parts[1].trim().to_string()
                } else {
                    String::new()
                };

                current_task = Some(Task {
                    id,
                    title,
                    depends: Vec::new(),
                    done_when: String::new(),
                    scenarios: Vec::new(),
                    paths_scope: Vec::new(),
                    test_command: None,
                });
                continue;
            }

            if let Some(ref mut task) = current_task {
                if trimmed.starts_with("- **depends:**") {
                    in_scenarios = false;
                    let val = trimmed.trim_start_matches("- **depends:**").trim();
                    if !val.is_empty() && val != "(пусто)" && val != "none" && val != "-" {
                        task.depends = val
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                } else if trimmed.starts_with("- **done_when:**") {
                    in_scenarios = false;
                    task.done_when = trimmed
                        .trim_start_matches("- **done_when:**")
                        .trim()
                        .to_string();
                } else if trimmed.starts_with("- **scenarios:**") {
                    in_scenarios = true;
                } else if trimmed.starts_with("- **paths_scope:**") {
                    in_scenarios = false;
                    let val = trimmed.trim_start_matches("- **paths_scope:**").trim();
                    if !val.is_empty() && val != "(пусто)" && val != "none" {
                        task.paths_scope = val
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                } else if trimmed.starts_with("- **test_command:**") {
                    in_scenarios = false;
                    let val = trimmed.trim_start_matches("- **test_command:**").trim();
                    if !val.is_empty() && val != "(пусто)" && val != "none" {
                        task.test_command = Some(val.to_string());
                    }
                } else if in_scenarios && (trimmed.starts_with("- ") || trimmed.starts_with("* ")) {
                    let scenario = trimmed[2..].trim().to_string();
                    if !scenario.is_empty() {
                        task.scenarios.push(scenario);
                    }
                }
            }
        }

        if let Some(task) = current_task {
            tasks.push(task);
        }

        if tasks.is_empty() {
            return Err(TasksError::Empty);
        }

        // Sort tasks by ID to ensure deterministic order (§5.4)
        tasks.sort_by(|a, b| a.id.cmp(&b.id));

        let list = TaskList { tasks };
        list.validate_dag()?;
        Ok(list)
    }

    /// Validate that dependencies form a valid Directed Acyclic Graph (DAG).
    pub fn validate_dag(&self) -> Result<(), TasksError> {
        let task_map: HashMap<&str, &Task> =
            self.tasks.iter().map(|t| (t.id.as_str(), t)).collect();

        // Check for missing dependency IDs
        for task in &self.tasks {
            for dep in &task.depends {
                if !task_map.contains_key(dep.as_str()) {
                    return Err(TasksError::UnknownDependency(dep.clone(), task.id.clone()));
                }
            }
        }

        // Cycle detection via DFS topological sort
        let mut visited = HashMap::new(); // 0 = unvisited, 1 = visiting, 2 = visited

        for task in &self.tasks {
            if *visited.get(task.id.as_str()).unwrap_or(&0) == 0 {
                let mut path = Vec::new();
                Self::dfs_cycle(&task.id, &task_map, &mut visited, &mut path)?;
            }
        }

        Ok(())
    }

    fn dfs_cycle<'a>(
        node: &'a str,
        task_map: &HashMap<&'a str, &'a Task>,
        visited: &mut HashMap<&'a str, u8>,
        path: &mut Vec<&'a str>,
    ) -> Result<(), TasksError> {
        visited.insert(node, 1); // in current DFS branch
        path.push(node);

        if let Some(task) = task_map.get(node) {
            for dep in &task.depends {
                let dep_str = dep.as_str();
                let status = *visited.get(dep_str).unwrap_or(&0);
                if status == 1 {
                    path.push(dep_str);
                    return Err(TasksError::CyclicDependency(path.join(" -> ")));
                } else if status == 0 {
                    Self::dfs_cycle(dep_str, task_map, visited, path)?;
                }
            }
        }

        path.pop();
        visited.insert(node, 2); // fully processed
        Ok(())
    }

    /// Return next task ready to be executed (§7.2).
    /// A task is ready if:
    /// 1. It is not already in `done`
    /// 2. All of its `depends` tasks are in `done`
    pub fn next_ready<'a>(&'a self, done: &[String]) -> Option<&'a Task> {
        let done_set: HashSet<&str> = done.iter().map(|s| s.as_str()).collect();

        for task in &self.tasks {
            if !done_set.contains(task.id.as_str()) {
                let all_deps_done = task
                    .depends
                    .iter()
                    .all(|dep| done_set.contains(dep.as_str()));
                if all_deps_done {
                    return Some(task);
                }
            }
        }
        None
    }

    /// Find task by ID.
    pub fn find(&self, task_id: &str) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == task_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tasks_parse_valid() {
        let md = r#"
## T-001: Init project
- **depends:** 
- **done_when:** cargo check passes
- **scenarios:**
  - Setup Cargo.toml
  - Create src/main.rs
- **paths_scope:** Cargo.toml, src/
- **test_command:** cargo test --lib

## T-002: Add feature
- **depends:** T-001
- **done_when:** ONE-check assert passes
- **scenarios:**
  - Verify feature flag
- **paths_scope:** src/feature.rs
- **test_command:**
"#;

        let list = TaskList::parse(md).unwrap();
        assert_eq!(list.tasks.len(), 2);
        assert_eq!(list.tasks[0].id, "T-001");
        assert_eq!(list.tasks[0].scenarios.len(), 2);
        assert_eq!(list.tasks[0].test_command.as_deref(), Some("cargo test --lib"));
        assert_eq!(list.tasks[1].depends, vec!["T-001"]);
        assert_eq!(list.tasks[1].test_command, None);

        // Next ready resolution
        let done = vec![];
        let ready1 = list.next_ready(&done).unwrap();
        assert_eq!(ready1.id, "T-001");

        let done = vec!["T-001".to_string()];
        let ready2 = list.next_ready(&done).unwrap();
        assert_eq!(ready2.id, "T-002");

        let done = vec!["T-001".to_string(), "T-002".to_string()];
        assert!(list.next_ready(&done).is_none());
    }

    #[test]
    fn test_tasks_cycle_detection() {
        let md = r#"
## T-001: Task 1
- **depends:** T-002
- **done_when:** ok

## T-002: Task 2
- **depends:** T-001
- **done_when:** ok
"#;
        let err = TaskList::parse(md).unwrap_err();
        match err {
            TasksError::CyclicDependency(_) => (),
            _ => panic!("Expected CyclicDependency error, got: {:?}", err),
        }
    }

    #[test]
    fn test_tasks_unknown_dep() {
        let md = r#"
## T-001: Task 1
- **depends:** T-999
- **done_when:** ok
"#;
        let err = TaskList::parse(md).unwrap_err();
        match err {
            TasksError::UnknownDependency(dep, task) => {
                assert_eq!(dep, "T-999");
                assert_eq!(task, "T-001");
            }
            _ => panic!("Expected UnknownDependency error"),
        }
    }
}
