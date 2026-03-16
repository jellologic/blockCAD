import { Check, X } from "lucide-react";
import { useEditorStore } from "@/stores/editor-store";

export function ConfirmationCorner() {
  const mode = useEditorStore((s) => s.mode);
  const exitSketchMode = useEditorStore((s) => s.exitSketchMode);

  if (mode !== "sketch") return null;

  return (
    <div className="absolute top-3 right-3 z-20 flex gap-2">
      <button
        onClick={() => exitSketchMode(true)}
        className="flex items-center gap-1.5 rounded-md bg-[#22cc44] px-3 py-1.5 text-white text-xs font-medium shadow-lg hover:brightness-110 transition-all"
        title="Confirm Sketch (Enter)"
      >
        <Check size={16} />
        OK
      </button>
      <button
        onClick={() => exitSketchMode(false)}
        className="flex items-center gap-1.5 rounded-md bg-[#cc3333] px-3 py-1.5 text-white text-xs font-medium shadow-lg hover:brightness-110 transition-all"
        title="Cancel Sketch (Escape)"
      >
        <X size={16} />
        Cancel
      </button>
    </div>
  );
}
