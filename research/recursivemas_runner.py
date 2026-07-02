"""Single-question RecursiveMAS inference for the GZMO bridge."""

from __future__ import annotations

import argparse
import os
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from types import SimpleNamespace
from typing import Any

import torch

_INFERENCE_DIR: Path | None = None
_LOAD_ERROR: str | None = None


def _default_recursivemas_root() -> Path:
    env = os.getenv("RECURSIVEMAS_ROOT", "").strip()
    if env:
        return Path(env).expanduser().resolve()
    # research/ -> survey_GZMO -> _foundation-audit -> Projects
    projects = Path(__file__).resolve().parents[3]
    return (projects / "RecursiveMAS").resolve()


def _ensure_inference_path() -> Path:
    global _INFERENCE_DIR, _LOAD_ERROR
    if _INFERENCE_DIR is not None:
        return _INFERENCE_DIR

    inference_dir = _default_recursivemas_root() / "inference"
    if not inference_dir.is_dir():
        _LOAD_ERROR = f"RecursiveMAS inference dir not found: {inference_dir}"
        raise RuntimeError(_LOAD_ERROR)

    parent = str(inference_dir.parent)
    if parent not in sys.path:
        sys.path.insert(0, parent)
    if str(inference_dir) not in sys.path:
        sys.path.insert(0, str(inference_dir))

    os.environ.setdefault("TOKENIZERS_PARALLELISM", "false")
    os.environ.setdefault("MAS_FORCE_DISABLE_TORCHVISION", "1")
    _INFERENCE_DIR = inference_dir
    return inference_dir


@dataclass
class InferResult:
    answer: str
    latency_ms: int
    style: str
    recursion_rounds: int
    backend: str = "recursivemas"


class RecursiveMASRunner:
    """Runs sequential-style RecursiveMAS on one custom question."""

    def __init__(
        self,
        *,
        style: str = "sequential_light",
        dataset: str = "math500",
        device: str | None = None,
    ) -> None:
        _ensure_inference_path()
        from system_loader import resolve_mas_paths  # type: ignore
        from inference_utils import inference_mas as mas  # type: ignore

        self.style = style
        self.dataset = dataset
        self.device = torch.device(
            device or os.getenv("RECURSIVEMAS_DEVICE", "cuda" if torch.cuda.is_available() else "cpu")
        )
        self._mas = mas
        self.paths = resolve_mas_paths(style=style, dataset=dataset)
        self._args = SimpleNamespace(
            mas_shape="chain",
            solver_pre_question=0,
            method="ours_recursive",
        )

        recommended = mas.get_release_recommended_settings(style, dataset) or {}
        self.latent_steps = int(
            os.getenv("RECURSIVEMAS_LATENT_STEPS", recommended.get("latent_length", 32))
        )
        self.batch_size = int(recommended.get("batch_size", 1))
        self.max_new_tokens = int(os.getenv("RECURSIVEMAS_MAX_NEW_TOKENS", "1000"))
        self.temperature = float(os.getenv("RECURSIVEMAS_TEMPERATURE", "0.6"))
        self.top_p = float(os.getenv("RECURSIVEMAS_TOP_P", "0.95"))
        self.model_dtype = mas.resolve_dtype("auto")
        self.outer_dtype = mas.resolve_dtype("auto")
        if self.model_dtype is None or self.outer_dtype is None:
            raise RuntimeError("Unsupported dtype configuration")
        if self.device.type == "cpu":
            if self.model_dtype in {torch.float16, torch.bfloat16}:
                self.model_dtype = torch.float32
            if self.outer_dtype in {torch.float16, torch.bfloat16}:
                self.outer_dtype = torch.float32

        family = self.paths.family
        if family != "sequential":
            raise ValueError(
                f"Bridge runner supports sequential styles only (got family={family}, style={style})"
            )

        self.planner_path = str(self.paths.repo_paths["planner"])
        self.critic_path = str(self.paths.repo_paths["critic"])
        self.solver_path = str(self.paths.repo_paths["solver"])
        self.planner_adapter = str(self.paths.inner_adapter_paths["planner"])
        self.critic_adapter = str(self.paths.inner_adapter_paths["critic"])
        self.solver_adapter = str(self.paths.inner_adapter_paths["solver"])
        self.outer_12_path = str(self.paths.outer_adapter_paths["outer_12"])
        self.outer_23_path = str(self.paths.outer_adapter_paths["outer_23"])
        self.outer_31_path = str(self.paths.outer_adapter_paths["outer_31"])
        self.outer_type = "outer_ln_res_adapter"
        self.inner_fallback = "ln_res_adapter"

    def run(self, question: str, *, recursion_rounds: int = 1) -> InferResult:
        if not question.strip():
            raise ValueError("question must be non-empty")
        if recursion_rounds <= 0:
            raise ValueError("recursion_rounds must be positive")

        mas = self._mas
        started = time.perf_counter()
        questions = [question.strip()]
        device = self.device
        trust_remote_code = True
        enable_thinking = False

        planner_to_refiner_rounds: list[list[torch.Tensor]] = []
        refiner_to_solver_rounds: list[list[torch.Tensor]] = []
        feedback_to_planner: list[torch.Tensor] | None = None

        for round_idx in range(recursion_rounds):
            if round_idx == 0:
                planner_to_refiner = mas.run_planner_latent_stage(
                    model_name_or_path=self.planner_path,
                    questions=questions,
                    agent1_inner_aligner_path=self.planner_adapter,
                    outer_12_path=self.outer_12_path,
                    outer_12_type=self.outer_type,
                    latent_steps=self.latent_steps,
                    batch_size=self.batch_size,
                    device=device,
                    model_dtype=self.model_dtype,
                    outer_dtype=self.outer_dtype,
                    trust_remote_code=trust_remote_code,
                    inner_adapter_type_fallback=self.inner_fallback,
                    enable_thinking=enable_thinking,
                )
            else:
                if feedback_to_planner is None:
                    raise RuntimeError("Missing recursive feedback latents for planner stage")
                planner_to_refiner = mas.run_planner_feedback_latent_stage(
                    model_name_or_path=self.planner_path,
                    questions=questions,
                    feedback_latents=feedback_to_planner,
                    agent1_inner_aligner_path=self.planner_adapter,
                    outer_12_path=self.outer_12_path,
                    outer_12_type=self.outer_type,
                    latent_steps=self.latent_steps,
                    batch_size=self.batch_size,
                    device=device,
                    model_dtype=self.model_dtype,
                    outer_dtype=self.outer_dtype,
                    trust_remote_code=trust_remote_code,
                    inner_adapter_type_fallback=self.inner_fallback,
                    enable_thinking=enable_thinking,
                )

            planner_to_refiner_rounds.append(list(planner_to_refiner))

            refiner_to_solver = mas.run_refiner_latent_stage(
                model_name_or_path=self.critic_path,
                questions=questions,
                planner_latents=planner_to_refiner,
                agent2_inner_aligner_path=self.critic_adapter,
                outer_23_path=self.outer_23_path,
                outer_23_type=self.outer_type,
                latent_steps=self.latent_steps,
                batch_size=self.batch_size,
                device=device,
                model_dtype=self.model_dtype,
                outer_dtype=self.outer_dtype,
                trust_remote_code=trust_remote_code,
                inner_adapter_type_fallback=self.inner_fallback,
                enable_thinking=enable_thinking,
            )
            refiner_to_solver_rounds.append(list(refiner_to_solver))

            if round_idx < recursion_rounds - 1:
                feedback_to_planner = mas.run_solver_feedback_latent_stage(
                    model_name_or_path=self.solver_path,
                    questions=questions,
                    refiner_latents=refiner_to_solver,
                    agent3_inner_aligner_path=self.solver_adapter,
                    outer_31_path=self.outer_31_path,
                    outer_31_type=self.outer_type,
                    latent_steps=self.latent_steps,
                    batch_size=self.batch_size,
                    device=device,
                    model_dtype=self.model_dtype,
                    outer_dtype=self.outer_dtype,
                    trust_remote_code=trust_remote_code,
                    inner_adapter_type_fallback=self.inner_fallback,
                    enable_thinking=enable_thinking,
                    args=self._args,
                )
                feedback_to_planner = list(feedback_to_planner)

        final_refiner_to_solver = refiner_to_solver_rounds[-1]
        solver_outputs = mas.run_solver_latent_stage(
            model_name_or_path=self.solver_path,
            questions=questions,
            refiner_latents=final_refiner_to_solver,
            args=self._args,
            batch_size=self.batch_size,
            max_new_tokens=self.max_new_tokens,
            do_sample=True,
            temperature=self.temperature,
            top_p=self.top_p,
            device=device,
            dtype=self.model_dtype,
            trust_remote_code=trust_remote_code,
            enable_thinking=enable_thinking,
        )

        answer = solver_outputs[0] if solver_outputs else ""
        latency_ms = int((time.perf_counter() - started) * 1000)
        return InferResult(
            answer=answer,
            latency_ms=latency_ms,
            style=self.style,
            recursion_rounds=recursion_rounds,
        )


def load_runner(
    style: str | None = None,
    dataset: str | None = None,
    device: str | None = None,
) -> RecursiveMASRunner:
    return RecursiveMASRunner(
        style=style or os.getenv("RECURSIVEMAS_STYLE", "sequential_light"),
        dataset=dataset or os.getenv("RECURSIVEMAS_DATASET", "math500"),
        device=device,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("question", nargs="?", default="What is 7 * 8?")
    parser.add_argument("--style", default=os.getenv("RECURSIVEMAS_STYLE", "sequential_light"))
    parser.add_argument("--dataset", default=os.getenv("RECURSIVEMAS_DATASET", "math500"))
    parser.add_argument("--device", default=os.getenv("RECURSIVEMAS_DEVICE"))
    parser.add_argument("--recursion-rounds", type=int, default=1)
    args = parser.parse_args()

    runner = load_runner(style=args.style, dataset=args.dataset, device=args.device)
    result = runner.run(args.question, recursion_rounds=args.recursion_rounds)
    print(result.answer)
    print(f"[latency_ms={result.latency_ms}]", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
