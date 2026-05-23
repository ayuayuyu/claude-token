import type { Usage } from "../types";
import { formatReset } from "../types";
import { RingGauge } from "./RingGauge";

interface Props {
  usage: Usage;
}

/** 5時間枠 / 7日枠の2リングを並べたカード。 */
export function UsageCard({ usage }: Props) {
  const fivePct = usage.five_hour?.utilization ?? 0;
  const sevenPct = usage.seven_day?.utilization ?? 0;

  return (
    <div className="card">
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
