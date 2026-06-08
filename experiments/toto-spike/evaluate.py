#!/usr/bin/env python3
"""GZMO ρ forecast evaluation: baselines vs Toto-2.0-4m (zero-shot)."""

from __future__ import annotations

import csv
import json
import math
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable

import numpy as np

ROOT = Path(__file__).resolve().parents[2]
RESULTS = Path(__file__).resolve().parent / "results"
HORIZONS = (32, 64, 96)
EMA_GAMMA = 0.2
TREND_K = 8
SAT_THRESHOLD = 6.0
HOLDOUT_FRAC = 0.20
MIN_TRAIN = 32
CONTEXT_LEN = 128


@dataclass
class Dataset:
    name: str
    label: str  # real | synthetic
    rho_mod: np.ndarray
    rho_velocity_ema: np.ndarray
    rho_mod_delta: np.ndarray
    rho_effective: np.ndarray | None = None


@dataclass
class ForecastResult:
    task: str
    dataset: str
    horizon: int
    model: str
    mase: float | None = None
    crps: float | None = None
    mae: float | None = None
    brier: float | None = None
    auroc: float | None = None
    lead_ticks: float | None = None
    n_eval: int = 0


def load_synapse(path: Path) -> Dataset:
    ticks, rho_mod, v_ema, delta, eff = [], [], [], [], []
    with path.open() as f:
        for line in f:
            ev = json.loads(line)
            if ev.get("event_type") != "chaos.rho_telemetry":
                continue
            d = ev["data"]
            ticks.append(int(d["tick"]))
            rho_mod.append(float(d["rho_mod"]))
            v_ema.append(float(d.get("rho_velocity_ema", 0.0)))
            delta.append(float(d.get("rho_mod_delta", 0.0)))
            eff.append(float(d.get("rho_effective", 28.0 + float(d["rho_mod"]))))
    return Dataset(
        name="synapse_real",
        label="real",
        rho_mod=np.array(rho_mod),
        rho_velocity_ema=np.array(v_ema),
        rho_mod_delta=np.array(delta),
        rho_effective=np.array(eff),
    )


def ema_velocity(delta: np.ndarray, gamma: float = EMA_GAMMA) -> np.ndarray:
    v = np.zeros_like(delta)
    for i, d in enumerate(delta):
        v[i] = (1 - gamma) * (v[i - 1] if i else 0.0) + gamma * d
    return v


def load_lab_csv(path: Path, name: str) -> Dataset:
    ticks, rho_mod, delta = [], [], []
    with path.open() as f:
        reader = csv.DictReader(f)
        for row in reader:
            ticks.append(int(row["tick"]))
            rho_mod.append(float(row["rho_mod"]))
            delta.append(float(row.get("rho_delta", row.get("rho_mod_delta", 0.0))))
    rho_mod_a = np.array(rho_mod)
    delta_a = np.array(delta)
    return Dataset(
        name=name,
        label="synthetic",
        rho_mod=rho_mod_a,
        rho_velocity_ema=ema_velocity(delta_a),
        rho_mod_delta=delta_a,
        rho_effective=28.0 + rho_mod_a,
    )


def mase(y_true: np.ndarray, y_pred: np.ndarray, y_train: np.ndarray) -> float:
    denom = np.mean(np.abs(np.diff(y_train)))
    if denom < 1e-12:
        denom = np.mean(np.abs(y_train)) + 1e-12
    return float(np.mean(np.abs(y_true - y_pred)) / denom)


def crps_quantile(y: float, quantiles: np.ndarray, levels: np.ndarray) -> float:
    """Average pinball loss across quantile levels (= CRPS for discrete quantiles)."""
    losses = []
    for q, alpha in zip(quantiles, levels):
        err = y - q
        losses.append(max(alpha * err, (alpha - 1) * err))
    return float(np.mean(losses))


def auroc_binary(y_true: np.ndarray, y_score: np.ndarray) -> float:
    pos = y_true == 1
    neg = ~pos
    n_pos, n_neg = pos.sum(), neg.sum()
    if n_pos == 0 or n_neg == 0:
        return float("nan")
    ranks = _rankdata(y_score)
    rank_sum_pos = ranks[pos].sum()
    return float((rank_sum_pos - n_pos * (n_pos + 1) / 2) / (n_pos * n_neg))


def _rankdata(x: np.ndarray) -> np.ndarray:
    order = np.argsort(x)
    ranks = np.empty_like(order, dtype=float)
    ranks[order] = np.arange(1, len(x) + 1)
    return ranks


def eval_indices(n: int) -> tuple[int, int]:
    holdout = max(1, int(n * HOLDOUT_FRAC))
    train_end = n - holdout
    return train_end, n


def forecast_persistence(y: np.ndarray, t: int, h: int) -> float:
    return float(y[t])


def forecast_linear_trend(y: np.ndarray, t: int, h: int, k: int = TREND_K) -> float:
    start = max(0, t - k + 1)
    seg = y[start : t + 1]
    if len(seg) < 2:
        return float(y[t])
    x = np.arange(len(seg), dtype=float)
    slope = np.polyfit(x, seg, 1)[0]
    return float(y[t] + h * slope)


def forecast_ema_carry(y: np.ndarray, v: np.ndarray, t: int, h: int) -> float:
    return float(y[t] + h * v[t])


def saturation_label(y: np.ndarray, t: int, h: int, thr: float) -> int:
    end = min(len(y), t + h + 1)
    return int(np.any(y[t + 1 : end] > thr))


def prob_from_quantiles(qs: np.ndarray, levels: np.ndarray, thr: float) -> float:
    """Fraction of quantile levels exceeding threshold."""
    return float(np.mean(qs > thr))


def run_t1_t3(
    ds: Dataset,
    model_name: str,
    forecaster: Callable[[Dataset, int, int], float | tuple[float, np.ndarray | None]],
    quantile_levels: np.ndarray | None = None,
) -> list[ForecastResult]:
    y = ds.rho_mod
    v = ds.rho_velocity_ema
    n = len(y)
    train_end, _ = eval_indices(n)
    results: list[ForecastResult] = []

    for h in HORIZONS:
        y_true_list, y_pred_list = [], []
        y_train = y[:train_end]
        for t in range(max(MIN_TRAIN, train_end - int(n * HOLDOUT_FRAC)), n - h):
            out = forecaster(ds, t, h)
            if isinstance(out, tuple):
                pred, _ = out
            else:
                pred = out
            y_true_list.append(y[t + h])
            y_pred_list.append(pred)

        if not y_true_list:
            continue
        yt = np.array(y_true_list)
        yp = np.array(y_pred_list)
        res = ForecastResult(
            task="T1_rho_mod",
            dataset=ds.name,
            horizon=h,
            model=model_name,
            mase=mase(yt, yp, y_train),
            mae=float(np.mean(np.abs(yt - yp))),
            n_eval=len(yt),
        )
        if quantile_levels is not None:
            crps_vals = []
            for t in range(max(MIN_TRAIN, train_end - int(n * HOLDOUT_FRAC)), n - h):
                out = forecaster(ds, t, h)
                if isinstance(out, tuple) and out[1] is not None:
                    crps_vals.append(crps_quantile(y[t + h], out[1], quantile_levels))
            if crps_vals:
                res.crps = float(np.mean(crps_vals))
        results.append(res)

    # T3: predict rho_velocity_ema
    y3 = ds.rho_velocity_ema
    for h in HORIZONS:
        y_true_list, y_pred_list = [], []
        y_train = y3[:train_end]
        for t in range(max(MIN_TRAIN, train_end - int(n * HOLDOUT_FRAC)), n - h):
            if model_name == "persistence":
                pred = y3[t]
            elif model_name == "linear_trend":
                pred = forecast_linear_trend(y3, t, h)
            elif model_name == "ema_carry_forward":
                pred = y3[t]  # velocity is already smoothed; best naive = hold
            else:
                pred = y3[t]
            y_true_list.append(y3[t + h])
            y_pred_list.append(pred)
        if not y_true_list:
            continue
        yt = np.array(y_true_list)
        yp = np.array(y_pred_list)
        results.append(
            ForecastResult(
                task="T3_rho_velocity_ema",
                dataset=ds.name,
                horizon=h,
                model=model_name,
                mase=mase(yt, yp, y_train),
                mae=float(np.mean(np.abs(yt - yp))),
                n_eval=len(yt),
            )
        )
    return results


def run_t2_saturation(
    ds: Dataset,
    model_name: str,
    scorer: Callable[[Dataset, int, int], float],
) -> list[ForecastResult]:
    y = ds.rho_mod
    n = len(y)
    train_end, _ = eval_indices(n)
    results: list[ForecastResult] = []

    for h in HORIZONS:
        labels, scores = [], []
        for t in range(max(MIN_TRAIN, train_end - int(n * HOLDOUT_FRAC)), n - h):
            lab = saturation_label(y, t, h, SAT_THRESHOLD)
            labels.append(lab)
            scores.append(scorer(ds, t, h))
        if len(labels) < 5:
            results.append(
                ForecastResult(
                    task="T2_saturate_gt6",
                    dataset=ds.name,
                    horizon=h,
                    model=model_name,
                    brier=float("nan"),
                    auroc=float("nan"),
                    n_eval=len(labels),
                )
            )
            continue
        labels_a = np.array(labels)
        scores_a = np.array(scores)
        brier = float(np.mean((scores_a - labels_a) ** 2))
        results.append(
            ForecastResult(
                task="T2_saturate_gt6",
                dataset=ds.name,
                horizon=h,
                model=model_name,
                brier=brier,
                auroc=auroc_binary(labels_a, scores_a),
                n_eval=len(labels),
            )
        )
    return results


def baseline_forecast(ds: Dataset, model: str, t: int, h: int) -> float:
    y, v = ds.rho_mod, ds.rho_velocity_ema
    if model == "persistence":
        return forecast_persistence(y, t, h)
    if model == "linear_trend":
        return forecast_linear_trend(y, t, h)
    if model == "ema_carry_forward":
        return forecast_ema_carry(y, v, t, h)
    raise ValueError(model)


def baseline_saturation_score(ds: Dataset, model: str, t: int, h: int) -> float:
    y, v = ds.rho_mod, ds.rho_velocity_ema
    if model == "persistence":
        return 1.0 if y[t] > SAT_THRESHOLD else 0.0
    if model == "linear_trend":
        pred = forecast_linear_trend(y, t, h)
        return 1.0 if pred > SAT_THRESHOLD else 0.0
    if model == "ema_carry_forward":
        pred = forecast_ema_carry(y, v, t, h)
        return 1.0 if pred > SAT_THRESHOLD else 0.0
    raise ValueError(model)


class TotoForecaster:
    def __init__(self) -> None:
        import torch
        from toto2 import Toto2Model

        self.torch = torch
        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        self.model = Toto2Model.from_pretrained("Datadog/Toto-2.0-4m")
        self.model = self.model.to(self.device).eval()
        self.levels = np.array([0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9])
        self.latency_ms: list[float] = []

    def _context_tensor(self, ds: Dataset, t: int) -> "torch.Tensor":
        start = max(0, t + 1 - CONTEXT_LEN)
        # multivariate: rho_mod, rho_velocity_ema, rho_mod_delta
        cols = [
            ds.rho_mod[start : t + 1],
            ds.rho_velocity_ema[start : t + 1],
            ds.rho_mod_delta[start : t + 1],
        ]
        arr = np.stack(cols, axis=0)  # (n_variates, time)
        if arr.shape[1] < CONTEXT_LEN:
            pad = CONTEXT_LEN - arr.shape[1]
            arr = np.pad(arr, ((0, 0), (pad, 0)), mode="edge")
        t_tensor = self.torch.tensor(arr, dtype=self.torch.float32, device=self.device)
        return t_tensor.unsqueeze(0)  # batch=1

    def forecast(self, ds: Dataset, t: int, h: int) -> tuple[float, np.ndarray]:
        target = self._context_tensor(ds, t)
        target_mask = self.torch.ones_like(target, dtype=self.torch.bool)
        series_ids = self.torch.zeros(1, target.shape[1], dtype=self.torch.long, device=self.device)
        t0 = time.perf_counter()
        with self.torch.no_grad():
            quantiles = self.model.forecast(
                {"target": target, "target_mask": target_mask, "series_ids": series_ids},
                horizon=h,
                decode_block_size=min(768, h),
                has_missing_values=False,
            )
        self.latency_ms.append((time.perf_counter() - t0) * 1000)
        # quantiles: (9, batch, n_variates, horizon) — take rho_mod variate 0, last step
        q = quantiles[:, 0, 0, -1].cpu().numpy()
        p50_idx = 4
        return float(q[p50_idx]), q


def run_all_baselines(datasets: list[Dataset]) -> list[ForecastResult]:
    out: list[ForecastResult] = []
    for ds in datasets:
        for model in ("persistence", "linear_trend", "ema_carry_forward"):
            fc = lambda d, t, h, m=model: baseline_forecast(d, m, t, h)
            out.extend(
                run_t1_t3(
                    ds,
                    model,
                    lambda d, t, h, m=model: baseline_forecast(d, m, t, h),
                )
            )
            out.extend(
                run_t2_saturation(
                    ds,
                    model,
                    lambda d, t, h, m=model: baseline_saturation_score(d, m, t, h),
                )
            )
    return out


def run_toto(datasets: list[Dataset], toto: TotoForecaster) -> list[ForecastResult]:
    out: list[ForecastResult] = []

    def toto_fc(ds: Dataset, t: int, h: int) -> tuple[float, np.ndarray | None]:
        p50, qs = toto.forecast(ds, t, h)
        return p50, qs

    for ds in datasets:
        out.extend(run_t1_t3(ds, "toto_4m", toto_fc, toto.levels))
        out.extend(
            run_t2_saturation(
                ds,
                "toto_4m",
                lambda d, t, h: prob_from_quantiles(
                    toto.forecast(d, t, h)[1], toto.levels, SAT_THRESHOLD
                ),
            )
        )
    return out


def write_tsv(path: Path, rows: list[ForecastResult]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fields = [
        "task",
        "dataset",
        "horizon",
        "model",
        "mase",
        "crps",
        "mae",
        "brier",
        "auroc",
        "n_eval",
    ]
    with path.open("w") as f:
        f.write("\t".join(fields) + "\n")
        for r in rows:
            vals = []
            for field in fields:
                v = getattr(r, field)
                if v is None or (isinstance(v, float) and math.isnan(v)):
                    vals.append("")
                elif isinstance(v, float):
                    vals.append(f"{v:.6f}")
                else:
                    vals.append(str(v))
            f.write("\t".join(vals) + "\n")


def best_baseline_mase(rows: list[ForecastResult], task: str, ds: str, h: int) -> float:
    baselines = [r for r in rows if r.task == task and r.dataset == ds and r.horizon == h and r.model != "toto_4m"]
    if not baselines:
        return float("nan")
    return min(r.mase for r in baselines if r.mase is not None)


def main() -> None:
    synapse_path = ROOT / "data" / "Synapse" / "events.jsonl"
    lab_fast = Path.home() / "Projects/chaos-breathing-lab/output/matrix/active_story_30s_linear_decay_fast.csv"
    lab_baseline = Path.home() / "Projects/chaos-breathing-lab/output/matrix/active_story_30s_baseline.csv"

    datasets: list[Dataset] = []
    if synapse_path.exists():
        datasets.append(load_synapse(synapse_path))
    if lab_fast.exists():
        datasets.append(load_lab_csv(lab_fast, "lab_linear_decay_fast"))
    if lab_baseline.exists():
        datasets.append(load_lab_csv(lab_baseline, "lab_baseline_saturation"))

    print(f"Loaded {len(datasets)} datasets")
    for ds in datasets:
        print(f"  {ds.name} ({ds.label}): n={len(ds.rho_mod)} rho_mod=[{ds.rho_mod.min():.3f},{ds.rho_mod.max():.3f}]")

    baseline_rows = run_all_baselines(datasets)
    write_tsv(RESULTS / "baseline_results.tsv", baseline_rows)
    print(f"Wrote {RESULTS / 'baseline_results.tsv'}")

    toto_meta = {"loaded": False, "error": None, "latency_ms_p50": None, "latency_ms_p90": None}
    all_rows = list(baseline_rows)
    try:
        toto = TotoForecaster()
        toto_meta["loaded"] = True
        if toto.latency_ms:
            lat = np.array(toto.latency_ms)
        else:
            # warmup
            toto.forecast(datasets[0], min(64, len(datasets[0].rho_mod) - 2), 32)
            lat = np.array(toto.latency_ms)
        toto_meta["latency_ms_p50"] = float(np.percentile(lat, 50))
        toto_meta["latency_ms_p90"] = float(np.percentile(lat, 90))
        toto_rows = run_toto(datasets, toto)
        write_tsv(RESULTS / "toto_results.tsv", toto_rows)
        all_rows.extend(toto_rows)
        print(f"Wrote {RESULTS / 'toto_results.tsv'}")
    except Exception as e:
        toto_meta["error"] = str(e)
        print(f"Toto load/run failed: {e}")

    write_tsv(RESULTS / "all_results.tsv", all_rows)
    meta_path = RESULTS / "run_meta.json"
    meta_path.write_text(json.dumps(toto_meta, indent=2))
    print(f"Wrote {meta_path}")


if __name__ == "__main__":
    main()
