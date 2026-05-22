import { colorOf } from "../types";

interface Props {
  /** 使用率 (0-100)。 */
  pct: number;
  /** 中央上に出すラベル (例: "5h")。 */
  label: string;
  /** リング下に出す補足 (例: リセット時刻)。 */
  caption: string;
  size?: number;
}

/** SVG の円弧で使用率を表すリングゲージ。使用率に応じて色が変わる。 */
export function RingGauge({ pct, label, caption, size = 96 }: Props) {
  const stroke = 9;
  const radius = (size - stroke) / 2;
  const circumference = 2 * Math.PI * radius;
  const clamped = Math.max(0, Math.min(100, pct));
  const offset = circumference * (1 - clamped / 100);
  const color = colorOf(clamped);

  return (
    <div className="ring">
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke="rgba(255,255,255,0.12)"
          strokeWidth={stroke}
        />
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke={color}
          strokeWidth={stroke}
          strokeLinecap="round"
          strokeDasharray={circumference}
          strokeDashoffset={offset}
          transform={`rotate(-90 ${size / 2} ${size / 2})`}
          style={{ transition: "stroke-dashoffset 0.6s ease, stroke 0.6s ease" }}
        />
        <text
          x="50%"
          y="44%"
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
          fill={color}
        >
          {`${Math.round(clamped)}%`}
        </text>
      </svg>
      <span className="ring-caption">{caption}</span>
    </div>
  );
}
