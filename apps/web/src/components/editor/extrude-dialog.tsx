import { useState } from "react";
import { useEditorStore } from "@/stores/editor-store";

interface ExtrudeDialogProps {
  open: boolean;
  onClose: () => void;
}

export function ExtrudeDialog({ open, onClose }: ExtrudeDialogProps) {
  const [depth, setDepth] = useState(10);
  const addFeature = useEditorStore((s) => s.addFeature);

  if (!open) return null;

  const handleCreate = () => {
    addFeature("extrude", "Extrude", {
      type: "extrude",
      params: {
        direction: [0, 0, 1],
        depth,
        symmetric: false,
        draft_angle: 0,
      },
    });
    onClose();
  };

  return (
    <div className="absolute left-1/2 top-1/2 z-50 -translate-x-1/2 -translate-y-1/2 rounded-lg border border-white/10 bg-[#1a1a2e] p-4 shadow-xl">
      <h3 className="mb-3 text-sm font-semibold text-white">Extrude</h3>
      <label className="mb-2 block text-xs text-white/60">
        Depth
        <input
          type="number"
          value={depth}
          onChange={(e) => setDepth(Number(e.target.value))}
          className="mt-1 block w-full rounded border border-white/10 bg-white/5 px-2 py-1 text-sm text-white"
          min={0.1}
          step={0.5}
        />
      </label>
      <div className="mt-3 flex gap-2">
        <button
          onClick={handleCreate}
          className="rounded bg-blue-600 px-3 py-1 text-xs text-white hover:bg-blue-700"
        >
          OK
        </button>
        <button
          onClick={onClose}
          className="rounded bg-white/10 px-3 py-1 text-xs text-white/60 hover:bg-white/20"
        >
          Cancel
        </button>
      </div>
    </div>
  );
}
