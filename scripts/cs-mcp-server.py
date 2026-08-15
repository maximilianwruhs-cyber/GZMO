#!/usr/bin/env python3
"""
Code Stitcher MCP Server — stdio JSON-RPC 2.0 Bridge

Wraps the `code-stitcher` CLI as MCP tools for agent consumption.
Nutzt die vorhandene CLI — keine Rust-Code-Änderungen.

Usage:
  export CODE_STITCHER_BIN=/home/gzmo/Projects/code-stitcher/target/release/code-stitcher
  python3 scripts/cs-mcp-server.py

Registration (OpenClaw mcp.json):
```json
{
  "code-stitcher": {
    "command": "python3",
    "args": ["/abs/pfad/zu/scripts/cs-mcp-server.py"],
    "env": {
      "CODE_STITCHER_BIN": "/home/gzmo/Projects/code-stitcher/target/release/code-stitcher",
      "CS_INGREDIENTS": "/home/gzmo/Projects/code-stitcher/fixtures/ingested_ingredients",
      "CS_RECIPES_DIR": "/home/gzmo/Projects/code-stitcher/recipes"
    }
  }
}
```
"""

import json
import os
import shlex
import subprocess
import sys
from pathlib import Path

# ── Config ─────────────────────────────────────────────────────────
CODE_STITCHER_BIN = os.environ.get(
    "CODE_STITCHER_BIN",
    "/home/gzmo/Projects/code-stitcher/target/release/code-stitcher"
)
CS_INGREDIENTS = os.environ.get(
    "CS_INGREDIENTS",
    "/home/gzmo/Projects/code-stitcher/fixtures/ingested_ingredients"
)
CS_RECIPES_DIR = os.environ.get(
    "CS_RECIPES_DIR",
    "/home/gzmo/Projects/code-stitcher/recipes"
)

# ── JSON-RPC Helpers ───────────────────────────────────────────────

def rpc_error(code: int, message: str, req_id=None):
    return {
        "jsonrpc": "2.0",
        "id": req_id,
        "error": {"code": code, "message": message}
    }

def rpc_result(result, req_id):
    return {
        "jsonrpc": "2.0",
        "id": req_id,
        "result": result
    }

def text_content(text: str, is_error: bool = False) -> list:
    content = [{"type": "text", "text": text}]
    if is_error:
        return {"isError": True, "content": content}
    return {"content": content}


# ── Tool Implementations ───────────────────────────────────────────

def tool_ingest(source: str) -> dict:
    """Parse a Rust source file → ingredient JSON (BLAKE3 id)."""
    source_path = Path(source)
    if not source_path.exists():
        return text_content(f"Source file not found: {source}", is_error=True)
    try:
        result = subprocess.run(
            [CODE_STITCHER_BIN, "ingest", "--source", str(source_path)],
            capture_output=True, text=True, timeout=30
        )
        if result.returncode != 0:
            return text_content(
                f"Ingest failed (exit {result.returncode}):\n{result.stderr.strip()}",
                is_error=True
            )
        return text_content(result.stdout.strip() or result.stderr.strip())
    except subprocess.TimeoutExpired:
        return text_content("Ingest timed out after 30s", is_error=True)
    except FileNotFoundError:
        return text_content(
            f"code-stitcher binary not found at: {CODE_STITCHER_BIN}",
            is_error=True
        )


def tool_stitch(recipe_path: str, run: bool = False) -> dict:
    """Stitch recipe + ingredients → emit source / compile binary."""
    if not os.path.exists(recipe_path):
        return text_content(f"Recipe file not found: {recipe_path}", is_error=True)

    cmd = [
        CODE_STITCHER_BIN, "stitch",
        "--recipe", recipe_path,
        "--ingredients", CS_INGREDIENTS,
    ]
    output_path = None
    if run:
        output_path = f"./stitched_{Path(recipe_path).stem}"
        cmd.extend(["--output", output_path, "--run"])

    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
        if result.returncode != 0:
            return text_content(
                f"Stitch failed (exit {result.returncode}):\n{result.stderr.strip()}",
                is_error=True
            )
        output = result.stdout.strip() or result.stderr.strip()
        if run:
            output += f"\n\nBinary compiled: {output_path}"
        return text_content(output)
    except subprocess.TimeoutExpired:
        return text_content("Stitch timed out after 60s", is_error=True)
    except FileNotFoundError:
        return text_content(
            f"code-stitcher binary not found at: {CODE_STITCHER_BIN}",
            is_error=True
        )


def tool_emit_source(recipe_path: str) -> dict:
    """Dump stitched source code without compiling."""
    if not os.path.exists(recipe_path):
        return text_content(f"Recipe file not found: {recipe_path}", is_error=True)
    try:
        result = subprocess.run(
            [
                CODE_STITCHER_BIN, "emit-source",
                "--recipe", recipe_path,
                "--ingredients", CS_INGREDIENTS,
            ],
            capture_output=True, text=True, timeout=30
        )
        if result.returncode != 0:
            return text_content(
                f"emit-source failed (exit {result.returncode}):\n{result.stderr.strip()}",
                is_error=True
            )
        return text_content(
            f"```rust\n{result.stdout.strip()}\n```\n---\n{result.stderr.strip()}"
        )
    except subprocess.TimeoutExpired:
        return text_content("emit-source timed out after 30s", is_error=True)
    except FileNotFoundError:
        return text_content(
            f"code-stitcher binary not found at: {CODE_STITCHER_BIN}",
            is_error=True
        )


def tool_list_ingredients() -> dict:
    """List all ingredients in the store."""
    ingredients_dir = Path(CS_INGREDIENTS)
    if not ingredients_dir.exists():
        return text_content(f"Ingredients directory not found: {ingredients_dir}", is_error=True)

    files = sorted(ingredients_dir.glob("*.json"))
    if not files:
        return text_content("No ingredients found.")

    lines = [f"Ingredient Store: {ingredients_dir}", f"Total: {len(files)} files", ""]
    for f in files:
        try:
            data = json.loads(f.read_text())
            name = data.get("name", "?")
            granularity = data.get("granularity", "?")
            node_id = data.get("id", "?")[:12]
            lines.append(f"  • {f.name} — {name} ({granularity}) [{node_id}…]")
        except json.JSONDecodeError:
            lines.append(f"  • {f.name} — ⚠️ corrupt")

    return text_content("\n".join(lines))


def tool_list_recipes() -> dict:
    """List available recipes (approved + drafts)."""
    recipes_dir = Path(CS_RECIPES_DIR)
    if not recipes_dir.exists():
        return text_content(f"Recipes directory not found: {recipes_dir}", is_error=True)

    lines = [f"Recipes: {recipes_dir}", ""]

    for subdir in ["approved", "drafts"]:
        dir_path = recipes_dir / subdir
        if not dir_path.exists():
            continue
        recipe_files = sorted(dir_path.glob("*.json"))
        if not recipe_files:
            lines.append(f"## {subdir} (empty)")
            continue
        lines.append(f"## {subdir} ({len(recipe_files)} recipes)")
        for f in recipe_files:
            try:
                data = json.loads(f.read_text())
                rid = data.get("recipe_id", "?")
                name = data.get("name", "?")
                approved = data.get("approved", False)
                steps = len(data.get("execution_dag", {}).get("nodes", []))
                badge = "✅" if approved else "⏳"
                lines.append(f"  {badge} {rid} — {name} ({steps} steps)")
            except json.JSONDecodeError:
                lines.append(f"  ⚠️ {f.name} — corrupt")
        lines.append("")

    return text_content("\n".join(lines))


def tool_verify_recipe(recipe_path: str) -> dict:
    """Verify a recipe's integrity and approval status."""
    if not os.path.exists(recipe_path):
        return text_content(f"Recipe file not found: {recipe_path}", is_error=True)
    try:
        data = json.loads(Path(recipe_path).read_text())
    except json.JSONDecodeError as e:
        return text_content(f"Invalid JSON: {e}", is_error=True)

    checks = []
    rid = data.get("recipe_id", "unknown")
    checks.append(f"Recipe ID: {rid}")
    checks.append(f"Name: {data.get('name', '?')}")
    checks.append(f"Target: {data.get('target_runtime', '?')}")
    checks.append(f"Approved: {data.get('approved', False)}")
    checks.append(f"Approved by: {data.get('approved_by', '?')}")
    checks.append(f"Approved at: {data.get('approved_at', '?')}")
    checks.append(f"Signature: {'✅ present' if data.get('signature') else '⏳ none'}")
    checks.append(f"Public Key: {'✅ present' if data.get('public_key') else '⏳ none'}")

    dag = data.get("execution_dag", {})
    nodes = dag.get("nodes", [])
    edges = dag.get("edges", [])
    checks.append(f"DAG: {len(nodes)} nodes, {len(edges)} edges")

    if not nodes:
        checks.append("⚠️ Empty DAG — stitch will fail")

    return text_content("\n".join(checks))


# ── Tool Registry ─────────────────────────────────────────────────

TOOLS = [
    {
        "name": "cs_ingest",
        "description": "Parse Rust source file → content-addressed Ingredient (BLAKE3). Speichert JSON in fixtures/ingested_ingredients/.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "source": {"type": "string", "description": "Pfad zur .rs Datei"}
            },
            "required": ["source"]
        }
    },
    {
        "name": "cs_stitch",
        "description": "Stitch recipe + ingredients → emit source + (optional) compile + run native binary.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "recipe": {"type": "string", "description": "Pfad zur Recipe JSON Datei"},
                "run": {"type": "boolean", "description": "Optional: compile + run binary (default: false)"}
            },
            "required": ["recipe"]
        }
    },
    {
        "name": "cs_emit_source",
        "description": "Dump stitched source code ohne zu kompilieren. Gut zum Inspizieren des generierten Codes.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "recipe": {"type": "string", "description": "Pfad zur Recipe JSON Datei"}
            },
            "required": ["recipe"]
        }
    },
    {
        "name": "cs_list_ingredients",
        "description": "Liste aller verfügbaren Ingredients im Store (Name, Granularity, BLAKE3 ID).",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    },
    {
        "name": "cs_list_recipes",
        "description": "Liste aller Rezepte (approved + drafts) mit Namen, Step-Anzahl und Approval-Status.",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    },
    {
        "name": "cs_verify_recipe",
        "description": "Prüfe ein Recipe auf Integrität: Approval-Status, Signatur, Public Key, DAG-Struktur.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "recipe": {"type": "string", "description": "Pfad zur Recipe JSON Datei"}
            },
            "required": ["recipe"]
        }
    }
]

TOOL_DISPATCH = {
    "cs_ingest": tool_ingest,
    "cs_stitch": tool_stitch,
    "cs_emit_source": tool_emit_source,
    "cs_list_ingredients": tool_list_ingredients,
    "cs_list_recipes": tool_list_recipes,
    "cs_verify_recipe": tool_verify_recipe,
}


# ── MCP Server ────────────────────────────────────────────────────

def handle_request(req: dict) -> dict | None:
    req_id = req.get("id")
    method = req.get("method", "")
    params = req.get("params", {})

    if method == "initialize":
        return rpc_result({
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "code-stitcher-mcp", "version": "0.1.0"}
        }, req_id)

    if method == "notifications/initialized":
        return None

    if method == "tools/list":
        return rpc_result({"tools": TOOLS}, req_id)

    if method == "tools/call":
        tool_name = params.get("name", "")
        args = params.get("arguments", {})

        if tool_name not in TOOL_DISPATCH:
            return rpc_error(-32601, f"Unknown tool: {tool_name}", req_id)

        try:
            if tool_name == "cs_ingest":
                result = TOOL_DISPATCH[tool_name](args.get("source", ""))
            elif tool_name == "cs_stitch":
                result = TOOL_DISPATCH[tool_name](
                    args.get("recipe", ""),
                    run=args.get("run", False)
                )
            elif tool_name == "cs_emit_source":
                result = TOOL_DISPATCH[tool_name](args.get("recipe", ""))
            elif tool_name == "cs_verify_recipe":
                result = TOOL_DISPATCH[tool_name](args.get("recipe", ""))
            else:
                result = TOOL_DISPATCH[tool_name]()

            return rpc_result(result, req_id)
        except Exception as e:
            return rpc_result(
                {"isError": True, "content": [{"type": "text", "text": f"Error: {e}"}]},
                req_id
            )

    return rpc_error(-32601, f"Method not found: {method}", req_id)


def main():
    # Print startup info to stderr (MCP protocol uses stdout for JSON-RPC)
    print(f"[cs-mcp] Code Stitcher MCP Server v0.1.0", file=sys.stderr)
    print(f"[cs-mcp] Binary: {CODE_STITCHER_BIN}", file=sys.stderr)
    print(f"[cs-mcp] Ingredients: {CS_INGREDIENTS}", file=sys.stderr)
    print(f"[cs-mcp] Recipes: {CS_RECIPES_DIR}", file=sys.stderr)

    # Verify binary exists
    if not os.path.exists(CODE_STITCHER_BIN):
        print(f"[cs-mcp] ⚠️ Binary not found: {CODE_STITCHER_BIN}", file=sys.stderr)
        print(f"[cs-mcp] Build with: cd /home/gzmo/Projects/code-stitcher && cargo build --release", file=sys.stderr)

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = json.loads(line)
            response = handle_request(request)
            if response:
                sys.stdout.write(json.dumps(response) + "\n")
                sys.stdout.flush()
        except json.JSONDecodeError as e:
            err_resp = rpc_error(-32700, f"Parse error: {e}")
            sys.stdout.write(json.dumps(err_resp) + "\n")
            sys.stdout.flush()


if __name__ == "__main__":
    main()
