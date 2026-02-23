interface TagProps {
  text: string;
}

export function Tag({ text }: TagProps) {
  return (
    <span className="inline-block rounded-full border border-border px-3 py-1 text-[12px] text-text-secondary">
      {text}
    </span>
  );
}
