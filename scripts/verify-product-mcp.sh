#!/usr/bin/env bash
# Cold-path product verify: init → memory status/search → MCP ops gate.
# Does not require LAN hosts, CT101, Redis, or Qdrant.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${GZMO_BIN:-}"
if [[ -z "$BIN" ]]; then
  if [[ -x "${ROOT}/target/release/gzmo" ]]; then
    BIN="${ROOT}/target/release/gzmo"
  elif command -v gzmo >/dev/null 2>&1; then
    BIN="$(command -v gzmo)"
  else
    echo "[!] No gzmo binary. Build: cargo build --release -p gzmo-cli" >&2
    exit 1
  fi
fi

VERIFY_DIR="${VERIFY_DIR:-$(mktemp -d /tmp/gzmo-product-verify-XXXX)}"
cleanup() {
  if [[ "${KEEP_VERIFY_DIR:-}" != "1" ]]; then
    rm -rf "$VERIFY_DIR"
  else
    echo "[*] Kept VERIFY_DIR=$VERIFY_DIR"
  fi
}
trap cleanup EXIT

echo "[*] Binary: $BIN"
echo "[*] Init → $VERIFY_DIR"
"$BIN" init --force --dir "$VERIFY_DIR" --bin "$BIN" >/dev/null 2>&1

if rg -n '192\.168|CT101|neo4j' "$VERIFY_DIR/gzmo.toml" "$VERIFY_DIR/mcp.json" >/dev/null 2>&1; then
  echo "[FAIL] LAN / CT101 / neo4j found in product config" >&2
  exit 1
fi
echo "[OK] product config has no LAN hosts"

export GZMO_CONFIG="$VERIFY_DIR/gzmo.toml"
export GZMO_ALLOW_LAB_VAULT=1
export GZMO_PRODUCT=1
unset GZMO_OPS_MCP || true

STATUS_JSON="$("$BIN" memory status --json)"
echo "$STATUS_JSON" | python3 -c 'import json,sys; d=json.load(sys.stdin); assert "vault_path" in d; print("[OK] memory status", d["vault_path"], "facts=", d["vault_facts"])'

"$BIN" memory search "smoke" --limit 2 >/dev/null
echo "[OK] memory search (empty vault ok)"

python3 - <<PY
import json, os, subprocess

bin_path = "${BIN}"
env = {
    **os.environ,
    "GZMO_CONFIG": "${VERIFY_DIR}/gzmo.toml",
    "GZMO_ALLOW_LAB_VAULT": "1",
    "GZMO_PRODUCT": "1",
    "RUST_LOG": "error",
}
env.pop("GZMO_OPS_MCP", None)

def call(tool, ops=False):
    e = dict(env)
    if ops:
        e["GZMO_OPS_MCP"] = "1"
    proc = subprocess.Popen(
        [bin_path, "mcp-serve"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        env=e,
    )
    def send(msg):
        proc.stdin.write(json.dumps(msg) + "\n")
        proc.stdin.flush()
    send({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
        "protocolVersion":"2024-11-05","capabilities":{},
        "clientInfo":{"name":"verify-product","version":"1"}}})
    proc.stdout.readline()
    send({"jsonrpc":"2.0","method":"notifications/initialized"})
    send({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":tool,"arguments":{}}})
    out = None
    for _ in range(20):
        line = proc.stdout.readline()
        if not line:
            break
        line = line.strip()
        if not line.startswith("{"):
            continue
        msg = json.loads(line)
        if msg.get("id") == 2:
            out = msg
            break
    proc.terminate()
    return out

st = call("gzmo_memory_status")
assert st and st.get("result"), st
assert "vault_path" in json.dumps(st)
print("[OK] mcp gzmo_memory_status")

deny = call("gzmo_ops_health")
text = json.dumps(deny)
assert "GZMO_OPS_MCP" in text and deny["result"].get("isError") is True
print("[OK] mcp gzmo_ops_health gated without GZMO_OPS_MCP")

proc = subprocess.Popen(
    [bin_path, "mcp-serve"],
    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
    text=True, env=env,
)
def send(msg):
    proc.stdin.write(json.dumps(msg) + "\n")
    proc.stdin.flush()
send({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
    "protocolVersion":"2024-11-05","capabilities":{},
    "clientInfo":{"name":"verify-product","version":"1"}}})
proc.stdout.readline()
send({"jsonrpc":"2.0","method":"notifications/initialized"})
send({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{
    "name":"gzmo_memory_search","arguments":{"query":"smoke","limit":2}}})
out = None
for _ in range(20):
    line = proc.stdout.readline()
    if not line:
        break
    line = line.strip()
    if not line.startswith("{"):
        continue
    msg = json.loads(line)
    if msg.get("id") == 2:
        out = msg
        break
proc.terminate()
assert out and out.get("result"), out
print("[OK] mcp gzmo_memory_search")
PY

echo "[OK] product MCP cold path verified"
