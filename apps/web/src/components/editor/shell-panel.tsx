import { useEditorStore } from "@/stores/editor-store";
import { usePreferencesStore } from "@/stores/preferences-store";

export function ShellPanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const updateOperationParams = useEditorStore((s) => s.updateOperationParams);
  const selectedFaceIndex = useEditorStore((s) => s.selectedFaceIndex);
  const unitSystem = usePreferencesStore((s) => s.unitSystem);

  if (!activeOperation || activeOperation.type !== "shell") return null;

  const {
    thickness = 1,
    faces_to_remove = [],
  } = activeOperation.params;

  const inputClass = "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass = "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  const toggleFace = (index: number) => {
    const current: number[] = faces_to_remove;
    const next = current.includes(index)
      ? current.filter((i: number) => i !== index)
      : [...current, index];
    updateOperationParams({ faces_to_remove: next });
  };

  return (
    <div className="space-y-3" data-testid="shell-panel">
      {/* Face Selection */}
      <div>
        <h4 className={sectionHeaderClass}>Faces to Remove</h4>
        <button
          onClick={() => {
            const store = useEditorStore.getState();
            if (store.mode === "select-face") {
              store.setMode("view");
            } else {
              store.setMode("select-face");
            }
          }}
          data-testid="shell-select-faces"
          className="w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1.5 text-xs text-[var(--cad-text-secondary)] hover:bg-[var(--cad-bg-hover)] transition-colors"
        >
          {faces_to_remove.length > 0
            ? `${faces_to_remove.length} face(s) selected`
            : "Click faces to select..."}
        </button>
        {selectedFaceIndex != null && (
          <button
            onClick={() => toggleFace(selectedFaceIndex)}
            data-testid="shell-toggle-face"
            className="mt-1 w-full rounded border border-[var(--cad-accent)]/30 bg-[var(--cad-accent)]/10 px-2 py-1 text-xs text-[var(--cad-accent)] hover:bg-[var(--cad-accent)]/20 transition-colors"
          >
            {faces_to_remove.includes(selectedFaceIndex)
              ? `Remove face ${selectedFaceIndex} from selection`
              : `Add face ${selectedFaceIndex} to selection`}
          </button>
        )}
        <p className="mt-0.5 text-[10px] text-[var(--cad-text-muted)]">
          Select faces to open (e.g., top face of a box)
        </p>
      </div>

      {/* Thickness */}
      <div>
        <label className={sectionHeaderClass}>Thickness</label>
        <div className="flex items-center gap-1">
          <input
            type="number"
            value={thickness}
            onChange={(e) => updateOperationParams({ thickness: Math.max(0.01, Number(e.target.value)) })}
            data-testid="shell-thickness"
            className={inputClass}
            min={0.01}
            step={0.5}
          />
          <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
        </div>
      </div>
    </div>
  );
}
