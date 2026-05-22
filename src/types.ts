/** Rust 側 `Usage` と対応する型 (serde のフィールド名に一致)。 */
export interface UsageWindow {
  utilization: number;
  resets_at: string | null;
}

export interface Usage {
  five_hour: UsageWindow | null;
  seven_day: UsageWindow | null;
}

/** 使用率に応じた状態レベル。色と表情の切り替えに使う。 */
export type Level = "calm" | "normal" | "warn" | "critical";

export function levelOf(pct: number): Level {
  if (pct >= 90) return "critical";
  if (pct >= 75) return "warn";
  if (pct >= 50) return "normal";
  return "calm";
}

const LEVEL_COLOR: Record<Level, string> = {
  calm: "#4ade80", // green
  normal: "#38bdf8", // blue
  warn: "#fbbf24", // amber
  critical: "#f87171", // red
};

const LEVEL_FACE: Record<Level, string> = {
  calm: "(˶ᵔ ᵕ ᵔ˶)",
  normal: "(๑•‿•๑)",
  warn: "(˶°ㅁ°)",
  critical: "(╯°□°)╯",
};

export function colorOf(pct: number): string {
  return LEVEL_COLOR[levelOf(pct)];
}

export function faceOf(pct: number): string {
  return LEVEL_FACE[levelOf(pct)];
}

/** ISO8601 のリセット時刻を "15:30" / "5/26 15:00" 形式に整形する。 */
export function formatReset(iso: string | null): string {
  if (!iso) return "—";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "—";
  const now = new Date();
  const hm = `${d.getHours()}:${String(d.getMinutes()).padStart(2, "0")}`;
  // 同じ日ならば時刻のみ、別日なら日付も付ける。
  const sameDay =
    d.getFullYear() === now.getFullYear() &&
    d.getMonth() === now.getMonth() &&
    d.getDate() === now.getDate();
  return sameDay ? hm : `${d.getMonth() + 1}/${d.getDate()} ${hm}`;
}
