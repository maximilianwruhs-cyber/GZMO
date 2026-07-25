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
    "distill_tick": (60, 90, 180),
    "spark_flare": (79, 110, 160),   # G5
    "dream_deep": (45, 75, 520),     # A2
    "promote_pin": (69, 100, 240),   # A4
    "serendipity": (71, 88, 200),    # B4
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

    # Living promote pins (theater mirror of craft — does not claim Brain Feed)
    promo = data / "beat-gate" / "promotions"
    for loop_name, motif_key in (("knowledge", "promote_pin"), ("cognition", "spark_flare")):
        pin = load(promo / f"living-applied-{loop_name}.json")
        if not pin or pin.get("loop") != loop_name:
            continue
        n, v, d = MOTIFS[motif_key]
        ev.append({"kind": f"promote_pin:{loop_name}", "ok": True, "note": n, "vel": v, "dur_ms": d})

    ser = load(data / "serendipity" / "weekly-check-latest.json") or {}
    if ser.get("ok"):
        n, v, d = MOTIFS["serendipity"]
        week = int(ser.get("week_applies") or 0)
        v = max(40, min(127, v + week * 8))
        ev.append({"kind": "serendipity", "ok": True, "note": n, "vel": v, "dur_ms": d, "week_applies": week})

    # Unpark Wave 2.3 file-drop motifs (hsp-emit-demo) → short phrase
    emit = load(data / "hsp-emit" / "latest-event.json")
    if emit and emit.get("motif"):
        intensity = float(emit.get("intensity") or 0.35)
        motif = str(emit.get("motif"))
        n, v, d = MOTIFS.get(motif, MOTIFS["distill"])
        v = max(40, min(127, int(v * (0.7 + 0.6 * intensity))))
        d = int(d * (0.8 + 0.5 * intensity))
        ev.append(
            {
                "kind": f"hsp_emit:{motif}",
                "ok": True,
                "note": n,
                "vel": v,
                "dur_ms": d,
                "source": "hsp-emit",
                "intensity": intensity,
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
    # Soft sine preview so --play is listenable without FluidSynth.
    import math
    samples = []
    for ev in events:
        freq = 440.0 * (2 ** ((ev["note"] - 69) / 12.0))
        n = max(1, int(sr * ev["dur_ms"] / 1000.0))
        amp = 0.22 * (ev["vel"] / 127.0)
        for i in range(n):
            t = i / sr
            s = amp * math.sin(2 * math.pi * freq * t)
            # light 2nd harmonic
            s += 0.08 * amp * math.sin(4 * math.pi * freq * t)
            env = min(1.0, i / (0.012 * sr), (n - i) / (0.03 * sr + 1))
            samples.append(int(max(-1, min(1, s * env)) * 32767))
        samples.extend([0] * int(sr * 0.045))
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
  # Preflight: silent "success" is usually a near-zero PipeWire sink, not WAV/ALSA format.
  if command -v wpctl >/dev/null 2>&1; then
    sink_line="$(wpctl status 2>/dev/null | awk '/Sinks:/{s=1;next} /Sources:/{s=0} s && /\*/ {print; exit}')"
    vol_raw="$(wpctl get-volume @DEFAULT_AUDIO_SINK@ 2>/dev/null | awk '{print $2}')"
    vol="${vol_raw:-0}"
    echo "[*] PipeWire default sink: ${sink_line:-unknown} (volume ${vol})"
    if awk -v v="$vol" 'BEGIN{exit !(v+0 < 0.15)}'; then
      echo "[!] default sink volume < 0.15 — bumping to 0.40 so motif is audible"
      wpctl set-volume @DEFAULT_AUDIO_SINK@ 0.40 || true
    fi
  else
    echo "[*] wpctl not found — cannot verify sink volume before play"
  fi

  if command -v hsp >/dev/null 2>&1; then
    echo "[*] hsp ping (speaker check between motif and HSP finish-phrase world)"
    hsp ping 2>/dev/null || echo "[!] hsp ping failed (non-fatal)"
  elif [[ -x "$ROOT/../HSP/run_hsp.sh" ]]; then
    (cd "$ROOT/../HSP" && ./run_hsp.sh ping) 2>/dev/null || echo "[!] HSP run_hsp.sh ping failed (non-fatal)"
  fi

  # Prefer PipeWire-facing players; bare aplay already uses ALSA default → PipeWire here.
  played=0
  if command -v pw-play >/dev/null 2>&1; then
    echo "[*] pw-play $OUT_DIR/latest.wav"
    pw-play "$OUT_DIR/latest.wav" && played=1 || echo "[!] pw-play failed"
  elif command -v paplay >/dev/null 2>&1; then
    echo "[*] paplay $OUT_DIR/latest.wav"
    paplay "$OUT_DIR/latest.wav" && played=1 || echo "[!] paplay failed"
  elif command -v aplay >/dev/null 2>&1; then
    echo "[*] aplay -D default $OUT_DIR/latest.wav (PipeWire ALSA plugin)"
    aplay -D default -q "$OUT_DIR/latest.wav" && played=1 || echo "[!] aplay failed"
  fi
  if [[ "$played" -eq 0 ]]; then
    echo "[!] no audible play — WAV at $OUT_DIR/latest.wav (install pipewire-pulse / alsa-utils)"
  fi
fi
