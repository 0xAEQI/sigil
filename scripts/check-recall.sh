#!/usr/bin/env bash
FLAG="/tmp/.sigil_recall_$(echo "$PWD" | md5sum | cut -c1-8)"
if [ -f "$FLAG" ]; then
    if [ "$(find "$FLAG" -mmin -30 2>/dev/null)" ]; then
        echo '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow"}}'
        exit 0
    fi
fi
echo '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"Call sigil_recall() before making non-trivial edits. It surfaces relevant knowledge (bug causes, decisions, patterns) that may affect this change."}}'
