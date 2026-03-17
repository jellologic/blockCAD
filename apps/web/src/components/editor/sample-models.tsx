import { useState, useRef, useEffect } from "react";
import { ChevronDown } from "lucide-react";
import { SAMPLE_MODELS } from "@/lib/samples";
import { useEditorStore } from "@/stores/editor-store";

export function SampleModelsDropdown() {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const loadSample = useEditorStore((s) => s.loadSample);

  // Close on outside click
  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  return (
    <div ref={ref} className="relative" data-testid="sample-models-dropdown">
      <button
        onClick={() => setOpen((v) => !v)}
        className="flex items-center gap-1 rounded px-3 py-1.5 text-xs font-medium text-[var(--cad-text-secondary)] hover:bg-white/10 hover:text-[var(--cad-text-primary)] transition-colors"
      >
        Samples
        <ChevronDown size={14} />
      </button>

      {open && (
        <div className="absolute left-0 top-full z-50 mt-1 w-64 rounded-md border border-[var(--cad-border)] bg-[var(--cad-bg-ribbon)] shadow-lg">
          {SAMPLE_MODELS.map((sample) => (
            <button
              key={sample.id}
              data-testid={`sample-${sample.id}`}
              onClick={() => {
                loadSample(sample.id);
                setOpen(false);
              }}
              className="flex w-full items-start gap-2 px-3 py-2 text-left text-xs transition-colors hover:bg-white/10 first:rounded-t-md last:rounded-b-md"
            >
              <span className="mt-0.5 text-sm leading-none">{sample.icon}</span>
              <div className="min-w-0">
                <div className="font-medium text-[var(--cad-text-primary)]">{sample.name}</div>
                <div className="text-[var(--cad-text-muted)] truncate">{sample.description}</div>
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
