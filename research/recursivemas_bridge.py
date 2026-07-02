#!/usr/bin/env python3
"""RecursiveMAS bridge — FastAPI when available, stdlib HTTP fallback otherwise.

Endpoints:
  GET  /health
  POST /infer  JSON: {"question": "...", "style": "sequential_light", "recursion_rounds": 1}

Environment:
  RECURSIVEMAS_MOCK=1          Use mock responses (no GPU / HF weights)
  RECURSIVEMAS_ROOT            Path to cloned RecursiveMAS repo
  RECURSIVEMAS_STYLE           Default collaboration style (sequential_light)
  RECURSIVEMAS_DATASET         Adapter task selector (math500)
  RECURSIVEMAS_DEVICE          cuda:0 | cpu
"""

from __future__ import annotations

import argparse
import json
import os
import time
from http.server import BaseHTTPRequestHandler, HTTPServer
from typing import Any

_MAS: Any = None
_LOAD_ERROR: str | None = None
_BACKEND: str = "unavailable"


def _try_load_mas() -> None:
    global _MAS, _LOAD_ERROR, _BACKEND
    if os.getenv("RECURSIVEMAS_MOCK", "").strip().lower() in {"1", "true", "yes"}:
        _MAS = "mock"
        _BACKEND = "mock"
        _LOAD_ERROR = None
        return

    try:
        from recursivemas_runner import load_runner

        style = os.getenv("RECURSIVEMAS_STYLE", "sequential_light")
        dataset = os.getenv("RECURSIVEMAS_DATASET", "math500")
        device = os.getenv("RECURSIVEMAS_DEVICE")
        _MAS = load_runner(style=style, dataset=dataset, device=device)
        _BACKEND = "recursivemas"
        _LOAD_ERROR = None
    except Exception as exc:
        _MAS = None
        _BACKEND = "unavailable"
        _LOAD_ERROR = str(exc)


def infer_payload(body: dict[str, Any]) -> dict[str, Any]:
    question = str(body.get("question", ""))
    style = str(body.get("style", os.getenv("RECURSIVEMAS_STYLE", "sequential_light")))
    rounds = int(body.get("recursion_rounds", 1))
    started = time.perf_counter()

    if _MAS == "mock":
        answer = f"[mock RecursiveMAS {style} r={rounds}] Processed: {question[:200]}"
        latency_ms = int((time.perf_counter() - started) * 1000)
        return {
            "answer": answer,
            "text": answer,
            "style": style,
            "recursion_rounds": rounds,
            "latency_ms": latency_ms,
            "input_tokens": 0,
            "output_tokens": 0,
            "total_tokens": 0,
            "backend": "mock",
            "mock": True,
        }

    if _MAS is None:
        return {
            "error": f"RecursiveMAS not loaded: {_LOAD_ERROR or 'unknown'}",
            "status": 503,
        }

    try:
        if style and style != getattr(_MAS, "style", style):
            from recursivemas_runner import load_runner

            runner = load_runner(style=style)
            result = runner.run(question, recursion_rounds=rounds)
        else:
            result = _MAS.run(question, recursion_rounds=rounds)
        text = result.answer
        latency_ms = result.latency_ms
    except Exception as exc:
        return {"error": str(exc), "status": 500}

    return {
        "answer": text,
        "text": text,
        "style": style,
        "recursion_rounds": rounds,
        "latency_ms": latency_ms,
        "input_tokens": 0,
        "output_tokens": 0,
        "total_tokens": 0,
        "backend": "recursivemas",
    }


class Handler(BaseHTTPRequestHandler):
    def _json(self, code: int, payload: dict[str, Any]) -> None:
        data = json.dumps(payload).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, fmt: str, *args: Any) -> None:
        return

    def do_GET(self) -> None:  # noqa: N802
        if self.path.rstrip("/") == "/health":
            self._json(
                200,
                {
                    "ok": _MAS is not None,
                    "backend": _BACKEND,
                    "style": os.getenv("RECURSIVEMAS_STYLE", "sequential_light"),
                    "device": os.getenv("RECURSIVEMAS_DEVICE", "cuda"),
                    "load_error": _LOAD_ERROR,
                },
            )
            return
        self._json(404, {"error": "not found"})

    def do_POST(self) -> None:  # noqa: N802
        if self.path.rstrip("/") != "/infer":
            self._json(404, {"error": "not found"})
            return
        length = int(self.headers.get("Content-Length", "0"))
        raw = self.rfile.read(length) if length else b"{}"
        try:
            body = json.loads(raw.decode() or "{}")
        except json.JSONDecodeError:
            self._json(400, {"error": "invalid json"})
            return
        out = infer_payload(body)
        status = int(out.pop("status", 200))
        if "error" in out and status >= 400:
            self._json(status, out)
            return
        self._json(200, out)


def serve_stdlib(host: str, port: int) -> None:
    _try_load_mas()
    server = HTTPServer((host, port), Handler)
    print(f"RecursiveMAS bridge (stdlib) http://{host}:{port} backend={_BACKEND}")
    server.serve_forever()


def serve_fastapi(host: str, port: int) -> None:
    from fastapi import FastAPI, HTTPException, Request
    from pydantic import BaseModel, Field

    app = FastAPI(title="RecursiveMAS Bridge", version="0.2.0")

    class InferRequest(BaseModel):
        question: str
        style: str = Field(default="sequential_light")
        recursion_rounds: int = Field(default=1, ge=1, le=3)

    @app.on_event("startup")
    def startup() -> None:
        _try_load_mas()

    @app.get("/health")
    def health() -> dict[str, Any]:
        return {
            "ok": _MAS is not None,
            "backend": _BACKEND,
            "style": os.getenv("RECURSIVEMAS_STYLE", "sequential_light"),
            "device": os.getenv("RECURSIVEMAS_DEVICE", "cuda"),
            "load_error": _LOAD_ERROR,
        }

    @app.post("/infer")
    async def infer(request: Request) -> dict[str, Any]:
        try:
            body = await request.json()
        except Exception:
            raise HTTPException(status_code=400, detail="invalid json") from None
        if not isinstance(body, dict):
            raise HTTPException(status_code=400, detail="invalid json object")
        out = infer_payload(body)
        status = int(out.pop("status", 200))
        if status >= 400:
            raise HTTPException(status_code=status, detail=out.get("error", "error"))
        return out

    import uvicorn

    uvicorn.run(app, host=host, port=port, log_level="warning")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8765)
    parser.add_argument("--fastapi", action="store_true", help="Require FastAPI/uvicorn")
    args = parser.parse_args()

    if args.fastapi:
        serve_fastapi(args.host, args.port)
        return
    # Default: stdlib server (reliable JSON POST for AttractorBench bridge).
    serve_stdlib(args.host, args.port)


if __name__ == "__main__":
    main()
