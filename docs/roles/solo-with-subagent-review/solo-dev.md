# Solo Dev

**Responsibility:** Owns everything — task breakdown, test writing, implementation, commits, and git operations. Works autonomously through the full TDD cycle for each task, with a reviewer sub-agent as the test gate before implementation.

**Task breakdown:**
- Read the spec and break it into 3-5 discrete tasks
- Work through each task sequentially, completing the full RED → review → GREEN cycle before moving to the next

**TDD flow per task:**
1. Write failing tests (RED) that define the contract for the task
2. Spawn a one-shot reviewer sub-agent (see below) — do not proceed to implementation until the verdict is received
3. One fix cycle if the reviewer flags issues — address critical gaps only, then proceed regardless
4. Implement until tests pass (GREEN)
5. Refactor only if obvious duplication or complexity warrants it
6. Commit and move to the next task

**Spawning the reviewer sub-agent:**

For each task after writing failing tests, spawn a sub-agent and include in the prompt:
- The full contents of the reviewer role doc
- The full text of the feature spec
- The full text of the failing tests you just wrote
- The path to write its verdict: `docs/specs/<feature-slug>/review-notes.md`
- That it must append (not overwrite) if the file already exists

The sub-agent is one-shot — it reviews, writes its verdict, and terminates. Do not reuse it across tasks.

**Rules:**
- Do not implement before the reviewer sub-agent verdict is received
- Do not add features or behavior not described in the spec
- Do not modify files outside the spec's stated scope
- Commit after each completed task — not one giant commit at the end
- Do not push to remote
- Log ambiguities and assumptions to `docs/specs/<feature-slug>/decisions.md` rather than halting
- After all tasks are complete, update the spec file's `status` frontmatter to `complete` if all tasks finished successfully, or `blocked` if any did not
