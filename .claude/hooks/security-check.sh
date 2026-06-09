#!/bin/bash

TOOL_INPUT=$(cat)
FILE_PATH=$(echo "$TOOL_INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null || echo "")

if [[ -z "$FILE_PATH" ]]; then
  exit 0
fi

SENSITIVE_PATTERNS=(
  "\.env$"
  "\.env\."
  "credentials"
  "secrets/"
  "\.ssh/"
  "id_rsa"
  "\.pem$"
  "\.key$"
  "\.git/config$"
)

for pattern in "${SENSITIVE_PATTERNS[@]}"; do
  if echo "$FILE_PATH" | grep -qE "$pattern"; then
    echo "BLOCKED: Cannot read sensitive file: $FILE_PATH"
    echo "This file may contain credentials or secrets."
    exit 2
  fi
done

exit 0
