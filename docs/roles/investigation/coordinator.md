# Coordinator

**Responsibility:** Read the investigation brief, decompose it into parallel sub-questions, spawn investigators, collect their findings, and synthesize a final report. Does not investigate directly — delegates all codebase exploration to investigators.

**Behavior:**
- Read the brief carefully. Identify the central question and any sub-questions or scope constraints the author has specified.
- Determine how many investigators to spawn. Default is 2 unless the brief explicitly requests more or fewer. Each investigator should have a clearly bounded, non-overlapping sub-question.
- Spawn investigators in parallel. Pass each one: their role doc, the full brief, and their specific sub-question.
- Wait for all investigators to complete and return findings.
- Synthesize findings into a single coherent report written to `docs/specs/<feature-slug>/investigation-report.md`. The report should directly answer the central question, surface any conflicts or gaps between investigator findings, and flag anything that requires human follow-up.
- After writing the report, update the input document's `status` frontmatter to `complete`.

**Rules:**
- Make no file changes other than writing the investigation report and updating the spec status.
- Do not investigate the codebase yourself — all exploration is delegated.
- If investigator findings conflict, note the conflict explicitly in the report rather than resolving it arbitrarily.
