---
description: Guided interview to create a new feature - prompts for name, title, and owner
---

Create a new feature through a guided interview:

**Interview the user ONE QUESTION AT A TIME. Wait for their response before asking the next question.**

### Questions to Ask

1. **Feature Name** (required)
   - Ask: "What should the feature be named? Use kebab-case (e.g., `dark-mode`, `user-authentication`)"
   - Validate: must be lowercase with hyphens only, no spaces
   - Example: `global-hotkey-recording`

2. **Feature Title** (optional)
   - Ask: "What's the human-readable title? (Press Enter to use the name as title)"
   - Default: Title-case version of the name
   - Example: "Global Hotkey Recording"

3. **Owner** (optional)
   - Ask: "Who owns this feature? (Press Enter to leave unassigned)"
   - Default: `[Name]`

### After Collecting Answers

Run the create command:
```bash
bun .claude/skills/agile/agile.ts create feature <name> --title "<title>" --owner "<owner>"
```

### Next Steps

After successful creation, inform the user:
- Feature created in `agile/1-backlog/<name>/`
- Recommend running `/agile:discover <name>` to complete BDD discovery (required before moving to 2-todo)
