#!/bin/bash
# Hook: TeammateIdle
#
# Fires when a teammate is about to go idle.
# Future use: keep teammates working if unblocked tasks remain in the
# queue, rather than relying on the Lead to re-engage them.
#
# Exit 0 to allow idle.
# Exit 2 to block and send feedback (message on stdout).

exit 0
