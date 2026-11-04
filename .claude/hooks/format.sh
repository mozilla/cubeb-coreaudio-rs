#!/bin/bash
set -e

cd "$CLAUDE_PROJECT_DIR" || exit 1

TOOL_INPUT=$(cat)
FILE_PATH=$(echo "$TOOL_INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null || echo "")

if [[ -z "$FILE_PATH" || ! "$FILE_PATH" =~ \.rs$ ]]; then
  exit 0
fi

echo "Running cargo fmt..."
cargo fmt --all 2>&1 || {
  echo "Warning: cargo fmt failed"
}

exit 0
