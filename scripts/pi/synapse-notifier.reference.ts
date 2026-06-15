/**
 * Synapse Notifier — Pi → GZMO Event Bridge
 *
 * Writes Pi lifecycle events into the GZMO Synapse JSONL bus at
 * the path configured in `settings.json.synapseNotifier.busPath`.
 *
 * Schema matches gzmo-core/src/synapse.rs:
 *   { id, event_type, source, timestamp, correlation_id?, reply_to?, data? }
 *
 * event_type values are snake_case (Rust `#[serde(rename_all = "snake_case")]`).
 *
 * Events emitted:
 *   - session_start  / session_end   on session lifecycle
 *   - quest_complete on every turn_end (with usage + tool result counts)
 *   - dream_complete / spark_complete / ingest_complete / distill_complete / wiki_complete / health_tick
 *     on GZMO tool invocation (gzmo_dream, gzmo_spark, gzmo_wiki, etc.)
 */

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { execFileSync, spawn } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { randomUUID } from "node:crypto";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function expandHome(filePath: string): string {
  if (filePath.startsWith("~/")) {
    return path.join(process.env.HOME || "", filePath.slice(2));
  }
  return filePath;
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

interface NotifierConfig {
  busPath: string;
  enabled: boolean;
  debug: boolean;
  distillOnSessionEnd: boolean;
}

const GZMO_ROOT =
  process.env.GZMO_ROOT ??
  path.join(process.env.HOME || "", "Projects", "_foundation-audit", "survey_GZMO");

function resolveGzmoBin(): string {
  return process.env.GZMO_BIN ?? path.join(GZMO_ROOT, "target", "release", "gzmo");
}

function piDistillStatePath(): string {
  return path.join(GZMO_ROOT, "data", "synapse-pi-distill.state.json");
}

function alreadyDistilledPiSession(sessionFile: string): boolean {
  try {
    const raw = fs.readFileSync(piDistillStatePath(), "utf-8");
    const state = JSON.parse(raw) as { distilled_paths?: string[] };
    return (state.distilled_paths ?? []).includes(sessionFile);
  } catch {
    return false;
  }
}

function spawnPiSessionDistill(sessionFile: string): void {
  const gzmo = resolveGzmoBin();
  if (!fs.existsSync(gzmo)) {
    debug("distill skip: gzmo binary missing at", gzmo);
    return;
  }
  if (!fs.existsSync(sessionFile)) {
    debug("distill skip: session file missing", sessionFile);
    return;
  }
  if (alreadyDistilledPiSession(sessionFile)) {
    debug("distill skip: already distilled", sessionFile);
    return;
  }
  const env = {
    ...process.env,
    GZMO_CONFIG: process.env.GZMO_CONFIG ?? path.join(GZMO_ROOT, "gzmo.toml"),
  };
  try {
    const child = spawn(gzmo, ["distill", "pi", sessionFile], {
      detached: true,
      stdio: "ignore",
      env,
      cwd: GZMO_ROOT,
    });
    child.unref();
    debug("spawned Pi session distill", sessionFile);
  } catch (err) {
    debug("Pi session distill spawn failed", err);
  }
}

function loadConfig(): NotifierConfig {
  try {
    const settingsPath = path.join(process.env.HOME || "", ".pi", "agent", "settings.json");
    const raw = fs.readFileSync(settingsPath, "utf-8");
    const settings = JSON.parse(raw);
    const synapse = (settings.synapseNotifier as {
      busPath?: string;
      enabled?: boolean;
      debug?: boolean;
      distillOnSessionEnd?: boolean;
    }) ?? {};
    let busPath = synapse.busPath ?? defaultBusPath();
    busPath = expandHome(busPath);
    return {
      busPath,
      enabled: synapse.enabled ?? true,
      debug: synapse.debug ?? false,
      distillOnSessionEnd: synapse.distillOnSessionEnd ?? true,
    };
  } catch {
    return {
      busPath: expandHome(defaultBusPath()),
      enabled: true,
      debug: false,
      distillOnSessionEnd: true,
    };
  }
}

function defaultBusPath(): string {
  const home = process.env.HOME || "";
  const candidates = [
    // survey_GZMO daemon log — this is where the Rust bus writes by default
    path.join(home, "Projects", "_foundation-audit", "survey_GZMO", "data", "Synapse", "events.jsonl"),
    path.join(home, "Projects", "gzmo-rebuild", "data", "Synapse", "events.jsonl"),
    path.join(home, ".gzmo", "Synapse", "events.jsonl"),
    path.join(home, "Projects", "gzmo", "data", "Synapse", "events.jsonl"),
  ];
  for (const c of candidates) {
    if (fs.existsSync(c)) return c;
  }
  return candidates[0];
}

const cfg = loadConfig();

function debug(...args: unknown[]): void {
  if (cfg.debug) console.error("[synapse-notifier]", ...args);
}

// ---------------------------------------------------------------------------
// Synapse event type names — snake_case to match Rust #[serde(rename_all = "snake_case")]
// ---------------------------------------------------------------------------

const EVT = {
  sessionStart:  "session_start",
  sessionEnd:    "session_end",
  questComplete: "quest_complete",
  questFail:     "quest_fail",
  dreamComplete: "dream_complete",
  sparkComplete: "spark_complete",
  ingestComplete:"ingest_complete",
  distillComplete:"distill_complete",
  wikiComplete:  "wiki_complete",
  healthTick:    "health_tick",
  healthFail:    "health_fail",
  mentorTeach:   "mentor_teach",
  mentorLearnStart: "mentor_learn_start",
  mentorLearnEnd:   "mentor_learn_end",
  topicShiftDistill: "topic_shift_distill",
  skillInvoke: "skill.invoke",
  skillComplete: "skill.complete",
  skillError: "skill.error",
} as const;

type EventType = (typeof EVT)[keyof typeof EVT] | string;

interface SynapseEvent {
  id: string;
  event_type: EventType;
  source: "pi_agent";
  timestamp: string;
  correlation_id?: string;
  reply_to?: string;
  data?: Record<string, unknown>;
}

let currentSessionId: string | null = null;
const pendingSkillInvokes = new Map<string, { skill: string; startedAt: number }>();

// ---------------------------------------------------------------------------
// File locking — same advisory lock as Rust SynapseBus (flock on *.lock)
// ---------------------------------------------------------------------------

function lockPathFor(busPath: string): string {
  const base = path.basename(busPath);
  return path.join(path.dirname(busPath), `${base}.lock`);
}

function shSingleQuote(s: string): string {
  return `'${s.replace(/'/g, `'\\''`)}'`;
}

/** Append under `flock -x` (compatible with gzmo-core fs2). */
function appendEventLocked(busPath: string, line: string): void {
  const dir = path.dirname(busPath);
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }

  const lockPath = lockPathFor(busPath);
  if (!fs.existsSync(lockPath)) {
    fs.writeFileSync(lockPath, "\n", { flag: "a" });
  }

  const payload = line.endsWith("\n") ? line : `${line}\n`;
  const tmp = path.join(dir, `.synapse-pi-${process.pid}-${Date.now()}.line`);
  fs.writeFileSync(tmp, payload, "utf8");

  const cmd = `cat ${shSingleQuote(tmp)} >> ${shSingleQuote(busPath)} && rm -f ${shSingleQuote(tmp)}`;
  try {
    execFileSync("flock", ["-x", lockPath, "bash", "-lc", cmd], {
      stdio: "pipe",
      timeout: 15_000,
    });
  } catch (err: unknown) {
    try {
      fs.unlinkSync(tmp);
    } catch {
      /* ignore */
    }
    try {
      fs.appendFileSync(busPath, payload, { flag: "a" });
    } catch (appendErr: unknown) {
      const msg = appendErr instanceof Error ? appendErr.message : String(appendErr);
      const flockMsg = err instanceof Error ? err.message : String(err);
      throw new Error(`synapse append failed (flock: ${flockMsg}; direct: ${msg})`);
    }
  }
}

// ---------------------------------------------------------------------------
// Append to JSONL
// ---------------------------------------------------------------------------

function appendEvent(event: SynapseEvent): void {
  appendEventLocked(cfg.busPath, JSON.stringify(event));
}

function withSessionId(data?: Record<string, unknown>): Record<string, unknown> | undefined {
  if (!currentSessionId) return data;
  return { ...(data ?? {}), session_id: currentSessionId };
}

function emit(
  event_type: string,
  data?: Record<string, unknown>,
  opts?: { reply_to?: string; correlation_id?: string },
): void {
  const event: SynapseEvent = {
    id: randomUUID(),
    event_type,
    source: "pi_agent",
    timestamp: new Date().toISOString(),
    correlation_id: opts?.correlation_id ?? currentSessionId ?? undefined,
    reply_to: opts?.reply_to,
    data: withSessionId(data),
  };
  try {
    appendEvent(event);
    debug(`${event_type} → ${cfg.busPath}`);
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err);
    console.error("[synapse-notifier] append failed:", msg);
  }
}

// ---------------------------------------------------------------------------
// Topic-Shift Embedding Distill (P1-B)
// ---------------------------------------------------------------------------

let baselineEmbedding: number[] | null = null;
let lastDistillTurnIndex = -100;
let lastDistillTimestamp = 0;

interface PiMessageRow {
  type: string;
  message?: {
    role: string;
    content: Array<{ type: string; text?: string }>;
  };
}

function getUserMessagesFromSession(sessionPath: string): string[] {
  try {
    if (!fs.existsSync(sessionPath)) return [];
    const content = fs.readFileSync(sessionPath, "utf-8");
    const lines = content.split(/\r?\n/);
    const userMsgs: string[] = [];
    for (const line of lines) {
      if (!line.trim()) continue;
      const row = JSON.parse(line) as PiMessageRow;
      if (row.type === "message" && row.message?.role === "user") {
        const texts = row.message.content
          .filter((c: any) => c.type === "text" && c.text)
          .map((c: any) => c.text);
        const text = texts.join("\n").trim();
        if (text) {
          userMsgs.push(text);
        }
      }
    }
    return userMsgs;
  } catch (err) {
    debug("Error reading session messages:", err);
    return [];
  }
}

function parseGzmoTomlConfig(): {
  topicShiftEnabled: boolean;
  topicShiftThreshold: number;
  embedUrl: string;
  embedModel: string;
} {
  const defaults = {
    topicShiftEnabled: false,
    topicShiftThreshold: 0.35,
    embedUrl: "http://192.168.31.110:8081/v1",
    embedModel: "gzmo-embed",
  };
  try {
    const tomlPath = path.join(GZMO_ROOT, "gzmo.toml");
    if (!fs.existsSync(tomlPath)) return defaults;
    const content = fs.readFileSync(tomlPath, "utf-8");
    
    let currentSection = "";
    let topicShiftEnabled = defaults.topicShiftEnabled;
    let topicShiftThreshold = defaults.topicShiftThreshold;
    let embedUrl = defaults.embedUrl;
    let embedModel = defaults.embedModel;

    const lines = content.split(/\r?\n/);
    for (const line of lines) {
      const trimmed = line.trim();
      if (trimmed.startsWith("#") || trimmed === "") continue;
      if (trimmed.startsWith("[") && trimmed.endsWith("]")) {
        currentSection = trimmed.slice(1, -1).trim();
        continue;
      }
      const parts = trimmed.split("=");
      if (parts.length >= 2) {
        const key = parts[0].trim();
        const rawVal = parts.slice(1).join("=").trim();
        let val = rawVal;
        if ((val.startsWith('"') && val.endsWith('"')) || (val.startsWith("'") && val.endsWith("'"))) {
          val = val.slice(1, -1);
        }
        
        if (currentSection === "session_distill") {
          if (key === "topic_shift_enabled") {
            topicShiftEnabled = val === "true";
          } else if (key === "topic_shift_threshold") {
            topicShiftThreshold = parseFloat(val) || defaults.topicShiftThreshold;
          }
        } else if (currentSection === "embeddings") {
          if (key === "url") {
            embedUrl = val;
          } else if (key === "model") {
            embedModel = val;
          }
        }
      }
    }
    return { topicShiftEnabled, topicShiftThreshold, embedUrl, embedModel };
  } catch (err) {
    debug("Failed to parse gzmo.toml:", err);
    return defaults;
  }
}

async function getEmbedding(url: string, model: string, text: string): Promise<number[] | null> {
  try {
    const res = await fetch(url.endsWith("/embeddings") ? url : `${url}/embeddings`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ input: text, model }),
    });
    if (!res.ok) {
      debug("Embedding request failed:", res.statusText);
      return null;
    }
    const data = await res.json() as { data: Array<{ embedding: number[] }> };
    if (data && data.data && data.data[0] && data.data[0].embedding) {
      return data.data[0].embedding;
    }
    return null;
  } catch (err) {
    debug("Embedding call failed:", err);
    return null;
  }
}

function cosineDistance(a: number[], b: number[]): number {
  if (a.length !== b.length) return 1.0;
  let dotProduct = 0;
  let normA = 0;
  let normB = 0;
  for (let i = 0; i < a.length; i++) {
    dotProduct += a[i] * b[i];
    normA += a[i] * a[i];
    normB += b[i] * b[i];
  }
  if (normA === 0 || normB === 0) return 1.0;
  const similarity = dotProduct / (Math.sqrt(normA) * Math.sqrt(normB));
  return 1.0 - similarity;
}

function spawnPiSessionDistillRange(sessionFile: string, startTurn: number, maxTurns: number): void {
  const gzmo = resolveGzmoBin();
  if (!fs.existsSync(gzmo)) {
    debug("distill skip: gzmo binary missing at", gzmo);
    return;
  }
  if (!fs.existsSync(sessionFile)) {
    debug("distill skip: session file missing", sessionFile);
    return;
  }
  const env = {
    ...process.env,
    GZMO_CONFIG: process.env.GZMO_CONFIG ?? path.join(GZMO_ROOT, "gzmo.toml"),
  };
  try {
    const child = spawn(gzmo, ["distill", "pi", sessionFile, "--from-turn", String(startTurn), "--max-turns", String(maxTurns)], {
      detached: true,
      stdio: "ignore",
      env,
      cwd: GZMO_ROOT,
    });
    child.unref();
    debug("spawned Pi session distill range", sessionFile, "startTurn:", startTurn, "maxTurns:", maxTurns);
  } catch (err) {
    debug("Pi session distill range spawn failed", err);
  }
}

async function checkTopicShift(event: any, ctx: any) {
  const tomlCfg = parseGzmoTomlConfig();
  if (!tomlCfg.topicShiftEnabled) {
    return;
  }

  const sessionFile = ctx.sessionManager.getSessionFile();
  if (!sessionFile) {
    return;
  }

  const userMsgs = getUserMessagesFromSession(sessionFile);
  if (userMsgs.length < 2) {
    return;
  }

  const now = Date.now();
  if (now - lastDistillTimestamp < 600000 || event.turnIndex - lastDistillTurnIndex < 3) {
    return;
  }

  const baselineIdx = lastDistillTurnIndex === -100 ? 0 : lastDistillTurnIndex + 1;
  if (baselineIdx >= userMsgs.length) {
    return;
  }
  const baselineText = userMsgs[baselineIdx];
  if (baselineText.length < 100) {
    return;
  }

  const windowSize = 3;
  const startIdx = Math.max(baselineIdx + 1, userMsgs.length - windowSize);
  const windowMsgs = userMsgs.slice(startIdx);
  const windowText = windowMsgs.join("\n").trim();
  if (windowText.length < 200) {
    return;
  }

  debug("Checking topic shift. Baseline:", baselineText.slice(0, 50), "Window:", windowText.slice(0, 50));

  const baseEmbed = baselineEmbedding ?? await getEmbedding(tomlCfg.embedUrl, tomlCfg.embedModel, baselineText);
  if (!baseEmbed) {
    return;
  }
  baselineEmbedding = baseEmbed;

  const winEmbed = await getEmbedding(tomlCfg.embedUrl, tomlCfg.embedModel, windowText);
  if (!winEmbed) {
    return;
  }

  const dist = cosineDistance(baseEmbed, winEmbed);
  debug(`Cosine distance computed: ${dist.toFixed(4)} (threshold: ${tomlCfg.topicShiftThreshold})`);

  if (dist > tomlCfg.topicShiftThreshold) {
    debug(`[topic-shift] Triggering mid-session distill. Distance: ${dist.toFixed(4)}`);
    
    const startTurn = lastDistillTurnIndex === -100 ? 0 : lastDistillTurnIndex + 1;
    const maxTurns = event.turnIndex - startTurn + 1;
    
    spawnPiSessionDistillRange(sessionFile, startTurn, maxTurns);
    
    emit("topic_shift_distill", {
      sessionFile,
      distance: dist,
      threshold: tomlCfg.topicShiftThreshold,
      startTurn,
      maxTurns,
    });

    lastDistillTurnIndex = event.turnIndex;
    lastDistillTimestamp = now;
    baselineEmbedding = winEmbed;
  }
}

// ---------------------------------------------------------------------------
// Extension entry point
// ---------------------------------------------------------------------------

export default function (pi: ExtensionAPI): void {
  if (!cfg.enabled) {
    debug("disabled via config");
    return;
  }

  // --- Session lifecycle ---

  pi.on("session_start", async (event, _ctx) => {
    currentSessionId = randomUUID();
    emit(EVT.sessionStart, { reason: event.reason, session_id: currentSessionId });
  });

  pi.on("session_shutdown", async (event, _ctx) => {
    const target = (event as { targetSessionFile?: string }).targetSessionFile;
    emit(EVT.sessionEnd, { reason: event.reason, targetSessionFile: target });
    if (cfg.distillOnSessionEnd && target) {
      spawnPiSessionDistill(target);
    }
    currentSessionId = null;
  });

  // --- GZMO tool invocation → engine event ---

  const TOOL_EVENT_MAP: Record<string, EventType> = {
    gzmo_dream:    EVT.dreamComplete,
    gzmo_spark:    EVT.sparkComplete,
    gzmo_ingest:   EVT.ingestComplete,
    gzmo_distill:     EVT.distillComplete,
    gzmo_distill_pi:  EVT.distillComplete,
    gzmo_wiki:     EVT.wikiComplete,
    gzmo_health:   EVT.healthTick,
    gzmo_mentor_ping:   EVT.healthTick,
    gzmo_mentor_status: EVT.healthTick,
    gzmo_mentor_reflect: EVT.mentorTeach,
    gzmo_mentor_teach:  EVT.mentorTeach,
    gzmo_mentor_learn_start: EVT.mentorLearnStart,
    gzmo_mentor_learn_end:   EVT.mentorLearnEnd,
  };

  pi.on("tool_call", async (event) => {
    if (event.toolName === "gzmo_chaos") {
      const input = event.input as { command?: string; args?: string } | undefined;
      const skill = input?.command ?? "unknown";
      pendingSkillInvokes.set(event.toolCallId, { skill, startedAt: Date.now() });
      emit(EVT.skillInvoke, {
        skill,
        args: input?.args ?? "",
        toolCallId: event.toolCallId,
      });
    }

    const mapped = TOOL_EVENT_MAP[event.toolName];
    if (mapped) {
      const data: Record<string, unknown> = {
        toolName: event.toolName,
        toolCallId: event.toolCallId,
        args: event.input,
        emitted_by: "pi_tool_echo",
      };
      if (event.toolName === "gzmo_mentor_teach" && event.input && typeof event.input === "object") {
        const msg = (event.input as { message?: string }).message;
        if (msg) data.message = msg.slice(0, 400);
      }
      emit(mapped, data);
    }
  });

  // --- Turn completion (main audit signal) ---

  pi.on("turn_end", async (event, ctx) => {
    const msg = event.message as {
      content?: Array<{ type: string; text?: string }>;
      usage?: {
        inputTokens?: number;
        outputTokens?: number;
        input?: number;
        output?: number;
      };
    };

    const textBlocks = (msg?.content ?? [])
      .filter((c) => c.type === "text")
      .map((c) => c.text ?? "")
      .join("\n")
      .slice(0, 2000);

    const inputTokens = msg?.usage?.inputTokens ?? msg?.usage?.input;
    const outputTokens = msg?.usage?.outputTokens ?? msg?.usage?.output;

    const turnFailed = (event as { isError?: boolean }).isError === true;
    if (turnFailed) {
      emit(EVT.questFail, {
        turnIndex: event.turnIndex,
        messageText: textBlocks,
        inputTokens: inputTokens ?? undefined,
        outputTokens: outputTokens ?? undefined,
      });
    } else {
      emit(EVT.questComplete, {
        turnIndex: event.turnIndex,
        timestamp: new Date().toISOString(),
        messageText: textBlocks,
        inputTokens: inputTokens ?? undefined,
        outputTokens: outputTokens ?? undefined,
        toolResultsCount: (event.toolResults ?? []).length,
      });
    }

    for (const tr of event.toolResults ?? []) {
      const toolName = (tr as { toolName?: string }).toolName;
      const toolCallId = (tr as { toolCallId?: string }).toolCallId ?? "";
      const isError = (tr as { isError?: boolean }).isError === true;
      if (toolName === "gzmo_chaos" || pendingSkillInvokes.has(toolCallId)) {
        const pending = pendingSkillInvokes.get(toolCallId);
        const skill = pending?.skill ?? "unknown";
        const duration_ms = pending ? Date.now() - pending.startedAt : undefined;
        if (isError) {
          emit(EVT.skillError, { skill, toolCallId, turnIndex: event.turnIndex, duration_ms });
        } else {
          emit(EVT.skillComplete, { skill, toolCallId, turnIndex: event.turnIndex, duration_ms });
        }
        pendingSkillInvokes.delete(toolCallId);
      }
    }

    // Check for topic shift mid-session (P1-B)
    checkTopicShift(event, ctx).catch((err) => {
      debug("checkTopicShift failed:", err);
    });
  });

  // --- Startup confirmation ---

  debug(`SynapseNotifier ready → ${cfg.busPath}`);
  if (cfg.enabled) {
    console.info("[synapse-notifier] Synapse bus wired to", cfg.busPath);
  }
}
