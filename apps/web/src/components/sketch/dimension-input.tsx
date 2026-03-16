import { useState, useEffect } from "react";
import { Html } from "@react-three/drei";
import * as THREE from "three";
import { useEditorStore } from "@/stores/editor-store";
import {
  usePreferencesStore,
  parseUserInput,
  parseAngleInput,
} from "@/stores/preferences-store";

function sketchToWorld(
  position: { x: number; y: number },
  plane: { origin: number[]; uAxis: number[]; vAxis: number[] }
): THREE.Vector3 {
  return new THREE.Vector3(
    plane.origin[0]! + position.x * plane.uAxis[0]! + position.y * plane.vAxis[0]!,
    plane.origin[1]! + position.x * plane.uAxis[1]! + position.y * plane.vAxis[1]!,
    plane.origin[2]! + position.x * plane.uAxis[2]! + position.y * plane.vAxis[2]!
  );
}

/** Fusion 360 style: minimal inline input at dimension position */
function InlineDimensionInput() {
  const sketchSession = useEditorStore((s) => s.sketchSession);
  const confirmDimension = useEditorStore((s) => s.confirmDimension);
  const cancelDimension = useEditorStore((s) => s.cancelDimension);
  const unitSystem = usePreferencesStore((s) => s.unitSystem);
  const [value, setValue] = useState("");

  // Pre-fill with existing value if editing
  useEffect(() => {
    setValue("");
  }, [sketchSession?.dimensionInput]);

  if (!sketchSession?.dimensionInput) return null;

  const { position, kind } = sketchSession.dimensionInput;
  const worldPos = sketchToWorld(position, sketchSession.plane);
  const isAngle = kind === "angle";

  const handleSubmit = () => {
    let mmValue: number | null;
    if (isAngle) {
      mmValue = parseAngleInput(value);
    } else {
      mmValue = parseUserInput(value);
    }
    if (mmValue !== null && mmValue > 0) {
      confirmDimension(mmValue);
      setValue("");
    }
  };

  return (
    <Html position={worldPos} center>
      <div className="flex items-center gap-1 rounded-md border border-[var(--cad-accent)] bg-[var(--cad-bg-panel)] p-1.5 shadow-xl">
        <input
          autoFocus
          type="text"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") handleSubmit();
            if (e.key === "Escape") {
              cancelDimension();
              setValue("");
            }
            e.stopPropagation();
          }}
          className="w-24 rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel-alt)] px-2 py-1 text-xs text-[var(--cad-text-primary)] font-mono focus:outline-none focus:border-[var(--cad-accent)]"
          placeholder={isAngle ? "45" : "25"}
        />
        <span className="text-[10px] text-[var(--cad-text-muted)] min-w-[20px]">
          {isAngle ? "°" : unitSystem}
        </span>
      </div>
    </Html>
  );
}

/** SolidWorks style: dialog panel with more controls */
function DialogDimensionInput() {
  const sketchSession = useEditorStore((s) => s.sketchSession);
  const confirmDimension = useEditorStore((s) => s.confirmDimension);
  const cancelDimension = useEditorStore((s) => s.cancelDimension);
  const unitSystem = usePreferencesStore((s) => s.unitSystem);
  const [value, setValue] = useState("");
  const [dimName] = useState(() => `D${Math.floor(Math.random() * 100)}`);

  useEffect(() => {
    setValue("");
  }, [sketchSession?.dimensionInput]);

  if (!sketchSession?.dimensionInput) return null;

  const { position, kind } = sketchSession.dimensionInput;
  const worldPos = sketchToWorld(position, sketchSession.plane);
  const isAngle = kind === "angle";

  const handleSubmit = () => {
    let mmValue: number | null;
    if (isAngle) {
      mmValue = parseAngleInput(value);
    } else {
      mmValue = parseUserInput(value);
    }
    if (mmValue !== null && mmValue > 0) {
      confirmDimension(mmValue);
      setValue("");
    }
  };

  return (
    <Html position={worldPos} center>
      <div className="rounded-lg border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] p-3 shadow-xl min-w-[180px]">
        <div className="text-[10px] text-[var(--cad-text-muted)] mb-1.5 font-medium">
          Modify Dimension
        </div>
        <div className="text-[9px] text-[var(--cad-text-muted)] mb-2">{dimName}</div>
        <div className="flex items-center gap-1.5 mb-2">
          <input
            autoFocus
            type="text"
            value={value}
            onChange={(e) => setValue(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleSubmit();
              if (e.key === "Escape") {
                cancelDimension();
                setValue("");
              }
              e.stopPropagation();
            }}
            className="flex-1 rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel-alt)] px-2 py-1 text-xs text-[var(--cad-text-primary)] font-mono focus:outline-none focus:border-[var(--cad-accent)]"
            placeholder={isAngle ? "45" : "25"}
          />
          <span className="text-[10px] text-[var(--cad-text-muted)]">
            {isAngle ? "°" : unitSystem}
          </span>
        </div>
        <div className="flex justify-end gap-1">
          <button
            onClick={() => {
              cancelDimension();
              setValue("");
            }}
            className="rounded px-2 py-0.5 text-[10px] text-[var(--cad-text-muted)] hover:bg-white/10"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            className="rounded bg-[var(--cad-accent)] px-2 py-0.5 text-[10px] text-white hover:brightness-110"
          >
            OK
          </button>
        </div>
      </div>
    </Html>
  );
}

export function DimensionInputOverlay() {
  const interactionStyle = usePreferencesStore((s) => s.interactionStyle);

  if (interactionStyle === "solidworks") {
    return <DialogDimensionInput />;
  }
  return <InlineDimensionInput />;
}
