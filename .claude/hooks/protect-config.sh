#!/bin/bash

TOOL_INPUT=$(cat)
FILE_PATH=$(echo "$TOOL_INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null || echo "")

if [[ -z "$FILE_PATH" ]]; then
  exit 0
fi

PROTECTED_PATTERNS=(
  "\.claude/settings\.json$"
  "\.claude/settings\.local\.json$"
)

for pattern in "${PROTECTED_PATTERNS[@]}"; do
  if echo "$FILE_PATH" | grep -qE "$pattern"; then
    echo "BLOCKED: Cannot modify protected configuration: $FILE_PATH"
    echo "Claude Code settings should be modified manually by developers."
    echo "This ensures intentional, reviewed changes to team-shared configuration."
    exit 2
  fi
done

exit 0
