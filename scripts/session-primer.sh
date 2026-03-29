#!/usr/bin/env bash
set -euo pipefail

SIGIL_BIN="/home/claudedev/sigil/target/debug/sigil"
if [ ! -x "$SIGIL_BIN" ]; then
    SIGIL_BIN="/home/claudedev/sigil/target/release/sigil"
fi
if [ ! -x "$SIGIL_BIN" ]; then
    echo "# Sigil Primer: UNAVAILABLE (binary not found)" >&2
    exit 0
fi

CWD="${PWD}"

# Single MCP session: initialize, get projects, get shared primer
INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"hook","version":"0.1"}}}'
PROJECTS_CALL='{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"sigil_projects","arguments":{}}}'
SHARED_CALL='{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"sigil_primer","arguments":{"project":"shared"}}}'

RESPONSES=$(printf '%s\n%s\n%s\n' "$INIT" "$PROJECTS_CALL" "$SHARED_CALL" | "$SIGIL_BIN" mcp 2>/dev/null)

# Extract shared primer (id=3, last line)
SHARED=$(echo "$RESPONSES" | tail -1 | python3 -c "
import sys, json
try:
    r = json.loads(sys.stdin.read())
    inner = json.loads(r['result']['content'][0]['text'])
    print(inner.get('content', ''))
except Exception:
    pass
" 2>/dev/null)

if [ -n "$SHARED" ]; then
    echo "# Shared Workflow Primer (from Sigil)"
    echo "$SHARED"
fi

# Detect project from PWD
PROJECTS_LINE=$(echo "$RESPONSES" | sed -n '2p')
PROJECT=$(echo "$PROJECTS_LINE" | python3 -c "
import sys, json, os
cwd = os.environ.get('CWD', '')
try:
    r = json.loads(sys.stdin.read())
    inner = json.loads(r['result']['content'][0]['text'])
    best = ''
    best_name = ''
    for p in inner.get('projects', []):
        repo = p.get('repo', '')
        if repo and cwd.startswith(repo) and len(repo) > len(best):
            best = repo
            best_name = p['name']
    if best_name:
        print(best_name)
except Exception:
    pass
" 2>/dev/null)

if [ -z "$PROJECT" ] || [ "$PROJECT" = "shared" ]; then
    exit 0
fi

# Fetch project primer (shared content already printed, so we strip it)
PROJECT_CALL="{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"sigil_primer\",\"arguments\":{\"project\":\"$PROJECT\"}}}"
PROJECT_RESPONSE=$(printf '%s\n%s\n' "$INIT" "$PROJECT_CALL" | "$SIGIL_BIN" mcp 2>/dev/null | tail -1)

PROJECT_CONTENT=$(echo "$PROJECT_RESPONSE" | python3 -c "
import sys, json
try:
    r = json.loads(sys.stdin.read())
    inner = json.loads(r['result']['content'][0]['text'])
    content = inner.get('content', '')
    # The project primer includes shared content after '---', strip it since we already printed it
    if '---' in content:
        content = content[:content.rfind('---')].rstrip()
    if content:
        print(content)
except Exception:
    pass
" 2>/dev/null)

if [ -n "$PROJECT_CONTENT" ]; then
    echo ""
    echo "# Project Primer: $PROJECT (from Sigil)"
    echo "$PROJECT_CONTENT"
fi
