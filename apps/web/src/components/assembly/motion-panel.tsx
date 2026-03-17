import { useState } from "react";
import { Play, Pause, Square, X } from "lucide-react";
import { useAssemblyStore } from "@/stores/assembly-store";

export function MotionPanel() {
  const mates = useAssemblyStore((s) => s.mates);
  const motionFrames = useAssemblyStore((s) => s.motionFrames);
  const currentFrame = useAssemblyStore((s) => s.currentFrame);
  const isPlaying = useAssemblyStore((s) => s.isPlaying);
  const runMotionStudy = useAssemblyStore((s) => s.runMotionStudy);
  const playMotion = useAssemblyStore((s) => s.playMotion);
  const pauseMotion = useAssemblyStore((s) => s.pauseMotion);
  const stopMotion = useAssemblyStore((s) => s.stopMotion);
  const setFrame = useAssemblyStore((s) => s.setFrame);
  const cancelOp = useAssemblyStore((s) => s.cancelOp);

  const [driverMateId, setDriverMateId] = useState(mates[0]?.id || "");
  const [startValue, setStartValue] = useState(0);
  const [endValue, setEndValue] = useState(10);
  const [numSteps, setNumSteps] = useState(20);

  const inputClass =
    "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass =
    "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  const handleRun = () => {
    if (!driverMateId) return;
    runMotionStudy(driverMateId, startValue, endValue, numSteps);
  };

  const hasFrames = motionFrames.length > 0;
  const currentDriverValue = hasFrames
    ? motionFrames[currentFrame]?.driverValue.toFixed(3)
    : "--";

  return (
    <div className="flex h-full flex-col bg-[var(--cad-bg-panel-alt)] border-r border-[var(--cad-border)]">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-[var(--cad-border)] px-3 py-2">
        <span className="text-sm font-medium text-[var(--cad-text-primary)]">
          Motion Study
        </span>
        <button
          onClick={cancelOp}
          data-testid="motion-close"
          className="rounded p-1 transition-colors hover:bg-[var(--cad-cancel)]/20"
        >
          <X size={18} style={{ color: "var(--cad-cancel)" }} />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-3 space-y-3">
        {/* Driver mate selector */}
        <div>
          <label className={sectionHeaderClass}>Driver Mate</label>
          <select
            value={driverMateId}
            onChange={(e) => setDriverMateId(e.target.value)}
            className={inputClass}
            data-testid="motion-driver-select"
          >
            {mates.length === 0 && <option value="">No mates available</option>}
            {mates.map((m) => (
              <option key={m.id} value={m.id}>
                {m.kind}: {m.compA} / {m.compB}
              </option>
            ))}
          </select>
        </div>

        {/* Start / End values */}
        <div className="grid grid-cols-2 gap-2">
          <div>
            <label className={sectionHeaderClass}>Start Value</label>
            <input
              type="number"
              value={startValue}
              onChange={(e) => setStartValue(Number(e.target.value))}
              className={inputClass}
              data-testid="motion-start-value"
              step={0.1}
            />
          </div>
          <div>
            <label className={sectionHeaderClass}>End Value</label>
            <input
              type="number"
              value={endValue}
              onChange={(e) => setEndValue(Number(e.target.value))}
              className={inputClass}
              data-testid="motion-end-value"
              step={0.1}
            />
          </div>
        </div>

        {/* Number of steps */}
        <div>
          <label className={sectionHeaderClass}>Steps</label>
          <input
            type="number"
            value={numSteps}
            onChange={(e) => setNumSteps(Math.max(1, Number(e.target.value)))}
            className={inputClass}
            data-testid="motion-steps"
            min={1}
          />
        </div>

        {/* Run button */}
        <button
          onClick={handleRun}
          disabled={!driverMateId || mates.length === 0}
          data-testid="motion-run"
          className="w-full rounded bg-[var(--cad-accent)] px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-[var(--cad-accent)]/80 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Run Study
        </button>

        {/* Playback controls -- only visible when frames exist */}
        {hasFrames && (
          <div className="space-y-2 border-t border-[var(--cad-border)] pt-3">
            <label className={sectionHeaderClass}>Playback</label>

            {/* Transport buttons */}
            <div className="flex items-center gap-2">
              {isPlaying ? (
                <button
                  onClick={pauseMotion}
                  data-testid="motion-pause"
                  className="rounded p-1.5 bg-[var(--cad-accent)]/20 text-[var(--cad-accent)] hover:bg-[var(--cad-accent)]/30 transition-colors"
                  title="Pause"
                >
                  <Pause size={16} />
                </button>
              ) : (
                <button
                  onClick={playMotion}
                  data-testid="motion-play"
                  className="rounded p-1.5 bg-[var(--cad-accent)]/20 text-[var(--cad-accent)] hover:bg-[var(--cad-accent)]/30 transition-colors"
                  title="Play"
                >
                  <Play size={16} />
                </button>
              )}
              <button
                onClick={stopMotion}
                data-testid="motion-stop"
                className="rounded p-1.5 bg-white/10 text-[var(--cad-text-secondary)] hover:bg-white/20 transition-colors"
                title="Stop (reset to start)"
              >
                <Square size={16} />
              </button>

              {/* Frame indicator */}
              <span
                className="ml-auto text-[10px] text-[var(--cad-text-muted)]"
                data-testid="motion-frame-display"
              >
                Frame {currentFrame + 1} / {motionFrames.length}
              </span>
            </div>

            {/* Frame slider */}
            <input
              type="range"
              min={0}
              max={motionFrames.length - 1}
              value={currentFrame}
              onChange={(e) => setFrame(Number(e.target.value))}
              className="w-full accent-[var(--cad-accent)]"
              data-testid="motion-slider"
            />

            {/* Current driver value */}
            <div className="text-[10px] text-[var(--cad-text-muted)]">
              Driver value:{" "}
              <span className="text-[var(--cad-text-primary)] font-medium" data-testid="motion-current-value">
                {currentDriverValue}
              </span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
