---
status: completed
created: 2025-12-24
dependencies: ["dictionary-context-assignment"]
review_round: 1
---

# Spec: Dictionary Page Context Indicators

## Description

Add visual indicators to the Dictionary page showing which window contexts each dictionary entry is assigned to. This provides visibility into context-specific entries directly from the Dictionary page, complementing the assignment UI on the Window Contexts page.

## Acceptance Criteria

- [ ] Dictionary entry list shows context assignment badges next to each entry
- [ ] Badge shows context names for 1-2 contexts (e.g., "Slack", "VS Code")
- [ ] Badge shows count for 3+ contexts (e.g., "3 contexts")
- [ ] Entries with no context assignment show "Global" badge
- [ ] Badge styling distinguishes Global (neutral) from context-specific (accent color)
- [ ] Hovering badge with count shows tooltip listing all context names

## Test Cases

- [ ] Entry with no context assignments shows "Global" badge
- [ ] Entry with 1 context shows that context's name as badge
- [ ] Entry with 2 contexts shows both names as badges
- [ ] Entry with 3+ contexts shows "N contexts" badge
- [ ] Tooltip on count badge lists all context names

## Dependencies

- `dictionary-context-assignment` - establishes the dictionaryEntryIds linkage

## Preconditions

- Dictionary page UI exists with entry list
- Window contexts store contains contexts with dictionaryEntryIds
- useDictionary and useWindowContext hooks available

## Implementation Notes

**Approach:**
1. In Dictionary page, fetch window contexts alongside dictionary entries
2. For each dictionary entry, compute which contexts reference it
3. Render badges based on the count of matching contexts

**Data lookup:**
```typescript
// Reverse lookup: entry ID -> contexts that include it
const contextsByEntryId = useMemo(() => {
  const map = new Map<string, WindowContext[]>();
  for (const ctx of contexts) {
    for (const entryId of ctx.dictionaryEntryIds) {
      const existing = map.get(entryId) ?? [];
      existing.push(ctx);
      map.set(entryId, existing);
    }
  }
  return map;
}, [contexts]);
```

**Badge rendering:**
```tsx
const assignedContexts = contextsByEntryId.get(entry.id) ?? [];

{assignedContexts.length === 0 ? (
  <Badge variant="outline">Global</Badge>
) : assignedContexts.length <= 2 ? (
  assignedContexts.map(ctx => <Badge key={ctx.id}>{ctx.name}</Badge>)
) : (
  <Tooltip content={assignedContexts.map(c => c.name).join(", ")}>
    <Badge>{assignedContexts.length} contexts</Badge>
  </Tooltip>
)}
```

## Related Specs

- `dictionary-context-assignment.spec.md` - assigns entries from WindowContexts page

## Integration Points

- Production call site: `src/pages/Dictionary.tsx` - entry list rendering
- Connects to: `useWindowContext` hook for context data

## Integration Test

- Test location: Component tests in Dictionary.test.tsx
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-24
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Dictionary entry list shows context assignment badges next to each entry | PASS | Dictionary.tsx:459 renders `<ContextBadges contexts={assignedContexts} />` for each entry |
| Badge shows context names for 1-2 contexts (e.g., "Slack", "VS Code") | PASS | Dictionary.tsx:113-139 ContextBadges component renders individual badges for 1-2 contexts |
| Badge shows count for 3+ contexts (e.g., "3 contexts") | PASS | Dictionary.tsx:144-152 renders `{contexts.length} contexts` for 3+ |
| Entries with no context assignment show "Global" badge | PASS | Dictionary.tsx:102-111 returns "Global" badge when contexts.length === 0 |
| Badge styling distinguishes Global (neutral) from context-specific (accent color) | PASS | Dictionary.tsx:105 uses `bg-neutral-100 text-neutral-600` for Global; 116, 131, 146 use `bg-purple-100 text-purple-700` for context-specific |
| Hovering badge with count shows tooltip listing all context names | PASS | Dictionary.tsx:148 uses `title={contextNames}` attribute for tooltip |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Entry with no context assignments shows "Global" badge | PASS | Dictionary.test.tsx:409-423 |
| Entry with 1 context shows that context's name as badge | PASS | Dictionary.test.tsx:425-437 |
| Entry with 2 contexts shows both names as badges | PASS | Dictionary.test.tsx:439-479 |
| Entry with 3+ contexts shows "N contexts" badge | PASS | Dictionary.test.tsx:481-494 |
| Tooltip on count badge lists all context names | PASS | Dictionary.test.tsx:496-509 |

### Pre-Review Gate Results

**Build Warning Check:** PASS - No new warnings from Dictionary.tsx implementation. Existing warnings are unrelated (preprocessing, AGC, etc. in Rust backend).

**Command Registration Check:** N/A - This spec adds no new backend commands.

**Event Subscription Check:** N/A - This spec adds no new events.

### Data Flow Verification

```
[UI Render] Dictionary page loads
     |
     v
[Hook] src/hooks/useWindowContext.ts:16-19 contexts = useQuery(list_window_contexts)
     |
     v
[Hook] src/hooks/useDictionary.ts entries = useQuery(list_dictionary_entries)
     |
     v
[Memoization] src/pages/Dictionary.tsx:530-540 contextsByEntryId = useMemo()
     |
     v
[Component] src/pages/Dictionary.tsx:784 assignedContexts={contextsByEntryId.get(entry.id) ?? []}
     |
     v
[Component] src/pages/Dictionary.tsx:101-154 ContextBadges renders badges
```

**Flow verified:** Complete with no broken links.

### Code Quality

**Strengths:**
- Clean implementation of the ContextBadges component with clear separation of concerns
- Efficient reverse lookup using useMemo to avoid recalculation on every render
- Good test coverage with specific test cases for each badge scenario
- Proper use of data-testid attributes for reliable testing
- Tooltip implementation using native title attribute (lightweight, no extra dependencies)
- Proper TypeScript typing with WindowContext[] interface

**Concerns:**
- None identified

### Verdict

**APPROVED** - All acceptance criteria are met with comprehensive test coverage. The implementation correctly wires the window context data into the Dictionary page display, shows appropriate badges based on context count, and provides tooltip functionality for the count badge. All 16 Dictionary tests pass, including the 5 new context badge tests. No deferrals or broken data flow links detected.
