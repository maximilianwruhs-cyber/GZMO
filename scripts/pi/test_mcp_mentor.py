#!/usr/bin/env python3
import sys
import os
import subprocess
import json
import time

def main():
    root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    bin_path = os.path.join(root, "target/release/gzmo")
    config_path = os.path.join(root, "gzmo.toml")
    
    print(f"[*] Starting MCP server {bin_path} with config {config_path}")
    env = os.environ.copy()
    env["GZMO_CONFIG"] = config_path
    
    proc = subprocess.Popen(
        [bin_path, "mcp-serve"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
        env=env
    )
    
    def send(msg):
        payload = json.dumps(msg) + "\n"
        proc.stdin.write(payload)
        proc.stdin.flush()
        print(f"--> {payload.strip()}")
        
    def recv():
        line = proc.stdout.readline()
        if not line:
            return None
        print(f"<-- {line.strip()}")
        return json.loads(line)

    try:
        # Step 1: Initialize
        send({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": {"name": "test-client", "version": "1.0"}
            }
        })
        resp = recv()
        assert resp and resp.get("id") == 1, "Initialize failed"
        
        # Step 2: Initialized notification
        send({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        })
        
        # Step 3: List Tools
        send({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        })
        resp = recv()
        assert resp and "result" in resp, "List tools failed"
        
        tools = [t["name"] for t in resp["result"].get("tools", [])]
        print(f"[*] Discovered MCP tools: {tools}")
        assert "gzmo_mentor_ping" in tools, "gzmo_mentor_ping missing"
        assert "gzmo_mentor_teach" in tools, "gzmo_mentor_teach missing"
        print("[OK] Socratic mentor tools registered successfully!")
        
        # Step 4: Call gzmo_mentor_ping
        send({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "gzmo_mentor_ping",
                "arguments": {}
            }
        })
        resp = recv()
        assert resp and "result" in resp, "Call ping failed"
        content = resp["result"].get("content", [])
        assert len(content) > 0, "No content returned from ping"
        text = content[0].get("text", "")
        print(f"[OK] gzmo_mentor_ping returned: {text}")
        assert "pong" in text, f"Unexpected ping response: {text}"
        
        # Step 5: Call gzmo_mentor_teach
        send({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "gzmo_mentor_teach",
                "arguments": {
                    "message": "teach me what a symbolic link is in Linux"
                }
            }
        })
        resp = recv()
        assert resp and "result" in resp, "Call teach failed"
        content = resp["result"].get("content", [])
        assert len(content) > 0, "No content returned from teach"
        text = content[0].get("text", "")
        print(f"[OK] gzmo_mentor_teach response preview: {text[:100]}...")
        assert len(text) > 20, "Teach response too short"
        
        print("[SUCCESS] All MCP mentor tests passed!")
        
    except Exception as e:
        print(f"[FAIL] Test failed: {e}")
        # print stderr for debugging
        proc.terminate()
        stderr_log = proc.stderr.read()
        if stderr_log:
            print(f"Stderr output:\n{stderr_log}")
        sys.exit(1)
        
    finally:
        proc.terminate()
        proc.wait()

if __name__ == "__main__":
    main()
