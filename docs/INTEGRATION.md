# Integration Verification

Guidelines for multi-component features.

## 1. Mock Usage Audit

When reviewing specs with mocked dependencies, verify the mocked component is actually instantiated in production code (lib.rs, main.tsx, etc.)

## 2. Deferral Tracking

Any comment like "handled separately", "will be implemented later", or "managed elsewhere" MUST reference a specific spec or ticket. Flag as NEEDS_WORK if no reference exists.

## 3. Final Integration Spec

Multi-component features require a final "integration" spec that:
- Verifies all components are wired together in production
- Includes an integration test (automated)
- Documents the end-to-end flow with file:line references

## 4. Feature Completion Gate

Before moving to 4-review:
- All "handled separately" comments must have corresponding completed specs
- Integration test must exist and pass
