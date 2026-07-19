#!/usr/bin/env bash
# Map nightburst metabolism artifacts → MIDI motif (HSP-adjacent demo).
# Does not require HSP daemon; optional --play uses `hsp ping` or aplay if present.
#
#   bash scripts/hsp-metabolism-sonify.sh
#   bash scripts/hsp-metabolism-sonify.sh --play
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT_DIR="$DATA/hsp-metabolism"
PLAY=0
for a in "$@"; do
  case "$a" in
    --play) PLAY=1 ;;
  esac
done
mkdir -p "$OUT_DIR"
export DATA OUT_DIR PLAY ROOT

python3 - <<'PY'
import json, os, struct, time, wave
from pathlib import Path

data = Path(os.environ["DATA"])
out = Path(os.environ["OUT_DIR"])
runs = data / "scheduler-runs"

# Motif map: metabolism night as short phrases (MIDI note, velocity, duration_ms).
MOTIFS = {
    "distill": (60, 90, 180),   # C4
    "promote": (62, 85, 160),   # D4
    "embed": (64, 80, 160),     # E4
    "dream": (57, 70, 420),     # A3 long
    "spark": (67, 95, 140),     # G4
    "arena": (72, 100, 220),    # C5
    "faithfulness": (76, 90, 200),  # E5
    "organ": (55, 60, 90),      # G3 tick
    "watchdog_ok": (64, 50, 100),
    "watchdog_stale": (41, 110, 500),  # F2 warning
    "idle": (48, 30, 80),
}


def load(p: Path):
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return None


def events_from_artifacts():
    ev = []
    for job in ("distill", "promote", "embed", "dream", "spark"):
        r = load(runs / f"latest-{job}.json")
        if not r:
            continue
        note, vel, dur = MOTIFS[job]
        if not r.get("ok", True):
            vel = max(40, vel - 30)
            note = max(36, note - 12)
        ev.append({"kind": job, "ok": bool(r.get("ok", True)), "note": note, "vel": vel, "dur_ms": dur})

    wd = load(runs / "latest-watchdog.json") or {}
    if wd:
        key = "watchdog_stale" if wd.get("stale") else "watchdog_ok"
        n, v, d = MOTIFS[key]
        ev.append({"kind": key, "ok": not wd.get("stale"), "note": n, "vel": v, "dur_ms": d})

    arena = load(data / "arena" / "latest.json")
    if arena:
        n, v, d = MOTIFS["arena"]
        # Scale duration by z if present
        z = float(arena.get("z") or 0.5)
        d = int(d * (0.6 + 0.8 * max(0.0, min(1.0, z))))
        ev.append({"kind": "arena", "ok": True, "note": n, "vel": v, "dur_ms": d, "z": z})

    faith = load(data / "faithfulness" / "latest.json")
    if faith is not None:
        n, v, d = MOTIFS["faithfulness"]
        ok = bool(faith.get("ok", False))
        if not ok:
            n, v = 46, 100
        ev.append({"kind": "faithfulness", "ok": ok, "note": n, "vel": v, "dur_ms": d})

    organs = load(data / "organ-trace" / "latest.json") or {}
    fired = int(organs.get("ok_count") or organs.get("organs_fired") or 0)
    if fired:
        n, v, d = MOTIFS["organ"]
        for i in range(min(fired, 8)):
            ev.append({"kind": "organ", "ok": True, "note": n + (i % 3), "vel": v, "dur_ms": d})

    # Unpark Wave 2.3 file-drop motifs (hsp-emit-demo) → short phrase
    emit = load(data / "hsp-emit" / "latest-event.json")
    if emit and emit.get("motif"):
        intensity = float(emit.get("intensity") or 0.35)
        n, v, d = MOTIFS.get("distill", MOTIFS["idle"])
        if emit.get("motif") == "distill_tick":
            n, v, d = MOTIFS["distill"]
        v = max(40, min(127, int(v * (0.7 + 0.6 * intensity))))
        ev.append(
            {
                "kind": f"hsp_emit:{emit.get('motif')}",
                "ok": True,
                "note": n,
                "vel": v,
                "dur_ms": d,
                "source": "hsp-emit",
            }
        )

    if not ev:
        n, v, d = MOTIFS["idle"]
        ev.append({"kind": "idle", "ok": True, "note": n, "vel": v, "dur_ms": d})
    return ev


def write_midi(path: Path, events):
    # Minimal Type-0 SMF, one track, 480 TPQ, tempo 120.
    tpq = 480
    tempo = 500_000  # 120 bpm

    def vlq(n: int) -> bytes:
        out = [n & 0x7F]
        n >>= 7
        while n:
            out.append(0x80 | (n & 0x7F))
            n >>= 7
        return bytes(reversed(out))

    track = bytearray()
    track += b"\x00\xff\x51\x03" + struct.pack(">I", tempo)[1:]
    ticks_per_ms = tpq / (tempo / 1000.0)  # at 120bpm: 480 / 500 = 0.96

    for ev in events:
        dur_ticks = max(1, int(ev["dur_ms"] * ticks_per_ms))
        gap = max(1, dur_ticks // 4)
        note = int(ev["note"]) & 0x7F
        vel = int(ev["vel"]) & 0x7F
        track += vlq(gap) + bytes([0x90, note, vel])
        track += vlq(dur_ticks) + bytes([0x80, note, 0x00])

    track += b"\x00\xff\x2f\x00"
    hdr = b"MThd" + struct.pack(">IHHH", 6, 0, 1, tpq)
    trk = b"MTrk" + struct.pack(">I", len(track)) + track
    path.write_bytes(hdr + trk)


def write_preview_wav(path: Path, events, sr=22050):
    # Tiny square-ish preview so --play works without FluidSynth.
    samples = []
    for ev in events:
        freq = 440.0 * (2 ** ((ev["note"] - 69) / 12.0))
        n = max(1, int(sr * ev["dur_ms"] / 1000.0))
        amp = 0.15 * (ev["vel"] / 127.0)
        for i in range(n):
            t = i / sr
            # soft square
            s = amp if (int(t * freq * 2) % 2 == 0) else -amp
            # short attack/release
            env = min(1.0, i / (0.01 * sr), (n - i) / (0.02 * sr))
            samples.append(int(max(-1, min(1, s * env)) * 32767))
        # gap
        samples.extend([0] * int(sr * 0.04))
    with wave.open(str(path), "w") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(sr)
        w.writeframes(b"".join(struct.pack("<h", s) for s in samples))


events = events_from_artifacts()
mid = out / "latest.mid"
wav = out / "latest.wav"
meta = out / "latest.json"
md = out / "latest.md"

write_midi(mid, events)
write_preview_wav(wav, events)

payload = {
    "schema": "gzmo.hsp.metabolism/v1",
    "ts": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    "events": events,
    "midi": str(mid),
    "wav": str(wav),
    "note": "HSP-adjacent motif from metabolism artifacts; daemon optional. Map s2.",
}
meta.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

lines = [
    "# HSP metabolism motif",
    "",
    f"Generated: {payload['ts']}",
    "",
    "| kind | note | vel | dur_ms | ok |",
    "|------|------|-----|--------|----|",
]
for e in events:
    lines.append(f"| {e['kind']} | {e['note']} | {e['vel']} | {e['dur_ms']} | {e.get('ok')} |")
lines += ["", f"MIDI: `{mid}`", f"WAV preview: `{wav}`", ""]
md.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(json.dumps({"ok": True, "events": len(events), "midi": str(mid), "wav": str(wav)}, indent=2))
PY

if [[ "$PLAY" -eq 1 ]]; then
  if command -v hsp >/dev/null 2>&1; then
    echo "[*] hsp ping (speaker check between motif and HSP finish-phrase world)"
    hsp ping 2>/dev/null || true
  elif [[ -x "$ROOT/../HSP/run_hsp.sh" ]]; then
    (cd "$ROOT/../HSP" && ./run_hsp.sh ping) 2>/dev/null || true
  fi
  if command -v aplay >/dev/null 2>&1; then
    aplay -q "$OUT_DIR/latest.wav" || true
  elif command -v paplay >/dev/null 2>&1; then
    paplay "$OUT_DIR/latest.wav" || true
  else
    echo "[*] preview WAV at $OUT_DIR/latest.wav (no aplay/paplay)"
  fi
fi
