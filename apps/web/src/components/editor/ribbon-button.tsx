import type { LucideIcon } from "lucide-react";

interface RibbonButtonProps {
  icon: LucideIcon;
  label: string;
  shortcut?: string;
  size?: "large" | "small";
  active?: boolean;
  disabled?: boolean;
  onClick: () => void;
}

export function RibbonButton({
  icon: Icon,
  label,
  shortcut,
  size = "large",
  active = false,
  disabled = false,
  onClick,
}: RibbonButtonProps) {
  const title = shortcut ? `${label} (${shortcut})` : label;

  if (size === "small") {
    return (
      <button
        onClick={onClick}
        disabled={disabled}
        title={title}
        className={`flex items-center gap-1.5 rounded px-2 py-1 text-xs transition-colors ${
          active
            ? "bg-[var(--cad-accent)]/20 text-[var(--cad-accent)]"
            : disabled
              ? "text-[var(--cad-text-muted)] cursor-not-allowed"
              : "text-[var(--cad-text-secondary)] hover:bg-white/10 hover:text-[var(--cad-text-primary)]"
        }`}
      >
        <Icon size={14} />
        <span>{label}</span>
      </button>
    );
  }

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      title={title}
      className={`flex flex-col items-center gap-1 rounded px-3 py-1.5 transition-colors ${
        active
          ? "bg-[var(--cad-accent)]/20 text-[var(--cad-accent)]"
          : disabled
            ? "text-[var(--cad-text-muted)] cursor-not-allowed"
            : "text-[var(--cad-text-secondary)] hover:bg-white/10 hover:text-[var(--cad-text-primary)]"
      }`}
    >
      <Icon size={24} />
      <span className="text-[10px] leading-tight">{label}</span>
    </button>
  );
}
