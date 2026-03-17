import { useState, useEffect } from "react";
import { useEditorStore } from "@/stores/editor-store";
import {
  getFilletRadius,
  setFilletRadius,
  getChamferDistance,
  setChamferDistance,
  getLinearPatternCount,
  setLinearPatternCount,
  getCircularPatternCount,
  setCircularPatternCount,
} from "./tools";

const inputClass =
  "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
const sectionHeaderClass =
  "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

function FilletToolPanel() {
  const [radius, setRadius] = useState(getFilletRadius);

  useEffect(() => {
    setFilletRadius(radius);
  }, [radius]);

  return (
    <div className="space-y-2" data-testid="sketch-fillet-panel">
      <div>
        <label className={sectionHeaderClass}>Fillet Radius</label>
        <div className="flex items-center gap-1">
          <input
            type="number"
            value={radius}
            onChange={(e) => setRadius(Math.max(0.01, Number(e.target.value)))}
            data-testid="sketch-fillet-radius"
            className={inputClass}
            min={0.01}
            step={0.5}
          />
          <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">mm</span>
        </div>
      </div>
      <p className="text-[10px] text-[var(--cad-text-muted)]">
        Click near a line-line intersection to apply the fillet.
      </p>
    </div>
  );
}

function ChamferToolPanel() {
  const [distance, setDistance] = useState(getChamferDistance);

  useEffect(() => {
    setChamferDistance(distance);
  }, [distance]);

  return (
    <div className="space-y-2" data-testid="sketch-chamfer-panel">
      <div>
        <label className={sectionHeaderClass}>Chamfer Distance</label>
        <div className="flex items-center gap-1">
          <input
            type="number"
            value={distance}
            onChange={(e) => setDistance(Math.max(0.01, Number(e.target.value)))}
            data-testid="sketch-chamfer-distance"
            className={inputClass}
            min={0.01}
            step={0.5}
          />
          <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">mm</span>
        </div>
      </div>
      <p className="text-[10px] text-[var(--cad-text-muted)]">
        Click near a line-line intersection to apply the chamfer.
      </p>
    </div>
  );
}

function TrimToolPanel() {
  return (
    <div className="space-y-2" data-testid="sketch-trim-panel">
      <p className="text-[10px] text-[var(--cad-text-muted)]">
        Click on a line segment to trim the clicked portion between the two nearest intersections.
      </p>
    </div>
  );
}

function ExtendToolPanel() {
  return (
    <div className="space-y-2" data-testid="sketch-extend-panel">
      <p className="text-[10px] text-[var(--cad-text-muted)]">
        Click near a line endpoint to extend it to the nearest intersection with another line.
      </p>
    </div>
  );
}

function OffsetToolPanel() {
  return (
    <div className="space-y-2" data-testid="sketch-offset-panel">
      <p className="text-[10px] text-[var(--cad-text-muted)]">
        Step 1: Click on a line to select it. Step 2: Click to set the offset side and distance.
      </p>
    </div>
  );
}

function MirrorToolPanel() {
  return (
    <div className="space-y-2" data-testid="sketch-mirror-panel">
      <p className="text-[10px] text-[var(--cad-text-muted)]">
        Step 1: Click on a line to mirror. Step 2: Click on the mirror axis line.
      </p>
    </div>
  );
}

function LinearPatternToolPanel() {
  const [count, setCount] = useState(getLinearPatternCount);

  useEffect(() => {
    setLinearPatternCount(count);
  }, [count]);

  return (
    <div className="space-y-2" data-testid="sketch-linear-pattern-panel">
      <div>
        <label className={sectionHeaderClass}>Instance Count</label>
        <input
          type="number"
          value={count}
          onChange={(e) => setCount(Math.max(2, Math.round(Number(e.target.value))))}
          data-testid="sketch-linear-pattern-count"
          className={inputClass}
          min={2}
          step={1}
        />
      </div>
      <p className="text-[10px] text-[var(--cad-text-muted)]">
        Step 1: Click near a line to select it. Step 2: Click to set direction and spacing.
      </p>
    </div>
  );
}

function CircularPatternToolPanel() {
  const [count, setCount] = useState(getCircularPatternCount);

  useEffect(() => {
    setCircularPatternCount(count);
  }, [count]);

  return (
    <div className="space-y-2" data-testid="sketch-circular-pattern-panel">
      <div>
        <label className={sectionHeaderClass}>Instance Count</label>
        <input
          type="number"
          value={count}
          onChange={(e) => setCount(Math.max(2, Math.round(Number(e.target.value))))}
          data-testid="sketch-circular-pattern-count"
          className={inputClass}
          min={2}
          step={1}
        />
      </div>
      <p className="text-[10px] text-[var(--cad-text-muted)]">
        Step 1: Click near a line to select it. Step 2: Click to set the rotation center.
      </p>
    </div>
  );
}

const TOOL_LABELS: Record<string, string> = {
  trim: "Trim",
  extend: "Extend",
  offset: "Offset",
  mirror: "Mirror",
  "sketch-fillet": "Sketch Fillet",
  "sketch-chamfer": "Sketch Chamfer",
  "sketch-linear-pattern": "Linear Pattern",
  "sketch-circular-pattern": "Circular Pattern",
};

const TOOL_PANELS: Record<string, React.FC> = {
  trim: TrimToolPanel,
  extend: ExtendToolPanel,
  offset: OffsetToolPanel,
  mirror: MirrorToolPanel,
  "sketch-fillet": FilletToolPanel,
  "sketch-chamfer": ChamferToolPanel,
  "sketch-linear-pattern": LinearPatternToolPanel,
  "sketch-circular-pattern": CircularPatternToolPanel,
};

/**
 * Panel displayed in the left sidebar when a sketch modification tool is active.
 * Shows parameter controls (radius, distance, count) and usage instructions.
 */
export function SketchToolPanel() {
  const activeTool = useEditorStore((s) => s.sketchSession?.activeTool);

  if (!activeTool || !TOOL_PANELS[activeTool]) return null;

  const PanelComponent = TOOL_PANELS[activeTool]!;
  const label = TOOL_LABELS[activeTool] ?? activeTool;

  return (
    <div className="border-t border-[var(--cad-border)] px-3 py-2">
      <h4 className="mb-2 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]">
        {label} Tool
      </h4>
      <PanelComponent />
    </div>
  );
}
