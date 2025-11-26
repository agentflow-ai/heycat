const HELP_TEXT = `
TCR (Test-Commit-Refactor) Skill

Automates the test-commit-refactor workflow using Claude Code hooks.

USAGE:
  bun .claude/skills/tcr/tcr.ts <command> [options]

COMMANDS:
  run [files...]     Run tests for specific files
  status [--coverage] Show current TCR state (step, failures, last result)
  reset              Reset failure counter to continue past threshold
  coverage [target]  Run coverage checks (frontend, backend, or both)
  verify-config      Verify coverage thresholds are in sync across all config files
  help               Show this help message

WORKFLOW:
  1. Mark a todo as "completed" → tests run automatically
  2. Tests pass → WIP commit created
  3. Tests fail → failure counter increments
  4. 5 failures → prompted to reconsider approach
  5. Git commits blocked until tests pass

TEST DISCOVERY:
  Convention-based: foo.ts → foo.test.ts or foo.spec.ts
  Backend: src-tauri/ changes trigger cargo test

EXAMPLES:
  bun .claude/skills/tcr/tcr.ts run src/App.tsx
  bun .claude/skills/tcr/tcr.ts status
  bun .claude/skills/tcr/tcr.ts reset
`;

const COMMAND_HELP: Record<string, string> = {
  run: `
TCR RUN - Run tests for specific files

USAGE:
  bun .claude/skills/tcr/tcr.ts run [files...]

ARGUMENTS:
  files    Source files to find tests for (uses convention-based discovery)

EXAMPLES:
  bun .claude/skills/tcr/tcr.ts run src/App.tsx
  bun .claude/skills/tcr/tcr.ts run src/utils/auth.ts src/utils/validation.ts
`,

  status: `
TCR STATUS - Show current TCR state

USAGE:
  bun .claude/skills/tcr/tcr.ts status
  bun .claude/skills/tcr/tcr.ts status --coverage  # Include live coverage metrics

OPTIONS:
  --coverage, -c   Run tests and display current coverage metrics

DISPLAYS:
  - Current step being worked on
  - Failure count for current step
  - Last test result and timestamp
  - Recent errors from .tcr-errors.log (if any)
`,

  reset: `
TCR RESET - Reset failure counter

USAGE:
  bun .claude/skills/tcr/tcr.ts reset

Use this command to continue past the 5-failure threshold when you want
to keep working on the current approach.
`,

  coverage: `
TCR COVERAGE - Run coverage checks

USAGE:
  bun .claude/skills/tcr/tcr.ts coverage [target]

ARGUMENTS:
  target   Optional: "frontend", "backend", or omit for both

EXAMPLES:
  bun .claude/skills/tcr/tcr.ts coverage          # Run both
  bun .claude/skills/tcr/tcr.ts coverage frontend # Frontend only
  bun .claude/skills/tcr/tcr.ts coverage backend  # Backend only
`,

  "verify-config": `
TCR VERIFY-CONFIG - Verify coverage configuration sync

USAGE:
  bun .claude/skills/tcr/tcr.ts verify-config

Checks that coverage thresholds are consistent across all three locations:
  1. TCR config (.claude/skills/tcr/lib/coverage/config.ts)
  2. Vitest config (vitest.config.ts)
  3. Husky pre-commit (.husky/pre-commit)

Exits with code 1 if any mismatches are found.
`,
};

export function handleHelp(args: string[]): void {
  const command = args[0];

  if (command && COMMAND_HELP[command]) {
    console.log(COMMAND_HELP[command]);
  } else {
    console.log(HELP_TEXT);
  }
}
