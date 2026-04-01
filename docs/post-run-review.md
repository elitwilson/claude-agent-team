# Post-Run Review Checklist

You're on the feature branch. Run succeeded. Ready to merge?

---

- [ ] `git diff main --stat` — no files outside the spec's **In Scope** section
- [ ] Read `docs/runs/<slug>/decisions.md` — verify every assumption the agent made
- [ ] Read `docs/runs/<slug>/review-notes.md` — scrutinize any tasks the Reviewer flagged
- [ ] Open spec **Success Criteria** — for each line, point to the test and implementation that covers it
- [ ] Tests check observable behavior, not implementation details
- [ ] `cargo test` passes
- [ ] Smoke test the new flow manually
- [ ] Merge
