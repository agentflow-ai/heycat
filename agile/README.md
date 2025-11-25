# Agile Board

Kanban-style issue tracking using markdown files organized by status folders.

## Workflow

```
1-backlog → 2-todo → 3-in-progress → 4-review → 5-done
```

1. **Backlog** - Ideas and future work
2. **Todo** - Ready to be worked on
3. **In Progress** - Currently being worked on
4. **Review** - Awaiting review/feedback
5. **Done** - Completed items

## Creating Issues

Copy a template from `templates/` to the appropriate status folder:

```bash
cp agile/templates/feature.md agile/1-backlog/my-feature.md
```

### Templates
- `feature.md` - New features and enhancements
- `bug.md` - Bug reports
- `task.md` - General tasks

## Moving Issues

Move files between folders as status changes:

```bash
git mv agile/2-todo/my-feature.md agile/3-in-progress/
```

## Naming Convention

Use kebab-case for file names:
- `user-authentication.md`
- `fix-login-bug.md`
- `update-docs.md`
