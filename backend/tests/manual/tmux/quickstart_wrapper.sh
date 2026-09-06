#!/usr/bin/env bash
set -uo pipefail

config_dir="$1"
bin_path="$2"

env OMNESAGENT_CONFIG_DIR="$config_dir" "$bin_path" quickstart
status=$?
printf '\nEXIT_STATUS=%s\n' "$status"
sleep 5
