// Zero-latency deterministic heuristic classifier for OmnesAgent Triage Router.

use super::intent::{AgentIntent, ExecutionEngine};

/// Result of heuristic classification.
#[derive(Debug, Clone, PartialEq)]
pub struct HeuristicDecision {
    pub intent: AgentIntent,
    pub engine: ExecutionEngine,
    pub confidence: f32,
    pub matched_signals: Vec<&'static str>,
}

/// Classify a user query using deterministic pattern matching.
pub fn classify_message(message: &str) -> HeuristicDecision {
    let lower = message.to_lowercase();
    let trimmed = message.trim();
    let len = trimmed.len();

    let mut signals = Vec::new();

    // 1. DevOps / System Admin check
    if has_devops_signal(&lower, &mut signals) {
        return HeuristicDecision {
            intent: AgentIntent::SystemAdmin,
            engine: ExecutionEngine::StandardLoop,
            confidence: 0.95,
            matched_signals: signals,
        };
    }

    // 2. Explicit Ralph / Autonomous development check
    if lower.contains("ralph") || lower.contains("автономно") || lower.contains("tdd") {
        signals.push("ralph_autonomous_direct");
        return HeuristicDecision {
            intent: AgentIntent::EngineeringTask,
            engine: ExecutionEngine::RalphAutonomous,
            confidence: 0.98,
            matched_signals: signals,
        };
    }

    // 3. Engineering Modification check (verbs, code fences, diffs)
    let has_modifying_verb = has_engineering_verb(&lower, &mut signals);
    let has_code_fence = message.contains("```");
    if has_code_fence {
        signals.push("code_fence");
    }

    let mentions_file_path = has_file_path_reference(&lower, &mut signals);

    if has_modifying_verb || (has_code_fence && len > 60) {
        // Evaluate task scale / complexity to choose between StandardLoop and RalphAutonomous
        let is_complex_task = is_complex_engineering_task(&lower, len, has_code_fence);
        let engine = if is_complex_task {
            signals.push("ralph_complex_task");
            ExecutionEngine::RalphAutonomous
        } else {
            ExecutionEngine::StandardLoop
        };

        return HeuristicDecision {
            intent: AgentIntent::EngineeringTask,
            engine,
            confidence: 0.90,
            matched_signals: signals,
        };
    }

    // 3. Code Exploration check (read-only search, find, explain architecture)
    if has_exploration_signal(&lower, mentions_file_path, &mut signals) {
        return HeuristicDecision {
            intent: AgentIntent::CodeExploration,
            engine: ExecutionEngine::StandardLoop,
            confidence: 0.88,
            matched_signals: signals,
        };
    }

    // 4. Fallback to DirectChat (pure conversation, explanations, questions)
    signals.push("direct_chat_default");
    HeuristicDecision {
        intent: AgentIntent::DirectChat,
        engine: ExecutionEngine::StandardLoop,
        confidence: 0.92,
        matched_signals: signals,
    }
}

fn has_devops_signal(lower: &str, signals: &mut Vec<&'static str>) -> bool {
    const DEVOPS_KEYWORDS: &[(&str, &'static str)] = &[
        ("docker", "devops_docker"),
        ("docker-compose", "devops_compose"),
        ("dockerfile", "devops_dockerfile"),
        ("kubernetes", "devops_k8s"),
        ("kubectl", "devops_kubectl"),
        ("systemctl", "devops_systemctl"),
        ("systemd", "devops_systemd"),
        ("ssh ", "devops_ssh"),
        ("crontab", "devops_crontab"),
        ("cron job", "devops_cron"),
        ("vps", "devops_vps"),
        ("nginx", "devops_nginx"),
        ("caddy", "devops_caddy"),
    ];

    let mut hit = false;
    for (kw, tag) in DEVOPS_KEYWORDS {
        if lower.contains(kw) {
            signals.push(tag);
            hit = true;
        }
    }
    hit
}

fn has_engineering_verb(lower: &str, signals: &mut Vec<&'static str>) -> bool {
    const MODIFY_VERBS: &[(&str, &'static str)] = &[
        // Russian verbs
        ("создай", "verb_create_ru"),
        ("напиши код", "verb_write_code_ru"),
        ("напиши функцию", "verb_write_fn_ru"),
        ("напиши тест", "verb_write_test_ru"),
        ("исправь", "verb_fix_ru"),
        ("перепиши", "verb_rewrite_ru"),
        ("измени", "verb_modify_ru"),
        ("добавь", "verb_add_ru"),
        ("отрефактори", "verb_refactor_ru"),
        ("протестируй", "verb_test_ru"),
        ("запусти тест", "verb_run_test_ru"),
        ("почини", "verb_repair_ru"),
        ("закоммить", "verb_commit_ru"),
        ("запушь", "verb_push_ru"),
        ("сделай фичу", "verb_feature_ru"),
        ("реализуй", "verb_implement_ru"),
        ("удали файл", "verb_delete_file_ru"),
        // English verbs
        ("create file", "verb_create_en"),
        ("implement", "verb_implement_en"),
        ("refactor", "verb_refactor_en"),
        ("fix bug", "verb_fix_en"),
        ("fix error", "verb_fix_error_en"),
        ("rewrite", "verb_rewrite_en"),
        ("write test", "verb_write_test_en"),
        ("run test", "verb_run_test_en"),
        ("git commit", "verb_git_commit"),
    ];

    let mut hit = false;
    for (verb, tag) in MODIFY_VERBS {
        if lower.contains(verb) {
            signals.push(tag);
            hit = true;
        }
    }
    hit
}

fn has_exploration_signal(lower: &str, mentions_file: bool, signals: &mut Vec<&'static str>) -> bool {
    const EXPLORE_KEYWORDS: &[(&str, &'static str)] = &[
        ("где находится", "explore_where_ru"),
        ("где лежит", "explore_where_ru"),
        ("где объявлен", "explore_where_ru"),
        ("найди в коде", "explore_find_ru"),
        ("покажи файл", "explore_show_file_ru"),
        ("как устроен", "explore_how_works_ru"),
        ("как работает модуль", "explore_module_ru"),
        ("архитектур", "explore_arch_ru"),
        ("граф зависимостей", "explore_deps_ru"),
        ("where is", "explore_where_en"),
        ("find in codebase", "explore_find_en"),
        ("show me the file", "explore_show_en"),
        ("explain code", "explore_explain_en"),
        ("codebase architecture", "explore_arch_en"),
    ];

    let mut hit = false;
    for (kw, tag) in EXPLORE_KEYWORDS {
        if lower.contains(kw) {
            signals.push(tag);
            hit = true;
        }
    }

    if mentions_file && (lower.contains("найди") || lower.contains("покажи") || lower.contains("show") || lower.contains("find")) {
        signals.push("file_inspection");
        hit = true;
    }

    hit
}

fn has_file_path_reference(lower: &str, signals: &mut Vec<&'static str>) -> bool {
    const EXTENSIONS: &[(&str, &'static str)] = &[
        (".rs", "ext_rs"),
        (".dart", "ext_dart"),
        (".ts", "ext_ts"),
        (".js", "ext_js"),
        (".py", "ext_py"),
        (".json", "ext_json"),
        (".toml", "ext_toml"),
        (".yaml", "ext_yaml"),
        (".yml", "ext_yml"),
    ];

    let mut hit = false;
    for (ext, tag) in EXTENSIONS {
        if lower.contains(ext) {
            signals.push(tag);
            hit = true;
        }
    }
    if lower.contains("src/") || lower.contains("lib/") || lower.contains("crates/") {
        signals.push("path_prefix");
        hit = true;
    }
    hit
}

fn is_complex_engineering_task(lower: &str, len: usize, has_code_fence: bool) -> bool {
    // Ralph is invoked for comprehensive tasks: test-driven implementation,
    // autonomous multi-step refactoring, or explicit autonomous commands.
    if lower.contains("ralph")
        || lower.contains("автономно")
        || lower.contains("полный цикл")
        || lower.contains("напиши тесты и проверь")
        || lower.contains("tdd")
        || lower.contains("self-heal")
    {
        return true;
    }

    // Long multi-prompt engineering task
    if len > 350 && has_code_fence && (lower.contains("тест") || lower.contains("test")) {
        return true;
    }

    false
}
