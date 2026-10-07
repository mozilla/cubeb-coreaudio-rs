---
name: github-issue-resolver
description: Fetches and resolves GitHub issues from mozilla/cubeb-coreaudio-rs repository. Analyzes problems, proposes multiple solutions for approval, implements the chosen solution, and runs tests automatically. Use when working on GitHub issues, fixing bugs, or implementing features from the issue tracker.
allowed-tools: Read, Grep, Glob, Bash, WebFetch
---

# GitHub Issue Resolver for cubeb-coreaudio-rs

Automates the complete workflow for resolving GitHub issues in the cubeb-coreaudio-rs repository.

## Workflow

### Step 1: Fetch Issue from GitHub

When the user provides an issue number, fetch it from the GitHub API:

**API Endpoint:**
```
https://api.github.com/repos/mozilla/cubeb-coreaudio-rs/issues/{number}
```

**Extract:**
- Issue title
- Full description
- Labels (bug, enhancement, etc.)
- Current state (open/closed)
- Comments (if relevant)

**Display the issue clearly to the user.**

### Step 2: Analyze the Problem

After fetching the issue:

1. **Understand the context:**
   - Read relevant source files using Glob and Read tools
   - Search for related code using Grep
   - Check test files for existing coverage
   - Review CLAUDE.md and CLAUDE.local.md for project conventions

2. **Identify root cause:**
   - Analyze error messages
   - Review stack traces
   - Check related issues or PRs
   - Consider architectural constraints

### Step 3: Propose Solutions

Generate 2-3 potential solutions with:

**For each solution provide:**
- **Summary:** Brief one-line description
- **Approach:** How it works technically
- **Pros:** Advantages of this approach
- **Cons:** Potential drawbacks or limitations
- **Complexity:** Low/Medium/High
- **Files affected:** Which files need changes
- **Test strategy:** What tests are needed

**Present all solutions clearly and wait for user approval.**

**IMPORTANT:** Do NOT implement anything until the user explicitly approves a solution.

### Step 4: Get User Approval

Wait for explicit approval from the user:

Examples of approval:
- "Approve solution #1"
- "Go with solution #2"
- "I like the third approach"
- "Implement solution 1"

If the user asks questions or wants modifications, discuss those before proceeding.

### Step 5: Implement the Approved Solution

Once approved:

1. **Make code changes:**
   - Read affected files first
   - Make targeted, focused changes
   - Follow project conventions from CLAUDE.md (minimal comments, clear naming)
   - Keep changes minimal and directly related to the issue

2. **Verify syntax:**
   - Check Rust syntax if needed
   - Ensure no obvious compilation errors

3. **Stage changes:**
   - Use git add for modified files
   - Prepare for commit

### Step 6: Run Tests Automatically

Execute the test suite to validate changes:

**Run appropriate tests based on the changes:**

```bash
# Standard test suite (always run this first)
sh run_tests.sh

# If device-specific changes were made:
# sh run_device_tests.sh

# For specific test:
# cargo test <test_name> -- --ignored --nocapture
```

**Report test results:**
- Format check status (cargo fmt)
- Clippy lint results
- Test pass/fail status
- Any errors or warnings

**If tests fail:**
- Analyze the failure
- Propose fixes
- Get approval before re-implementing
- Re-run tests after fixes

### Step 7: Commit (Optional)

If user requests it, create a commit:

```bash
git commit -m "Fix #<issue_number>: <brief description>

<More detailed explanation if needed>

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

## Example Usage

```
User: "Work on issue #456"

Claude: [Fetches issue using WebFetch]
Claude: "Issue #456: Audio glitches on M1 Macs with high sample rates

Description: When using 96kHz sample rate on M1 MacBook Pro,
audio output has periodic glitches every 2-3 seconds..."

Claude: [Analyzes codebase]
Claude: "I've analyzed the issue and found 3 potential solutions:

**Solution 1: Increase Buffer Size**
- Approach: Adjust minimum buffer size for high sample rates
- Pros: Simple, low risk
- Cons: Increases latency
- Complexity: Low
- Files: src/backend/mod.rs (AudioUnitStream::setup)
- Tests: test_high_sample_rate

**Solution 2: Adaptive Buffer Management**
- Approach: Dynamically adjust based on sample rate
- Pros: Optimal performance, no latency impact
- Cons: More complex logic
- Complexity: Medium
- Files: src/backend/mod.rs, src/buffer_manager.rs
- Tests: test_adaptive_buffers, test_high_sample_rate

**Solution 3: CoreAudio Property Adjustment**
- Approach: Set kAudioDevicePropertyBufferFrameSize earlier
- Pros: Addresses root cause
- Cons: May conflict with other streams
- Complexity: Medium
- Files: src/backend/mod.rs
- Tests: test_buffer_property_timing

Which solution would you like me to implement?"

User: "Go with solution 2"

Claude: [Implements adaptive buffer management]
Claude: "Changes implemented. Running tests..."
Claude: [Runs sh run_tests.sh]
Claude: "✅ All tests passed!
- Format check: PASSED
- Clippy: PASSED
- Tests: 156 passed, 0 failed

The implementation successfully resolves issue #456."
```

## Project-Specific Context

### Repository
- **GitHub:** https://github.com/mozilla/cubeb-coreaudio-rs/
- **Owner:** mozilla
- **Repo:** cubeb-coreaudio-rs

### Testing Commands
```bash
# Full test suite (use this by default)
sh run_tests.sh

# Device tests (changes system audio - use only if needed)
sh run_device_tests.sh

# Single test
cargo test <test_name>

# Ignored test (interactive/manual)
cargo test <test_name> -- --ignored --nocapture

# Timing-sensitive tests (must be serial)
cargo test test_ops_timing_sensitive -- --ignored --test-threads=1
```

### Code Style (from CLAUDE.md)
- Minimize comments strictly
- Only comment on non-trivial logic or unclear parameters
- Prioritize clear naming over comments
- Never restate what code does
- Refactor unclear code instead of explaining it

### Key Files
- `src/backend/mod.rs` - Main stream implementation (~6000+ lines)
- `src/capi.rs` - C API entry point
- `src/device_property.rs` - Device property queries
- `src/aggregate_device.rs` - Aggregate device management
- `src/buffer_manager.rs` - Audio buffer management
- `src/mixer.rs` - Channel mixing
- `src/resampler.rs` - Sample rate conversion

## Important Guidelines

1. **Always fetch the issue first** - Don't assume you know what it's about
2. **Analyze thoroughly** - Read related code before proposing solutions
3. **Multiple solutions** - Always propose 2-3 options when possible
4. **Wait for approval** - Never implement without explicit user consent
5. **Run tests automatically** - Always validate changes with the test suite
6. **Follow conventions** - Respect project style from CLAUDE.md
7. **Focused changes** - Keep modifications minimal and targeted
8. **Report clearly** - Show test results and any issues

## Handling Edge Cases

**If issue number is not provided:**
```
"Which issue number would you like to work on?"
```

**If issue is already closed:**
```
"Note: This issue is marked as closed. Would you still like to work on it?"
```

**If tests fail after implementation:**
```
"Tests failed. Here's the analysis:
[Show failures]

Would you like me to:
1. Fix the test failures
2. Revert and try a different approach
3. Investigate further"
```

**If the issue is unclear:**
```
"The issue description is unclear. I have some questions:
[Ask clarifying questions]"
```

## Success Criteria

A successful resolution includes:
✅ Issue fetched and displayed
✅ Problem analyzed with codebase context
✅ Multiple solutions proposed
✅ User approval obtained
✅ Implementation completed
✅ Tests run and passing
✅ Clear reporting at each step
