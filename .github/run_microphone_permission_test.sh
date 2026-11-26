#!/bin/bash
set -euo pipefail

export MACOSX_DEPLOYMENT_TARGET=10.15

# Reuse the normal test harness but launch it as its own TCC client.
test_binary=$(cargo test --lib --no-run --message-format=json |
  jq -er 'select(.reason == "compiler-artifact" and .profile.test and .executable != null) | .executable')
test_dir=$(mktemp -d "${RUNNER_TEMP}/cubeb-microphone.XXXXXX")
bundle_id="org.mozilla.cubeb-coreaudio.permission-test.$(uuidgen)"
app="$test_dir/CubebMicrophoneTest.app"
tcc_db="$HOME/Library/Application Support/com.apple.TCC/TCC.db"
service=kTCCServiceMicrophone

mkdir -p "$app/Contents/MacOS"
cp "$test_binary" "$app/Contents/MacOS/CubebMicrophoneTest"
cat > "$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>CubebMicrophoneTest</string>
  <key>CFBundleIdentifier</key>
  <string>$bundle_id</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>LSBackgroundOnly</key>
  <true/>
  <key>NSMicrophoneUsageDescription</key>
  <string>Test the cubeb response to denied microphone access.</string>
</dict>
</plist>
EOF

codesign --force --sign - "$app"
codesign -d -r- "$app" 2>&1 | sed -n 's/^# designated => //p; s/^designated => //p' |
  csreq -r- -b "$test_dir/requirement"
requirement=$(xxd -p "$test_dir/requirement" | tr -d '\n')
test -n "$requirement"

# Seed a denial for only this app. A fresh bundle ID avoids cached decisions
# and leaves the runner's grant available for the regular capture tests.
sqlite3 -bail -cmd '.timeout 5000' "$tcc_db" "
  INSERT INTO access (
    service, client, client_type, auth_value, auth_reason, auth_version,
    csreq, indirect_object_identifier, flags, last_modified
  ) VALUES ('$service', '$bundle_id', 0, 0, 2, 1,
    X'$requirement', 'UNUSED', 0, unixepoch());
"
cleanup() {
  sqlite3 -bail -cmd '.timeout 5000' "$tcc_db" \
    "DELETE FROM access WHERE service = '$service' AND client = '$bundle_id' AND client_type = 0;"
}
trap cleanup EXIT

log="$test_dir/test.log"
errors="$test_dir/test.stderr"
touch "$log" "$errors"
open -n -W -g --stdout "$log" --stderr "$errors" "$app" --args \
  --ignored --exact backend::tests::interfaces::test_ops_context_stream_init_permission_denied
cat "$log" "$errors"
# open waits for termination but does not report the test process's exit code.
grep -Fq 'test result: ok. 1 passed; 0 failed;' "$log"
