import { useState, useRef } from "react";
import type { LucideIcon } from "lucide-react";

interface RibbonButtonProps {
  icon: LucideIcon;
  label: string;
  shortcut?: string;
  description?: string;
  size?: "large" | "small";
  active?: boolean;
  disabled?: boolean;
  testId?: string;
  onClick: () => void;
}

function Tooltip({
  label,
  shortcut,
  description,
  anchorRef,
}: {
  label: string;
  shortcut?: string;
  description?: string;
  anchorRef: React.RefObject<HTMLButtonElement | null>;
}) {
  if (!anchorRef.current) return null;

  return (
    <div
      className="fixed z-[150] pointer-events-none"
      style={{
        left: anchorRef.current.getBoundingClientRect().left,
        top: anchorRef.current.getBoundingClientRect().bottom + 4,
      }}
    >
      <div className="rounded border border-[var(--cad-border)] bg-[#1e1e22] shadow-lg px-2.5 py-1.5 max-w-[220px]">
        <div className="flex items-center gap-2">
          <span className="text-xs font-medium text-[var(--cad-text-primary)]">{label}</span>
          {shortcut && (
            <kbd className="rounded bg-white/10 px-1 py-0.5 text-[9px] text-[var(--cad-text-muted)]">
              {shortcut}
            </kbd>
          )}
        </div>
        {description && (
          <p className="mt-0.5 text-[10px] text-[var(--cad-text-muted)] leading-tight">{description}</p>
        )}
      </div>
    </div>
  );
}

export function RibbonButton({
  icon: Icon,
  label,
  shortcut,
  description,
  size = "large",
  active = false,
  disabled = false,
  testId,
  onClick,
}: RibbonButtonProps) {
  const [showTooltip, setShowTooltip] = useState(false);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout>>(null);

  const handleMouseEnter = () => {
    timerRef.current = setTimeout(() => setShowTooltip(true), 500);
  };
  const handleMouseLeave = () => {
    if (timerRef.current) clearTimeout(timerRef.current);
    setShowTooltip(false);
  };

  const title = shortcut ? `${label} (${shortcut})` : label;

  if (size === "small") {
    return (
      <>
        <button
          ref={buttonRef}
          onClick={onClick}
          disabled={disabled}
          title={!showTooltip ? title : undefined}
          data-testid={testId}
          onMouseEnter={handleMouseEnter}
          onMouseLeave={handleMouseLeave}
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
        {showTooltip && (
          <Tooltip label={label} shortcut={shortcut} description={description} anchorRef={buttonRef} />
        )}
      </>
    );
  }

  return (
    <>
      <button
        ref={buttonRef}
        onClick={onClick}
        disabled={disabled}
        title={!showTooltip ? title : undefined}
        data-testid={testId}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
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
      {showTooltip && (
        <Tooltip label={label} shortcut={shortcut} description={description} anchorRef={buttonRef} />
      )}
    </>
  );
}
