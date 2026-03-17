import { useEditorStore } from "@/stores/editor-store";
import { usePreferencesStore } from "@/stores/preferences-store";

export function FaceFilletPanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const updateOperationParams = useEditorStore((s) => s.updateOperationParams);
  const selectedFaceIndex = useEditorStore((s) => s.selectedFaceIndex);
  const unitSystem = usePreferencesStore((s) => s.unitSystem);

  if (!activeOperation || activeOperation.type !== "face_fillet") return null;

  const {
    radius = 1,
    face_indices = [],
  } = activeOperation.params;

  const inputClass = "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass = "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  return (
    <div className="space-y-3" data-testid="face-fillet-panel">
      {/* Face Selection */}
      <div>
        <h4 className={sectionHeaderClass}>Faces</h4>
        <button
          onClick={() => {
            const store = useEditorStore.getState();
            if (store.mode === "select-face") {
              store.setMode("view");
            } else {
              store.setMode("select-face");
            }
          }}
          data-testid="face-fillet-select-faces"
          className="w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1.5 text-xs text-[var(--cad-text-secondary)] hover:bg-[var(--cad-bg-hover)] transition-colors"
        >
          {face_indices.length > 0
            ? `${face_indices.length} face(s) selected — [${face_indices.join(", ")}]`
            : "Click faces to select..."}
        </button>
        {selectedFaceIndex != null && (
          <button
            onClick={() => {
              const current: number[] = face_indices;
              const next = current.includes(selectedFaceIndex)
                ? current.filter((i: number) => i !== selectedFaceIndex)
                : [...current, selectedFaceIndex];
              updateOperationParams({ face_indices: next });
            }}
            data-testid="face-fillet-toggle-face"
            className="mt-1 w-full rounded border border-[var(--cad-accent)]/30 bg-[var(--cad-accent)]/10 px-2 py-1 text-xs text-[var(--cad-accent)] hover:bg-[var(--cad-accent)]/20 transition-colors"
          >
            {face_indices.includes(selectedFaceIndex)
              ? `Remove face ${selectedFaceIndex} from selection`
              : `Add face ${selectedFaceIndex} to selection`}
          </button>
        )}
        <p className="mt-0.5 text-[10px] text-[var(--cad-text-muted)]">
          Select faces to apply fillet to their edges.
        </p>
      </div>

      {/* Radius */}
      <div>
        <label className={sectionHeaderClass}>Radius</label>
        <div className="flex items-center gap-1">
          <input
            type="number"
            value={radius}
            onChange={(e) => updateOperationParams({ radius: Math.max(0.1, Number(e.target.value)) })}
            data-testid="face-fillet-radius"
            className={inputClass}
            min={0.1}
            step={0.5}
          />
          <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
        </div>
      </div>
    </div>
  );
}
