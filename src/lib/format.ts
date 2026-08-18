// Presentation formatting for the landing rows. The en-US forms match
// the pinned prototype (`prototype/map-1-8`) and stay deterministic
// across machines.

/** `2026-08-16T22:31:05Z` -> `Aug 16, 10:31 PM`. */
export function formatDate(createdAt: string): string {
  const date = new Date(createdAt);
  if (Number.isNaN(date.getTime())) {
    return "—";
  }
  return date.toLocaleString("en-US", {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
}

/** Milliseconds -> `18s` / `3m 7s`; null (unreadable log) -> `—`. */
export function formatDuration(durationMs: number | null): string {
  if (durationMs === null) {
    return "—";
  }
  const seconds = Math.round(durationMs / 1000);
  if (seconds < 60) {
    return `${seconds}s`;
  }
  return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
}

/** The `date · step count · duration` row meta line. */
export function formatSummaryMeta(summary: {
  created_at: string;
  step_count: number;
  duration_ms: number | null;
}): string {
  const steps = summary.step_count === 1 ? "1 step" : `${summary.step_count} steps`;
  return [
    formatDate(summary.created_at),
    steps,
    formatDuration(summary.duration_ms),
  ].join(" · ");
}

/** Event timestamp -> local `HH:MM:SS` for the step list and metadata. */
export function formatEventTime(ts: string): string {
  const date = new Date(ts);
  if (Number.isNaN(date.getTime())) {
    return "—";
  }
  return date.toLocaleTimeString("en-US", { hour12: false });
}

const MODIFIER_LABELS: Record<string, string> = {
  fn: "Fn",
  control: "Ctrl",
  option: "Option",
  shift: "Shift",
  command: "Cmd",
  caps_lock: "CapsLock",
};

/** Key metadata -> `Cmd+s` / `h` / `key 36` for the metadata grid. */
export function formatKey(key: {
  key_code: number;
  chars: string;
  modifiers: string[];
}): string {
  const parts = key.modifiers.map(
    (modifier) => MODIFIER_LABELS[modifier] ?? modifier,
  );
  parts.push(key.chars !== "" ? key.chars : `key ${key.key_code}`);
  return parts.join("+");
}

/** Element metadata -> `AXButton "OK" · ax`; fallback -> `— · fallback`. */
export function formatElement(element: {
  role: string | null;
  title: string | null;
  source: string;
}): string {
  const named = [
    element.role ?? "",
    element.title !== null && element.title !== "" ? `"${element.title}"` : "",
  ]
    .filter((part) => part !== "")
    .join(" ");
  return `${named !== "" ? named : "—"} · ${element.source}`;
}
