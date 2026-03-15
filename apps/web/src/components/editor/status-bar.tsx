import { useEditorStore } from "@/stores/editor-store";

export function StatusBar() {
  const meshData = useEditorStore((s) => s.meshData);
  const mode = useEditorStore((s) => s.mode);
  const selectedFaceIndex = useEditorStore((s) => s.selectedFaceIndex);
  const activeOperation = useEditorStore((s) => s.activeOperation);

  let statusText = "Ready";
  if (activeOperation) {
    statusText = `Editing ${activeOperation.type}`;
  } else if (mode === "select-face" && selectedFaceIndex !== null) {
    statusText = `Face ${selectedFaceIndex} selected`;
  } else if (mode === "select-face") {
    statusText = "Select a face";
  }

  return (
    <div className="flex items-center justify-between bg-[var(--cad-bg-panel)] border-t border-[var(--cad-border)] px-3 text-[10px] text-[var(--cad-text-muted)]">
      <span data-testid="status-text">{statusText}</span>
      <div className="flex items-center gap-3">
        {meshData && meshData.vertexCount > 0 && (
          <>
            <span data-testid="vertex-count">Verts: {meshData.vertexCount}</span>
            <span>Tris: {meshData.triangleCount}</span>
          </>
        )}
        <span>mm</span>
      </div>
    </div>
  );
}
