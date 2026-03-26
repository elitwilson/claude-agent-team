#!/bin/bash
# Hook: TaskCreated
#
# Fires when a task is being created.
# Future use: reject malformed or out-of-scope tasks before they enter
# the task queue, rather than relying on prompt guardrails alone.
#
# Exit 0 to allow creation.
# Exit 2 to block and send feedback (message on stdout).

exit 0
