import { colorOf, levelOf } from "../types";

interface Props {
  /** 使用率 (0-100)。 */
  pct: number;
  /** 中央上に出すラベル (例: "5h")。 */
  label: string;
  /** リング下に出す補足 (例: リセット時刻)。 */
  caption: string;
  size?: number;
  /** SVG の <defs id> を一意にするキー。 */
  idKey: string;
}

/** SVG の円弧で使用率を表すリングゲージ。単色 + パルス。 */
export function RingGauge({ pct, label, caption, size = 82, idKey: _idKey }: Props) {
  const stroke = 8;
  const radius = (size - stroke) / 2;
  const circumference = 2 * Math.PI * radius;
  const clamped = Math.max(0, Math.min(100, pct));
  const offset = circumference * (1 - clamped / 100);
  const accent = colorOf(clamped);
  const level = levelOf(clamped);

  return (
    <div className={`ring ring-${level}`}>
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
        {/* 背景トラック */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke="rgba(255,255,255,0.08)"
          strokeWidth={stroke}
        />

        {/* 使用率アーク (単色) */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke={accent}
          strokeWidth={stroke}
          strokeLinecap="round"
          strokeDasharray={circumference}
          strokeDashoffset={offset}
          transform={`rotate(-90 ${size / 2} ${size / 2})`}
          style={{
            transition: "stroke-dashoffset 0.8s ease, stroke 0.6s ease",
          }}
        />

        {/* 中央: ラベル + 使用率 */}
        <text
          x="50%"
          y="42%"
          textAnchor="middle"
          dominantBaseline="middle"
          className="ring-label"
        >
          {label}
        </text>
        <text
          x="50%"
          y="64%"
          textAnchor="middle"
          dominantBaseline="middle"
          className="ring-pct"
          fill={accent}
        >
          {`${Math.round(clamped)}`}
          <tspan className="ring-pct-sign" dx="1" dy="0">
            %
          </tspan>
        </text>
      </svg>
      <span className="ring-caption">{caption}</span>
    </div>
  );
}
