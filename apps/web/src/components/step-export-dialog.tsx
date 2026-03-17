import { useState } from "react";
import { X, Download } from "lucide-react";
import { useEditorStore } from "@/stores/editor-store";

interface StepExportDialogProps {
  onClose: () => void;
}

export function StepExportDialog({ onClose }: StepExportDialogProps) {
  const exportSTEP = useEditorStore((s) => s.exportSTEP);
  const hasMesh = useEditorStore((s) => s.meshData !== null);

  const [schema, setSchema] = useState<"AP203" | "AP214">("AP214");
  const [author, setAuthor] = useState("");
  const [organization, setOrganization] = useState("");

  const handleExport = () => {
    exportSTEP({ schema, author, organization });
    onClose();
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      data-testid="step-export-dialog"
    >
      <div className="w-[360px] rounded-lg border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-[var(--cad-border)] px-4 py-3">
          <span className="text-sm font-medium text-[var(--cad-text-primary)]">
            Export STEP
          </span>
          <button
            onClick={onClose}
            data-testid="step-export-close"
            className="rounded p-1 hover:bg-white/10"
          >
            <X size={16} className="text-[var(--cad-text-muted)]" />
          </button>
        </div>

        {/* Form */}
        <div className="flex flex-col gap-3 p-4">
          {/* Schema selector */}
          <div className="flex flex-col gap-1">
            <label className="text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]">
              Schema
            </label>
            <div className="flex gap-2">
              <button
                onClick={() => setSchema("AP203")}
                data-testid="step-schema-ap203"
                className={`flex-1 rounded px-3 py-1.5 text-xs transition-colors ${
                  schema === "AP203"
                    ? "bg-[var(--cad-accent)]/20 text-[var(--cad-accent)] font-medium border border-[var(--cad-accent)]/40"
                    : "text-[var(--cad-text-secondary)] border border-[var(--cad-border)] hover:bg-white/5"
                }`}
              >
                AP203
              </button>
              <button
                onClick={() => setSchema("AP214")}
                data-testid="step-schema-ap214"
                className={`flex-1 rounded px-3 py-1.5 text-xs transition-colors ${
                  schema === "AP214"
                    ? "bg-[var(--cad-accent)]/20 text-[var(--cad-accent)] font-medium border border-[var(--cad-accent)]/40"
                    : "text-[var(--cad-text-secondary)] border border-[var(--cad-border)] hover:bg-white/5"
                }`}
              >
                AP214
              </button>
            </div>
            <span className="text-[9px] text-[var(--cad-text-muted)]">
              {schema === "AP203"
                ? "Configuration Controlled Design -- widely compatible"
                : "Automotive Design -- richer color/layer support"}
            </span>
          </div>

          {/* Author */}
          <div className="flex flex-col gap-1">
            <label className="text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]">
              Author (optional)
            </label>
            <input
              type="text"
              value={author}
              onChange={(e) => setAuthor(e.target.value)}
              placeholder="Your name"
              data-testid="step-author"
              className="rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel-alt)] px-2 py-1.5 text-xs text-[var(--cad-text-primary)] placeholder:text-[var(--cad-text-muted)] focus:border-[var(--cad-accent)] focus:outline-none"
            />
          </div>

          {/* Organization */}
          <div className="flex flex-col gap-1">
            <label className="text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]">
              Organization (optional)
            </label>
            <input
              type="text"
              value={organization}
              onChange={(e) => setOrganization(e.target.value)}
              placeholder="Company or team"
              data-testid="step-organization"
              className="rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel-alt)] px-2 py-1.5 text-xs text-[var(--cad-text-primary)] placeholder:text-[var(--cad-text-muted)] focus:border-[var(--cad-accent)] focus:outline-none"
            />
          </div>

          {/* Export button */}
          <button
            onClick={handleExport}
            disabled={!hasMesh}
            data-testid="step-export-confirm"
            className={`mt-1 flex items-center justify-center gap-2 rounded px-4 py-2 text-xs font-medium transition-colors ${
              hasMesh
                ? "bg-[var(--cad-accent)] text-white hover:bg-[var(--cad-accent)]/80"
                : "bg-[var(--cad-border)] text-[var(--cad-text-muted)] cursor-not-allowed"
            }`}
          >
            <Download size={14} />
            Export STEP
          </button>
        </div>
      </div>
    </div>
  );
}
