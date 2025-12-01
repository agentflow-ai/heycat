---
description: Guides BDD scenario creation through product research questions
---

Run BDD discovery for a feature to define scenarios through guided product research:

1. Run `bun .claude/skills/agile/agile.ts discover <issue-name>` to see current phase and guidance
2. The output shows:
   - Current discovery phase (persona, paths, scope, synthesize, complete)
   - Questions to ask for the current phase
   - Expected output format to add to the feature file

**Interview the user ONE QUESTION AT A TIME. Wait for their response before asking the next question.**

When a phase is complete:
- Use the Edit tool to add the gathered information to `## BDD Scenarios` in the feature file
- Run `bun .claude/skills/agile/agile.ts discover <issue-name> advance` to move to next phase

**Discovery Phases:**

| Phase | Purpose | Output to add |
|-------|---------|---------------|
| persona | WHO/WHAT/WHY | `### User Persona`, `### Problem Statement` |
| paths | Happy + failure flows | Gherkin scenarios with Given/When/Then |
| scope | Boundaries | `### Out of Scope`, `### Assumptions` |
| synthesize | Validate completeness | Fix any validation errors |

**Useful subcommands:**
- `discover <name>` - Show current phase guidance
- `discover <name> advance` - Move to next phase
- `discover <name> status` - Show progress and validation status
- `discover <name> validate` - Check BDD format without advancing
- `discover <name> reset` - Restart discovery

**Stage Gate:** Features cannot move to `2-todo` without completing discovery (phase: complete) and passing BDD validation. Use `--force` to bypass if needed.
