export function Waveform({
  levels,
  bars = 10,
  className = "",
}: {
  levels: number[];
  bars?: number;
  className?: string;
}) {
  const values = levels.slice(-bars);
  const padded = [...Array(Math.max(0, bars - values.length)).fill(0), ...values];
  return (
    <span className={`waveform ${className}`} aria-hidden>
      {padded.map((value, index) => (
        <span
          key={index}
          className="waveform-bar"
          style={{ height: `${3 + 11 * Math.max(0, Math.min(1, value))}px` }}
        />
      ))}
    </span>
  );
}
