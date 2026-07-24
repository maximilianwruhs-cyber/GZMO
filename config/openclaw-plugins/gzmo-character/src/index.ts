import { execFile } from "node:child_process";
import { homedir } from "node:os";
import { join } from "node:path";
import { promisify } from "node:util";
import { Type } from "typebox";
import { defineToolPlugin } from "openclaw/plugin-sdk/tool-plugin";

const execFileAsync = promisify(execFile);

function resolveChooser(): string {
  return (
    process.env.OPENCLAW_CHARACTER_CHOOSER?.trim() ||
    join(homedir(), "github-clone", "GZMO", "scripts", "openclaw-choose-character.sh")
  );
}

function resolveArgs(params: {
  command?: string;
  action?: string;
  args?: string | Record<string, unknown>;
}): string {
  const direct = params.command?.trim();
  if (direct) return direct;

  if (typeof params.args === "string" && params.args.trim()) {
    return params.args.trim();
  }

  if (params.args && typeof params.args === "object") {
    const action = String(params.args.action ?? params.args.query ?? "").trim();
    const q = String(params.args.q ?? params.args.query ?? "").trim();
    const slug = String(params.args.slug ?? params.args.name ?? "").trim();
    if (action === "search" && q) return `search ${q}`;
    if (action === "list" || action === "who" || action === "status") {
      return action === "status" ? "who" : action;
    }
    if (action && action !== "install") return action;
    if (slug) return slug;
    if (q) return `search ${q}`;
  }

  const action = params.action?.trim();
  if (action) {
    if (action === "status") return "who";
    return action;
  }

  return "who";
}

export default defineToolPlugin({
  id: "gzmo-character",
  name: "GZMO Character",
  description: "GZMO-safe OpenClaw persona chooser (Telegram /character).",
  tools: (tool) => [
    tool({
      name: "character",
      label: "Character",
      description:
        "List/search/install OpenClaw personas from openclaw-agents without wiping GZMO AGENTS.md. Prefer this over exec/read of the character skill.",
      parameters: Type.Object({
        command: Type.Optional(
          Type.String({
            description: "Raw args after /character: who | list | search <q> | <slug>",
          }),
        ),
        commandName: Type.Optional(Type.String()),
        skillName: Type.Optional(Type.String()),
        action: Type.Optional(
          Type.String({ description: "Alias for command when models pass action=list" }),
        ),
        args: Type.Optional(
          Type.Union([
            Type.String(),
            Type.Object(
              {
                action: Type.Optional(Type.String()),
                query: Type.Optional(Type.String()),
                q: Type.Optional(Type.String()),
                slug: Type.Optional(Type.String()),
                name: Type.Optional(Type.String()),
              },
              { additionalProperties: true },
            ),
          ]),
        ),
      }),
      async execute(params, _config, context) {
        context.signal?.throwIfAborted();
        const raw = resolveArgs(params);
        const argv = raw.split(/\s+/).filter(Boolean);
        const chooser = resolveChooser();
        const env = {
          ...process.env,
          OPENCLAW_CHARACTER_FORCE: "1",
        };
        try {
          const { stdout, stderr } = await execFileAsync(
            "bash",
            [chooser, ...(argv.length ? argv : ["who"])],
            {
              env,
              timeout: 120_000,
              maxBuffer: 2 * 1024 * 1024,
            },
          );
          const text = [stdout, stderr].filter(Boolean).join("\n").trim();
          return text || "(no output)";
        } catch (err) {
          const e = err as {
            stdout?: string;
            stderr?: string;
            message?: string;
            code?: number | string;
          };
          const text = [e.stdout, e.stderr, e.message].filter(Boolean).join("\n").trim();
          return `character failed (code=${e.code ?? "?"}):\n${text}`;
        }
      },
    }),
  ],
});
