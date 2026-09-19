// Comprehensive unit and regression tests for OmnesAgent Triage Router.

#[cfg(test)]
mod tests {
    use crate::agent::orchestrator::intent::{AgentIntent, ExecutionEngine};
    use crate::agent::orchestrator::heuristic::classify_message;
    use crate::agent::orchestrator::tool_group::filter_tools_for_intent;
    use crate::agent::orchestrator::profile::SessionExecutionProfile;
    use crate::agent::orchestrator::escalation::evaluate_escalation;
    use omnesagent_api::tool::ToolSpec;

    fn mock_tool(name: &str) -> ToolSpec {
        ToolSpec::new(name, "desc", serde_json::json!({}))
    }

    #[test]
    fn test_direct_chat_classification() {
        let greetings = ["Привет, как дела?", "Hello there!", "Объясни теорему Ферма", "Что такое monorepo?"];
        for msg in greetings {
            let decision = classify_message(msg);
            assert_eq!(
                decision.intent,
                AgentIntent::DirectChat,
                "Expected DirectChat for '{}', got {:?}",
                msg,
                decision.intent
            );
        }
    }

    #[test]
    fn test_code_exploration_classification() {
        let queries = [
            "Где находится класс Agent?",
            "Найди в коде реализацию stream_completion",
            "Покажи файл src/agent/loop_.rs",
            "Как устроен граф зависимостей?",
            "Where is the gateway handler defined?",
        ];
        for msg in queries {
            let decision = classify_message(msg);
            assert_eq!(
                decision.intent,
                AgentIntent::CodeExploration,
                "Expected CodeExploration for '{}', got {:?}",
                msg,
                decision.intent
            );
        }
    }

    #[test]
    fn test_engineering_task_standard_loop() {
        let code_tasks = [
            "Исправь опечатку в этой строке",
            "Добавь поле id в структуру User",
            "Отрефактори функцию parse_header",
        ];
        for msg in code_tasks {
            let decision = classify_message(msg);
            assert_eq!(
                decision.intent,
                AgentIntent::EngineeringTask,
                "Expected EngineeringTask for '{}'",
                msg
            );
            assert_eq!(decision.engine, ExecutionEngine::StandardLoop);
        }
    }

    #[test]
    fn test_engineering_task_ralph_autonomous() {
        let complex_tasks = [
            "Запусти ralph для полного цикла разработки фичи",
            "Автономно реализуй новый модуль с тестами",
            "Напиши тесты и проверь реализацию TDD",
        ];
        for msg in complex_tasks {
            let decision = classify_message(msg);
            assert_eq!(
                decision.intent,
                AgentIntent::EngineeringTask,
                "Expected EngineeringTask for '{}'",
                msg
            );
            assert_eq!(
                decision.engine,
                ExecutionEngine::RalphAutonomous,
                "Expected RalphAutonomous for '{}'",
                msg
            );
        }
    }

    #[test]
    fn test_system_admin_classification() {
        let devops = [
            "Перезапусти контейнер в docker",
            "Покажи логи docker-compose",
            "Настрой cron job на VPS",
            "Проверь статус через systemctl",
        ];
        for msg in devops {
            let decision = classify_message(msg);
            assert_eq!(
                decision.intent,
                AgentIntent::SystemAdmin,
                "Expected SystemAdmin for '{}'",
                msg
            );
        }
    }

    #[test]
    fn test_ui_flags_in_profiles() {
        let chat_profile = SessionExecutionProfile::new(AgentIntent::DirectChat, ExecutionEngine::StandardLoop, vec![]);
        assert!(!chat_profile.ui_flags.show_todo_widget);
        assert!(!chat_profile.ui_flags.show_ralph_progress);
        assert_eq!(chat_profile.ui_flags.display_mode, "chat");

        let task_profile = SessionExecutionProfile::new(AgentIntent::EngineeringTask, ExecutionEngine::StandardLoop, vec![]);
        assert!(task_profile.ui_flags.show_todo_widget);
        assert_eq!(task_profile.ui_flags.display_mode, "task");

        let ralph_profile = SessionExecutionProfile::new(AgentIntent::EngineeringTask, ExecutionEngine::RalphAutonomous, vec![]);
        assert!(ralph_profile.ui_flags.show_todo_widget);
        assert!(ralph_profile.ui_flags.show_ralph_progress);
        assert_eq!(ralph_profile.ui_flags.display_mode, "ralph");
    }

    #[test]
    fn test_tool_pruning_direct_chat_has_zero_tools() {
        let inventory = vec![
            mock_tool("read_file"),
            mock_tool("write_to_file"),
            mock_tool("shell"),
        ];
        let chat_tools = filter_tools_for_intent(&inventory, AgentIntent::DirectChat);
        assert!(chat_tools.is_empty(), "DirectChat must have strictly 0 tools");
    }

    #[test]
    fn test_escalation_chat_to_engineering() {
        let inventory = vec![mock_tool("read_file"), mock_tool("write_to_file")];
        let profile = SessionExecutionProfile::new(AgentIntent::DirectChat, ExecutionEngine::StandardLoop, vec![]);

        // User asks to edit code
        let escalated = evaluate_escalation(&profile, "Теперь исправь ошибку в файле", &inventory, 0);
        assert_eq!(escalated.intent, AgentIntent::EngineeringTask);
        assert!(escalated.ui_flags.show_todo_widget);
        assert_eq!(escalated.allowed_tools.len(), 2);
    }
}
