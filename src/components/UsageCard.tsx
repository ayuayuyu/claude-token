import type { Usage } from "../types";
import { faceOf, formatReset } from "../types";
import { RingGauge } from "./RingGauge";

interface Props {
  usage: Usage;
}

/** 5時間枠 / 7日枠の2リングを並べたグラスモーフィズムのカード。 */
export function UsageCard({ usage }: Props) {
  const fivePct = usage.five_hour?.utilization ?? 0;
  const sevenPct = usage.seven_day?.utilization ?? 0;
  // 表情は逼迫している方の枠に合わせる。
  const face = faceOf(Math.max(fivePct, sevenPct));

  return (
    <div className="card">
      <div className="face">{face}</div>
      <div className="rings">
        <RingGauge
          pct={fivePct}
          label="5h"
          caption={formatReset(usage.five_hour?.resets_at ?? null)}
        />
        <RingGauge
          pct={sevenPct}
          label="7d"
          caption={formatReset(usage.seven_day?.resets_at ?? null)}
        />
      </div>
    </div>
  );
}
