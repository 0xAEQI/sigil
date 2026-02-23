interface BadgeProps {
  text: string;
  color?: string;
}

export function Badge({ text, color = "#6C3CE1" }: BadgeProps) {
  return (
    <span
      className="inline-block rounded-full px-3 py-1 text-[11px] font-semibold uppercase tracking-[2px] text-white"
      style={{ backgroundColor: color }}
    >
      {text}
    </span>
  );
}
