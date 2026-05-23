import type { Usage } from "../types";
import { formatReset, levelOf } from "../types";
import { RingGauge } from "./RingGauge";

interface Props {
  usage: Usage;
}

/** 5時間枠 / 7日枠の2リングを並べたグラスモーフィズムのカード。 */
export function UsageCard({ usage }: Props) {
  const fivePct = usage.five_hour?.utilization ?? 0;
  const sevenPct = usage.seven_day?.utilization ?? 0;
  // ムードは逼迫している方の枠に合わせる。
  const worst = Math.max(fivePct, sevenPct);
  const mood = levelOf(worst);

  return (
    <div className={`card mood-${mood}`}>
      <div className={`mood-avatar mood-avatar-${mood}`}>
        <img src="/ayuayuyu.png" alt="" draggable={false} />
      </div>
      <div className="rings">
        <RingGauge
          pct={fivePct}
          label="5h"
          caption={`reset ${formatReset(usage.five_hour?.resets_at ?? null)}`}
          idKey="five"
          size={82}
        />
        <RingGauge
          pct={sevenPct}
          label="7d"
          caption={`reset ${formatReset(usage.seven_day?.resets_at ?? null)}`}
          idKey="seven"
          size={82}
        />
      </div>
    </div>
  );
}
