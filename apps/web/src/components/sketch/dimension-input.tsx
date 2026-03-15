import { useState } from "react";
import { Html } from "@react-three/drei";
import * as THREE from "three";
import { useEditorStore } from "@/stores/editor-store";

export function DimensionInputOverlay() {
  const sketchSession = useEditorStore((s) => s.sketchSession);
  const confirmDimension = useEditorStore((s) => s.confirmDimension);
  const cancelDimension = useEditorStore((s) => s.cancelDimension);
  const [value, setValue] = useState("");

  if (!sketchSession?.dimensionInput) return null;

  const { position } = sketchSession.dimensionInput;
  const plane = sketchSession.plane;

  // Convert 2D sketch position to 3D world position
  const worldPos = new THREE.Vector3(
    plane.origin[0] + position.x * plane.uAxis[0] + position.y * plane.vAxis[0],
    plane.origin[1] + position.x * plane.uAxis[1] + position.y * plane.vAxis[1],
    plane.origin[2] + position.x * plane.uAxis[2] + position.y * plane.vAxis[2]
  );

  const handleSubmit = () => {
    const num = parseFloat(value);
    if (!isNaN(num) && num > 0) {
      confirmDimension(num);
      setValue("");
    }
  };

  return (
    <Html position={worldPos} center>
      <div className="flex items-center gap-1 rounded border border-[var(--cad-accent)] bg-[var(--cad-bg-panel)] p-1 shadow-lg">
        <input
          autoFocus
          type="number"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") handleSubmit();
            if (e.key === "Escape") {
              cancelDimension();
              setValue("");
            }
            e.stopPropagation(); // prevent sketch shortcuts
          }}
          className="w-20 rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel-alt)] px-1.5 py-0.5 text-xs text-[var(--cad-text-primary)]"
          placeholder="mm"
          min={0.01}
          step={0.5}
        />
        <span className="text-[10px] text-[var(--cad-text-muted)]">mm</span>
      </div>
    </Html>
  );
}
