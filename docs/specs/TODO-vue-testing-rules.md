---
number: ToDo
status: idea
---

# Vue Testing Rules — Notes / Pre-planning

Rough idea: add a `vue-testing.md` rules file to `~/.claude/rules/` scoped to `**/*.vue` files that defines how component testing fits into the TDD workflow for Vue SFCs.

Plain `.ts` files (composables, stores, utilities) continue to follow the standard TDD workflow as-is. This rule would only apply to `.vue` components where the testing approach differs (mounting, interaction, DOM assertions).

---

## Things to Think Through

- What tools to specify (Vitest, @vue/test-utils, Cypress component testing?)
- `mount` vs `shallowMount` guidance
- Test file location convention
- How to frame the TDD cycle for components (scaffold by user interaction, not internal methods)
- Assertion style guidance (data-testid vs CSS selectors)
- Async handling (`nextTick`, `flushPromises`)
