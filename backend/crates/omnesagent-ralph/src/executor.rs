//! Executor abstractions, CLI adapters, timeout management, and answer parsing.
//!
//! Conforms to §6, §7.2, FR-A4, FR-A14, Appendix C of specification.

use crate::journal::ExecutionResult;
use crate::minimality::DeleteList;
use crate::tasks::Task;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use thiserror::Error;
use tokio::process::Command;

#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error("Executor timed out after {0:?}")]
    Timeout(Duration),
    #[error("Executor process failed with exit code {0:?}: {1}")]
    ProcessFailed(Option<i32>, String),
    #[error("Failed to parse ralph block from executor output: {0}")]
    Parse(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Context provided to the executor for a given task turn.
#[derive(Debug, Clone)]
pub struct TaskContext<'a> {
    pub task: &'a Task,
    pub spec_context: &'a str,
    pub knowledge_context: &'a str,
    pub paths_scope: &'a [String],
    pub mode: &'a str,
}

/// Structured answer parsed from the executor's output block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutorAnswer {
    pub task_id: String,
    pub hypothesis: String,
    pub plan: Vec<String>,
    pub result: ExecutionResult,
    pub ladder_rung: String,
    #[serde(default)]
    pub markers: Vec<String>,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub raw_output: String,
}

/// Canonical Ponytail contract injected into every executor prompt (Appendix C).
pub const PONYTAIL_CONTRACT_PROMPT: &str = r#"
[Правила минимальности — лестница, останавливайся на первой верной ступени
ПОСЛЕ того, как понял задачу и прочитал код, который меняешь:]
1. Это вообще нужно строить? нет → не строй (YAGNI)
2. Уже есть в этой кодовой базе? переиспользуй, не переписывай
3. Стандартная библиотека умеет? используй её
4. Нативная возможность платформы покрывает? используй
5. Уже установленная зависимость решает? используй
6. Можно одной строкой? одной строкой
7. Только иначе: минимальный работающий код.

[Правила:]
- Root cause, не симптом: фикс делай в общей точке, проверь всех вызывающих.
- Никаких абстракций/зависимостей/boilerplate, которых не просили.
- Удаление лучше добавления; скучное лучше хитрого; меньше файлов.
- Осознанное упрощение с потолком помечай комментарием:
  `ponytail: <потолок>, <триггер пересмотра>`.
[Не отсекаемое никогда:] понимание задачи (трассируй поток целиком), валидация
на границах доверия, обработка потери данных, безопасность, доступность,
калибровка под реальное железо, всё явно запрошенное спекой.
[Проверка:] нетривиальная логика оставляет ОДНУ запускаемую проверку
(самопроверка/assert или маленький тест-файл; без фреймворков и фикстур).

[Формат ответа — последним блоком:
```ralph
{
  "task_id": "T-XXX",
  "hypothesis": "...",
  "plan": ["..."],
  "result": {
    "what_done": "...",
    "errors": [],
    "self_assessment": "ok"
  },
  "ladder_rung": "reuse: ...",
  "markers": ["..."]
}
```
]
[Запреты:] git commit/push, правки openspec/*, выход за paths_scope.
[Правила действуют и для любых подагентов, которых ты создашь.]
"#;

#[async_trait]
pub trait Executor: Send + Sync {
    /// Execute one task iteration.
    async fn run_task(&self, ctx: &TaskContext<'_>) -> Result<ExecutorAnswer, ExecutorError>;

    /// Conduct minimality review on the iteration git diff (§7.2.1).
    async fn review_delete_list(&self, diff: &str) -> Result<DeleteList, ExecutorError>;
}

/// Helper to parse the ```ralph { ... }``` JSON block from model response text.
pub fn parse_ralph_block(output: &str) -> Result<ExecutorAnswer, ExecutorError> {
    let marker = "```ralph";
    let start = output
        .rfind(marker)
        .ok_or_else(|| ExecutorError::Parse("Missing ```ralph code block".to_string()))?;

    let after_marker = &output[start + marker.len()..];
    let end = after_marker
        .find("```")
        .ok_or_else(|| ExecutorError::Parse("Unclosed ```ralph code block".to_string()))?;

    let json_text = after_marker[..end].trim();
    let mut answer: ExecutorAnswer = serde_json::from_str(json_text)
        .map_err(|e| ExecutorError::Parse(format!("Invalid JSON in ```ralph block: {}", e)))?;

    answer.raw_output = output.to_string();
    if answer.summary.is_empty() {
        answer.summary = answer
            .result
            .what_done
            .lines()
            .next()
            .unwrap_or("task completed")
            .to_string();
    }

    Ok(answer)
}

/// CLI Subprocess Executor (claude -p, hermes -z, or custom command).
pub struct SubprocessCliExecutor {
    pub command_template: String,
    pub timeout: Duration,
    pub working_dir: PathBuf,
}

impl SubprocessCliExecutor {
    pub fn new(command_template: String, timeout_secs: u64, working_dir: PathBuf) -> Self {
        Self {
            command_template,
            timeout: Duration::from_secs(timeout_secs),
            working_dir,
        }
    }

    /// Build full prompt string including task context, spec, knowledge, and Ponytail rules.
    pub fn build_prompt(&self, ctx: &TaskContext<'_>) -> String {
        format!(
            "# TASK: {} - {}\n\n## Done When:\n{}\n\n## Scenarios:\n{}\n\n## Paths Scope:\n{}\n\n## Spec Context:\n{}\n\n## Knowledge Context:\n{}\n\n{}",
            ctx.task.id,
            ctx.task.title,
            ctx.task.done_when,
            ctx.task.scenarios.join("\n- "),
            ctx.paths_scope.join(", "),
            ctx.spec_context,
            ctx.knowledge_context,
            PONYTAIL_CONTRACT_PROMPT
        )
    }
}

#[async_trait]
impl Executor for SubprocessCliExecutor {
    async fn run_task(&self, ctx: &TaskContext<'_>) -> Result<ExecutorAnswer, ExecutorError> {
        let prompt = self.build_prompt(ctx);
        let prompt_file = self
            .working_dir
            .join(format!(".ralph_prompt_{}.txt", ctx.task.id));
        tokio::fs::write(&prompt_file, &prompt).await?;

        let cmd_str = if self.command_template.contains("{prompt_file}") {
            self.command_template
                .replace("{prompt_file}", &prompt_file.to_string_lossy())
        } else if self.command_template == "claude" {
            format!("claude -p \"{}\"", prompt_file.to_string_lossy())
        } else if self.command_template == "hermes" {
            format!("hermes -z \"{}\"", prompt_file.to_string_lossy())
        } else {
            format!("{} \"{}\"", self.command_template, prompt_file.to_string_lossy())
        };

        let child = if cfg!(windows) {
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", &cmd_str])
                .current_dir(&self.working_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);
            cmd.spawn()?
        } else {
            let mut cmd = Command::new("sh");
            cmd.args(["-c", &cmd_str])
                .current_dir(&self.working_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);
            cmd.spawn()?
        };

        let result = match tokio::time::timeout(self.timeout, child.wait_with_output()).await {
            Ok(wait_res) => wait_res?,
            Err(_) => {
                let _ = tokio::fs::remove_file(&prompt_file).await;
                return Err(ExecutorError::Timeout(self.timeout));
            }
        };

        let _ = tokio::fs::remove_file(&prompt_file).await;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(ExecutorError::ProcessFailed(
                result.status.code(),
                stderr.to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&result.stdout);
        parse_ralph_block(&stdout)
    }

    async fn review_delete_list(&self, _diff: &str) -> Result<DeleteList, ExecutorError> {
        // Lite/default review returns empty delete list if no deletions identified
        Ok(DeleteList::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ralph_block() {
        let sample = r#"
Here is what I did:
I added the requested feature.

```ralph
{
  "task_id": "T-001",
  "hypothesis": "Fix off-by-one error",
  "plan": ["Change loop bound"],
  "result": {
    "what_done": "Changed i <= len to i < len",
    "errors": [],
    "self_assessment": "ok"
  },
  "ladder_rung": "one line",
  "markers": ["ponytail: ceiling 10, trigger on scale"]
}
```
All finished!
"#;

        let answer = parse_ralph_block(sample).unwrap();
        assert_eq!(answer.task_id, "T-001");
        assert_eq!(answer.hypothesis, "Fix off-by-one error");
        assert_eq!(answer.ladder_rung, "one line");
        assert_eq!(answer.markers.len(), 1);
        assert_eq!(answer.summary, "Changed i <= len to i < len");
    }
}
