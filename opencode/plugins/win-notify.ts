import { exec } from "node:child_process";
import { promisify } from "node:util";

import type { Plugin } from "@opencode-ai/plugin";
import type { Permission } from "@opencode-ai/sdk";

const execAsync = promisify(exec);

const AUMID = "OpenCode";
const TITLE_LIMIT = 40;
const MESSAGE_LIMIT = 200;

// --- PowerShell helpers ---

/**
 * Runs a PowerShell script safely by encoding it as Base64 (UTF-16LE).
 * Eliminates command-line quoting, newline, and variable expansion issues.
 */
async function runPs(script: string): Promise<boolean> {
  try {
    const encoded = Buffer.from(script, "utf16le").toString("base64");
    await execAsync(
      `powershell -NoProfile -ExecutionPolicy Bypass -EncodedCommand ${encoded}`,
      { windowsHide: true }
    );
    return true;
  } catch (error) {
    console.error("[win-notify] PowerShell failed:", error);
    return false;
  }
}

/** Escapes single quotes for PowerShell literal strings (' -> ''). */
function escapePsSingleQuotes(text: string): string {
  return text.replace(/'/g, "''");
}

/** Normalizes and truncates text so it fits comfortably in a toast. */
function clip(text: string | undefined | null, max: number): string {
  const value = (text ?? "").trim();
  return value.length > max ? `${value.slice(0, max - 1)}\u2026` : value;
}

/**
 * Native Windows toast, addressed to the given AUMID.
 */
async function sendNativeToast(title: string, message: string, appId: string): Promise<boolean> {
  const t = escapePsSingleQuotes(title);
  const m = escapePsSingleQuotes(message);
  const a = escapePsSingleQuotes(appId);

  const script = `
$null = [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime]
$Template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02)
$TextNodes = $Template.GetElementsByTagName('text')
$TextNodes.Item(0).AppendChild($Template.CreateTextNode('${t}')) | Out-Null
$TextNodes.Item(1).AppendChild($Template.CreateTextNode('${m}')) | Out-Null
[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('${a}').Show([Windows.UI.Notifications.ToastNotification]::new($Template))
  `;

  return runPs(script);
}

/**
 * Fallback popup notification using msg.exe (used when the toast AUMID isn't
 * registered, e.g. for a freshly-installed app).
 */
async function sendMsgPopup(title: string, message: string): Promise<boolean> {
  const body = escapePsSingleQuotes(`${title}: ${message}`);
  const script = `msg.exe "$env:USERNAME" /TIME:30 '${body}'`;
  return runPs(script);
}

/**
 * Sends a Windows notification, falling back from native toast → msg popup.
 */
async function sendToast(title: string, message: string): Promise<void> {
  const t = clip(title, TITLE_LIMIT);
  const m = clip(message, MESSAGE_LIMIT);
  if (m === "" && t === "") return;
  if (await sendNativeToast(t || "OpenCode", m, AUMID)) return;
  await sendMsgPopup(t || "OpenCode", m);
}

// --- Notification content helpers ---

/** Extracts a human-readable message from a session error payload. */
function errorMessage(error: unknown): string {
  if (error && typeof error === "object" && "data" in error) {
    const data = (error as { data?: unknown }).data;
    if (data && typeof data === "object" && "message" in data) {
      return String((data as { message: unknown }).message);
    }
  }
  return "An error occurred during execution.";
}

/** Renders a Permission as a human-readable label for the notification body. */
function permissionLabel(p: Permission): string {
  const title = p.title?.trim();
  const type = p.type?.trim();
  if (title && type && title !== type) return `${title} (${type})`;
  return title || type || "An action";
}

/**
 * Runtime-safe handling of `question.asked`.
 *
 * The plugin's `event` hook is typed against an older `Event` union that omits
 * this event (it lives in the v2 SDK), so we inspect the payload structurally
 * rather than at type level.
 */
async function handleQuestionEvent(event: unknown): Promise<void> {
  if (!event || typeof event !== "object") return;
  const e = event as { type?: unknown; properties?: unknown };
  if (e.type !== "question.asked") return;

  const props = e.properties as {
    questions?: Array<{ question?: string }>;
  } | undefined;
  const question = props?.questions?.[0]?.question;
  await sendToast(
    "OpenCode Question",
    clip(question, MESSAGE_LIMIT) || "Agent is waiting for your input."
  );
}

// --- Plugin Entrypoint ---

const WindowsNotifier: Plugin = async () => {
  return {
    event: async ({ event }) => {
      switch (event.type) {
        case "session.idle":
          await sendToast("OpenCode Done", "Task completed successfully.");
          break;

        case "session.error":
          await sendToast("OpenCode Error", errorMessage(event.properties.error));
          break;

        // Surface the TUI's own status toasts as native popups, but only for
        // the states that usually warrant attention.
        case "tui.toast.show":
          if (event.properties.variant === "error" || event.properties.variant === "warning") {
            await sendToast("OpenCode", event.properties.message);
          }
          break;

        default:
          await handleQuestionEvent(event);
      }
    },

    // Precise signal that the agent is blocked waiting on user approval.
    "permission.ask": async (permission) => {
      await sendToast("OpenCode Approval", `${permissionLabel(permission)} requires your input.`);
    },
  };
};

export default WindowsNotifier;