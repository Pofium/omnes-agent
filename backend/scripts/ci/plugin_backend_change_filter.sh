#!/usr/bin/env bash

# Classifies changed paths for the plugin backend CI job.
#
# Reads one changed path per line on stdin and prints "true" when any path
# affects the plugin backends or the feature-gated runtime coverage they
# carry (the live-config plugin regression compiles omnesagent-runtime with
# plugins-wasm-cranelift, so runtime-only changes must run the job too).
# omnesagent-config is listed for the same reason: it is the canonical home of
# the operator-facing plugin config surface that omnesagent-plugins compiles
# against, so a change there can break this job while nothing under
# crates/omnesagent-plugins moves.
# The root-package channel activation e2e is listed individually because it is
# the one piece of this job's coverage that lives outside a crate directory:
# it drives omnesagent-runtime, which omnesagent-plugins cannot depend on without
# inverting the crate graph, so it has to be a root `omnesagent` test target.
# Prints "false" otherwise. Always exits 0; the workflow step forwards the
# printed value to GITHUB_OUTPUT.

set -euo pipefail

run=false

while IFS= read -r path; do
    case "$path" in
        crates/omnesagent-plugins/*|\
        crates/omnesagent-runtime/*|\
        crates/omnesagent-config/*|\
        tests/plugin_channel_runtime_e2e.rs|\
        wit/*|\
        Cargo.toml|Cargo.lock|\
        .github/workflows/ci.yml|\
        scripts/ci/plugin_backend_change_filter*.sh)
            run=true
            ;;
    esac
done

echo "$run"
