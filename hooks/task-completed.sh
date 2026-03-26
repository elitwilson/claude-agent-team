#!/bin/bash
# Hook: TaskCompleted
#
# Fires when a task is being marked complete.
# Future use: enforce that Reviewer has written to review-notes.md before
# a Coder task can complete, providing a hard gate rather than relying on
# prompt instructions alone.
#
# Exit 0 to allow completion.
# Exit 2 to block and send feedback (message on stdout).

exit 0
