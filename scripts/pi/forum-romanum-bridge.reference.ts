/**
 * Forum Romanum bridge — maps pi-crew lifecycle hooks to Synapse bus events.
 *
 * Copy to ~/.pi/agent/extensions/forum-romanum-bridge.ts (after synapse-notifier).
 * Requires pi-crew package installed.
 */
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import * as fs from "node:fs";
import * as path from "node:path";
import { randomUUID } from "node:crypto";

const GZMO_ROOT =
  process.env.GZMO_ROOT ??
  path.join(process.env.HOME || "", "Projects", "_foundation-audit", "survey_GZMO");

function busPath(): string {
  const fromEnv = process.env.GZMO_SYNAPSE_BUS;
  if (fromEnv) return fromEnv.replace(/^~/, process.env.HOME || "");
  return path.join(GZMO_ROOT, "data", "Synapse", "events.jsonl");
}

function appendLocked(line: string): void {
  const bus = busPath();
  const dir = path.dirname(bus);
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
  const lock = path.join(dir, `${path.basename(bus)}.lock`);
  if (!fs.existsSync(lock)) fs.writeFileSync(lock, "\n");
  const tmp = path.join(dir, `.forum-${process.pid}.line`);
  fs.writeFileSync(tmp, line.endsWith("\n") ? line : `${line}\n`);
  const { execFileSync } = require("node:child_process") as typeof import("node:child_process");
  const q = (s: string) => `'${s.replace(/'/g, `'\\''`)}'`;
  try {
    execFileSync("flock", ["-x", lock, "bash", "-lc", `cat ${q(tmp)} >> ${q(bus)} && rm -f ${q(tmp)}`], {
      stdio: "pipe",
      timeout: 15_000,
    });
  } catch {
    fs.appendFileSync(bus, line.endsWith("\n") ? line : `${line}\n`);
    try {
      fs.unlinkSync(tmp);
    } catch {
      /* ignore */
    }
  }
}

function emit(
  event_type: string,
  data: Record<string, unknown>,
  opts?: { correlation_id?: string; reply_to?: string },
): string {
  const id = randomUUID();
  const event = {
    id,
    event_type,
    source: "pi_agent",
    timestamp: new Date().toISOString(),
    correlation_id: opts?.correlation_id,
    reply_to: opts?.reply_to,
    data,
  };
  appendLocked(JSON.stringify(event));
  return id;
}

type CrewHookEvent = {
  type: string;
  timestamp: string;
  runId: string;
  taskId?: string;
  data?: Record<string, unknown>;
};

function agentIdFromRole(role: string): string {
  const r = role.toLowerCase();
  if (r.includes("prometheus") || r.includes("proposer")) return "prometheus";
  if (r.includes("epimetheus") || r.includes("critic") || r.includes("review")) return "epimetheus";
  return role || "crew-agent";
}

export default function forumRomanumBridge(_pi: ExtensionAPI): void {
  let crewHooks: {
    register: (type: string, fn: (e: CrewHookEvent) => void) => void;
  } | null = null;

  try {
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const mod = require("pi-crew/src/runtime/crew-hooks.ts");
    crewHooks = mod.crewHooks ?? mod.default?.crewHooks ?? null;
  } catch {
    try {
      const mod = require(path.join(process.env.HOME || "", "node_modules/pi-crew/src/runtime/crew-hooks.ts"));
      crewHooks = mod.crewHooks;
    } catch {
      console.error("[forum-romanum] pi-crew not available — bridge inactive");
      return;
    }
  }

  if (!crewHooks) {
    console.error("[forum-romanum] crewHooks missing — bridge inactive");
    return;
  }

  const lastMessageByTask = new Map<string, string>();
  const proposalByTask = new Map<string, string>();

  crewHooks.register("task_started", (event) => {
    const role = String(event.data?.role ?? "agent");
    const agent_id = agentIdFromRole(role);
    const mode = agent_id === "epimetheus" ? "debate" : "explore";
    const msgId = emit(
      "agent.message",
      {
        agent_id,
        role: agent_id === "prometheus" ? "proposer" : "critic",
        mode,
        payload: {
          text: `task ${event.taskId ?? "?"} started (${role})`,
          runId: event.runId,
          taskId: event.taskId,
        },
      },
      { correlation_id: event.runId },
    );
    if (event.taskId) lastMessageByTask.set(event.taskId, msgId);

    if (agent_id === "prometheus" && event.taskId) {
      const proposal_id = randomUUID();
      proposalByTask.set(event.taskId, proposal_id);
      emit(
        "proposal.created",
        {
          agent_id,
          proposal_id,
          title: `crew task ${event.taskId}`,
          body: String(event.data?.brief ?? event.data?.role ?? "pi-crew task"),
          status: "draft",
        },
        { correlation_id: event.runId, reply_to: msgId },
      );
    }
  });

  crewHooks.register("task_completed", (event) => {
    const role = String(event.data?.role ?? "agent");
    const agent_id = agentIdFromRole(role);
    const reply = event.taskId ? lastMessageByTask.get(event.taskId) : undefined;
    emit(
      "agent.result",
      {
        agent_id,
        taskId: event.taskId,
        runId: event.runId,
        status: "completed",
        payload: event.data ?? {},
      },
      { correlation_id: event.runId, reply_to: reply },
    );

    if (agent_id === "epimetheus" && event.taskId) {
      const proposal_id = proposalByTask.get(event.taskId) ?? randomUUID();
      emit(
        "proposal.reviewed",
        {
          agent_id,
          proposal_id,
          verdict: "accept",
          comments: String(event.data?.summary ?? "crew task completed"),
        },
        { correlation_id: event.runId, reply_to: reply },
      );
    }
  });

  crewHooks.register("task_failed", (event) => {
    emit(
      "agent.error",
      {
        agent_id: agentIdFromRole(String(event.data?.role ?? "agent")),
        taskId: event.taskId,
        runId: event.runId,
        error: String(event.data?.error ?? "task_failed"),
      },
      { correlation_id: event.runId },
    );
  });

  console.info("[forum-romanum] pi-crew hooks wired to", busPath());
}
