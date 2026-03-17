import { useEffect, useRef } from "react";
import {
  Pencil, Trash2, ArrowUp, ArrowDown, EyeOff, Eye, Edit3,
} from "lucide-react";

interface FeatureTreeMenuProps {
  x: number;
  y: number;
  featureId: string;
  featureIndex: number;
  isSuppressed: boolean;
  isFirst: boolean;
  isLast: boolean;
  featureType: string;
  onEdit: (index: number) => void;
  onRename: (index: number) => void;
  onMoveUp: (index: number) => void;
  onMoveDown: (index: number) => void;
  onSuppress: (index: number) => void;
  onUnsuppress: (index: number) => void;
  onDelete: (index: number) => void;
  onClose: () => void;
}

interface MenuItem {
  label: string;
  icon: any;
  shortcut?: string;
  disabled?: boolean;
  danger?: boolean;
  onClick: () => void;
}

export function FeatureTreeMenu({
  x,
  y,
  featureIndex,
  isSuppressed,
  isFirst,
  isLast,
  featureType,
  onEdit,
  onRename,
  onMoveUp,
  onMoveDown,
  onSuppress,
  onUnsuppress,
  onDelete,
  onClose,
}: FeatureTreeMenuProps) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        onClose();
      }
    };
    const keyHandler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("mousedown", handler);
    document.addEventListener("keydown", keyHandler);
    return () => {
      document.removeEventListener("mousedown", handler);
      document.removeEventListener("keydown", keyHandler);
    };
  }, [onClose]);

  // Adjust position to stay within viewport
  useEffect(() => {
    if (!ref.current) return;
    const rect = ref.current.getBoundingClientRect();
    if (rect.bottom > window.innerHeight) {
      ref.current.style.top = `${y - rect.height}px`;
    }
    if (rect.right > window.innerWidth) {
      ref.current.style.left = `${x - rect.width}px`;
    }
  }, [x, y]);

  const items: (MenuItem | "separator")[] = [
    {
      label: "Edit Feature",
      icon: Edit3,
      disabled: featureType === "sketch",
      onClick: () => { onEdit(featureIndex); onClose(); },
    },
    {
      label: "Rename",
      icon: Pencil,
      shortcut: "F2",
      onClick: () => { onRename(featureIndex); onClose(); },
    },
    "separator",
    {
      label: "Move Up",
      icon: ArrowUp,
      disabled: isFirst,
      onClick: () => { onMoveUp(featureIndex); onClose(); },
    },
    {
      label: "Move Down",
      icon: ArrowDown,
      disabled: isLast,
      onClick: () => { onMoveDown(featureIndex); onClose(); },
    },
    "separator",
    isSuppressed
      ? {
          label: "Unsuppress",
          icon: Eye,
          onClick: () => { onUnsuppress(featureIndex); onClose(); },
        }
      : {
          label: "Suppress",
          icon: EyeOff,
          onClick: () => { onSuppress(featureIndex); onClose(); },
        },
    "separator",
    {
      label: "Delete",
      icon: Trash2,
      danger: true,
      shortcut: "Del",
      onClick: () => { onDelete(featureIndex); onClose(); },
    },
  ];

  return (
    <div
      ref={ref}
      className="fixed z-[100] min-w-[180px] rounded-md border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] shadow-xl py-1"
      style={{ left: x, top: y }}
    >
      {items.map((item, i) => {
        if (item === "separator") {
          return <div key={i} className="my-1 h-px bg-[var(--cad-border)]" />;
        }
        const Icon = item.icon;
        return (
          <button
            key={item.label}
            onClick={item.onClick}
            disabled={item.disabled}
            className={`flex w-full items-center gap-2 px-3 py-1.5 text-xs transition-colors ${
              item.disabled
                ? "text-[var(--cad-text-muted)] cursor-not-allowed"
                : item.danger
                  ? "text-red-400 hover:bg-red-500/10"
                  : "text-[var(--cad-text-secondary)] hover:bg-white/10 hover:text-[var(--cad-text-primary)]"
            }`}
          >
            <Icon size={14} />
            <span className="flex-1 text-left">{item.label}</span>
            {item.shortcut && (
              <span className="text-[10px] text-[var(--cad-text-muted)]">{item.shortcut}</span>
            )}
          </button>
        );
      })}
    </div>
  );
}
