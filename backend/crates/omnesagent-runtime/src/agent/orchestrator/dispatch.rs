// Dispatch seam: translate a triaged [`SessionExecutionProfile`] into an
// explicit engine route.
//
// The Triage Router owns the *routing* decision (which engine should run a
// turn); the concrete engines live in their own layers — the interactive agent
// loop (`omnesagent-runtime::agent::loop_`) or the autonomous Ralph engine
// (`omnesagent-ralph::RalphOrchestrator`). Expressing the decision as plain
// data keeps this crate free of an `omnesagent-runtime -> omnesagent-ralph`
// dependency (layering) while giving the binary/gateway a single, typed switch
// point via [`resolve_engine_route`].

use serde::{Deserialize, Serialize};

use super::intent::ExecutionEngine;
use super::profile::SessionExecutionProfile;

/// Longest feature slug produced by [`derive_slug`]; keeps the OpenSpec
/// `openspec/changes/<slug>/` path short and filesystem-safe.
const MAX_SLUG_LEN: usize = 48;

/// Work package handed to the autonomous Ralph engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RalphDispatch {
    /// Feature slug naming the OpenSpec workspace / run.
    pub slug: String,
    /// The original task text Ralph must plan and execute.
    pub task: String,
}

/// Engine selected for a triaged turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineRoute {
    /// Interactive agent loop — the default, low-overhead path.
    StandardLoop,
    /// Autonomous Ralph engine — multi-file / TDD / self-healing work.
    Ralph(RalphDispatch),
}

/// Resolve the execution route for a triaged profile and its triggering message.
#[must_use]
pub fn resolve_engine_route(profile: &SessionExecutionProfile, message: &str) -> EngineRoute {
    match profile.engine {
        ExecutionEngine::StandardLoop => EngineRoute::StandardLoop,
        ExecutionEngine::RalphAutonomous => EngineRoute::Ralph(RalphDispatch {
            slug: derive_slug(message),
            task: message.trim().to_string(),
        }),
    }
}

/// Derive a deterministic, filesystem-safe OpenSpec feature slug from a task
/// message: ASCII lowercase alphanumerics joined by single `-`. Non-ASCII
/// messages (e.g. Cyrillic) collapse to a stable `task-<hash>` fallback so two
/// distinct tasks never share a slug.
#[must_use]
pub fn derive_slug(message: &str) -> String {
    let mut slug = String::with_capacity(message.len());
    let mut prev_dash = false;
    for ch in message.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            slug.push(c);
            prev_dash = false;
        } else if !prev_dash && !slug.is_empty() {
            slug.push('-');
            prev_dash = true;
        }
        if slug.len() >= MAX_SLUG_LEN {
            break;
        }
    }

    let trimmed = slug.trim_matches('-');
    if trimmed.is_empty() {
        format!("task-{:08x}", fnv1a(message.as_bytes()))
    } else {
        trimmed.to_string()
    }
}

/// FNV-1a 32-bit hash — small, dependency-free, and deterministic across builds
/// (unlike `DefaultHasher`).
fn fnv1a(bytes: &[u8]) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for &b in bytes {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::orchestrator::intent::AgentIntent;

    fn profile(engine: ExecutionEngine) -> SessionExecutionProfile {
        SessionExecutionProfile::new(AgentIntent::EngineeringTask, engine, vec![])
    }

    #[test]
    fn standard_engine_maps_to_standard_loop() {
        let route = resolve_engine_route(&profile(ExecutionEngine::StandardLoop), "fix the typo");
        assert_eq!(route, EngineRoute::StandardLoop);
    }

    #[test]
    fn ralph_engine_carries_slug_and_trimmed_task() {
        let route = resolve_engine_route(
            &profile(ExecutionEngine::RalphAutonomous),
            "  Implement the parser and add tests  ",
        );
        match route {
            EngineRoute::Ralph(dispatch) => {
                assert_eq!(dispatch.task, "Implement the parser and add tests");
                assert_eq!(dispatch.slug, "implement-the-parser-and-add-tests");
            }
            other => panic!("expected Ralph route, got {other:?}"),
        }
    }

    #[test]
    fn slug_is_filesystem_safe_and_bounded() {
        let slug = derive_slug("Refactor: src/a.rs, src/b.rs & run tests!");
        assert!(!slug.contains('/'));
        assert!(!slug.contains(' '));
        assert!(slug.len() <= MAX_SLUG_LEN);
        assert!(!slug.starts_with('-') && !slug.ends_with('-'));
    }

    #[test]
    fn non_ascii_message_gets_stable_hash_slug() {
        let first = derive_slug("Автономно реализуй модуль памяти");
        let repeat = derive_slug("Автономно реализуй модуль памяти");
        let other = derive_slug("Совсем другая задача");
        assert!(first.starts_with("task-"));
        assert_eq!(first, repeat);
        assert_ne!(first, other);
    }
}
