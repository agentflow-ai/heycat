#!/bin/bash
set -e

MAIN_REPO="$(cd "$(dirname "$0")/.." && pwd)"
WORKTREES_DIR="$MAIN_REPO/worktrees"

usage() {
  cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Start a Claude session in a worktree (create new or resume existing).

Options:
  -i, --issue ID       Linear issue ID (REQUIRED for new worktrees, e.g., HEY-123)
  -r, --resume NAME    Resume session in existing worktree
  -l, --list           List available worktrees
  -h, --help           Show this help

Examples:
  $(basename "$0") --issue HEY-123      # Create worktree for Linear issue
  $(basename "$0") --resume HEY-123-fix-audio  # Resume in existing worktree
  $(basename "$0") -l                   # List worktrees
  $(basename "$0")                      # Interactive: prompts for Linear issue

Note: All new worktrees require a Linear issue ID (format: HEY-xxx)
EOF
}

list_worktrees() {
  echo "Available worktrees:"
  if [ -d "$WORKTREES_DIR" ]; then
    local found=0
    for dir in "$WORKTREES_DIR"/*; do
      if [ -d "$dir" ] && [ "$(basename "$dir")" != ".*" ]; then
        echo "  - $(basename "$dir")"
        found=1
      fi
    done
    if [ $found -eq 0 ]; then
      echo "  (none)"
    fi
  else
    echo "  (none)"
  fi
}

start_claude_in() {
  local path="$1"
  echo ""
  echo "Starting Claude in: $path"
  echo ""
  cd "$path"
  exec claude --dangerously-skip-permissions
}

# Parse arguments
RESUME=""
ISSUE_ID=""
while [[ $# -gt 0 ]]; do
  case $1 in
    -i|--issue) ISSUE_ID="$2"; shift 2 ;;
    -r|--resume) RESUME="$2"; shift 2 ;;
    -l|--list) list_worktrees; exit 0 ;;
    -h|--help) usage; exit 0 ;;
    -*) echo "Unknown option: $1"; usage; exit 1 ;;
    *) echo "Error: Positional arguments not allowed. Use --issue HEY-xxx"; usage; exit 1 ;;
  esac
done

# Validate issue ID format if provided
if [ -n "$ISSUE_ID" ]; then
  if ! [[ "$ISSUE_ID" =~ ^HEY-[0-9]+$ ]]; then
    echo "Error: Invalid issue ID format: $ISSUE_ID"
    echo "Expected format: HEY-<number> (e.g., HEY-123)"
    exit 1
  fi
fi

# Show configuration
echo ""
echo "Configuration:"
if [ -n "$RESUME" ]; then
  echo "  Mode: Resume existing worktree"
  echo "  Name: $RESUME"
elif [ -n "$ISSUE_ID" ]; then
  echo "  Mode: Create from Linear issue"
  echo "  Issue: $ISSUE_ID"
else
  echo "  Mode: Interactive (will prompt for Linear issue ID)"
fi
echo ""

# Resume mode: go directly to existing worktree
if [ -n "$RESUME" ]; then
  WORKTREE_PATH="$WORKTREES_DIR/$RESUME"
  if [ ! -d "$WORKTREE_PATH" ]; then
    echo "Error: Worktree not found: $RESUME"
    list_worktrees
    exit 1
  fi
  start_claude_in "$WORKTREE_PATH"
fi

# Create mode: use Claude CLI to create worktree
cd "$MAIN_REPO"

# Check for jq dependency
if ! command -v jq &> /dev/null; then
  echo "Error: jq is required but not installed."
  echo "Install with: brew install jq"
  exit 1
fi

SCHEMA='{"type":"object","properties":{"worktreePath":{"type":"string","description":"Full absolute path to the created worktree"},"success":{"type":"boolean"},"error":{"type":"string"}},"required":["success","worktreePath"]}'

# Build prompt with issue ID context
if [ -n "$ISSUE_ID" ]; then
  BRANCH_CONTEXT="Linear issue: $ISSUE_ID
Ask me for a short description (2-3 words, kebab-case) to complete the branch name.
Branch format will be: $ISSUE_ID-<description> (e.g., $ISSUE_ID-fix-audio)"
else
  BRANCH_CONTEXT="No issue ID provided.
Ask me for a Linear issue ID (format: HEY-xxx, e.g., HEY-123).
Then ask for a short description (2-3 words, kebab-case).
The branch format will be: HEY-<number>-<description> (e.g., HEY-123-fix-audio)
IMPORTANT: A Linear issue ID is REQUIRED. Do not proceed without one."
fi

PROMPT="Create a git worktree for feature development.

$BRANCH_CONTEXT

Steps:
1. Verify we're in the main repo (not a worktree) - check if .git is a directory
2. Check for clean working directory with git status --porcelain
3. If no issue ID provided, ask for one (format: HEY-xxx) - this is REQUIRED
4. Get a short description from the user (2-3 words, kebab-case)
5. Construct branch name as: <issue-id>-<description>
6. Validate that branch name matches HEY-\\d+-\\w+ pattern
7. Fetch origin main
8. Run: bun scripts/create-worktree.ts <branch-name>
9. Run: cd <worktree-path> && bun install

IMPORTANT:
- A Linear issue ID (HEY-xxx) is MANDATORY
- Do NOT accept branch names that don't start with HEY-<number>
- Return the full absolute path to the worktree in your response."

echo "Creating worktree via Claude..."
echo "  Sending request to Claude CLI..."

RESULT=$(claude -p "$PROMPT" \
  --output-format json \
  --json-schema "$SCHEMA" \
  --allowedTools "Bash,Read")

echo "  Response received from Claude"

# Check if Claude reported an error
IS_ERROR=$(echo "$RESULT" | jq -r '.is_error // false' 2>/dev/null)
if [ "$IS_ERROR" = "true" ]; then
  echo ""
  echo "Error: Claude reported an error"
  echo "$RESULT" | jq -r '.result // "No details"' 2>/dev/null
  exit 1
fi

# Extract worktree path from JSON response
WORKTREE_PATH=$(echo "$RESULT" | jq -r '.structured_output.worktreePath // empty' 2>/dev/null)
echo "  Extracted path: $WORKTREE_PATH"

if [ -z "$WORKTREE_PATH" ]; then
  echo ""
  echo "Failed to extract worktree path from Claude's response"
  echo ""
  echo "Expected field: .structured_output.worktreePath"
  echo "Raw response:"
  echo "$RESULT" | jq '.' 2>/dev/null || echo "$RESULT"
  exit 1
fi

if [ ! -d "$WORKTREE_PATH" ]; then
  echo "Error: Worktree path does not exist: $WORKTREE_PATH"
  exit 1
fi

start_claude_in "$WORKTREE_PATH"
