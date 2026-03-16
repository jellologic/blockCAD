import { ArrowUpDown } from "lucide-react";
import { useEditorStore } from "@/stores/editor-store";
import { usePreferencesStore } from "@/stores/preferences-store";

export function ExtrudePanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const updateOperationParams = useEditorStore((s) => s.updateOperationParams);
  const unitSystem = usePreferencesStore((s) => s.unitSystem);

  if (!activeOperation || (activeOperation.type !== "extrude" && activeOperation.type !== "cut_extrude")) return null;

  const {
    direction = [0, 0, 1],
    depth = 10,
    symmetric = false,
    draft_angle = 0,
    end_condition = "blind",
    direction2_enabled = false,
    depth2 = 10,
    draft_angle2 = 0,
    end_condition2 = "blind",
    from_offset = 0,
    thin_feature = false,
    thin_wall_thickness = 1,
    target_face_index = null,
    surface_offset = 0,
    flip_side_to_cut = false,
    cap_ends = false,
    from_condition = "sketch_plane",
    from_face_index = null,
    contour_index = null,
  } = activeOperation.params;

  const draftEnabled = draft_angle !== 0 || activeOperation.params._draftEnabled;
  const draftOutward = activeOperation.params._draftOutward ?? false;

  const draftEnabled2 = draft_angle2 !== 0 || activeOperation.params._draftEnabled2;
  const draftOutward2 = activeOperation.params._draftOutward2 ?? false;

  const flipDirection = () => {
    updateOperationParams({ direction: direction.map((v: number) => -v) });
  };

  const inputClass =
    "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass =
    "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  return (
    <div className="space-y-3" data-testid="extrude-panel">
      {/* From section */}
      <div>
        <h4 className={sectionHeaderClass}>From</h4>
        <select
          data-testid="extrude-from-select"
          className={inputClass}
          value={from_condition}
          onChange={(e) => {
            const val = e.target.value;
            updateOperationParams({
              from_condition: val,
              from_offset: val === "sketch_plane" ? 0 : activeOperation.params.from_offset ?? 0,
            });
            // If Surface or Vertex selected, enter face selection mode
            if (val === "surface" || val === "vertex") {
              useEditorStore.getState().setMode("select-face");
            }
          }}
        >
          <option value="sketch_plane">Sketch Plane</option>
          <option value="offset">Offset</option>
          <option value="surface">Surface/Face/Plane</option>
          <option value="vertex">Vertex</option>
        </select>
        {from_condition === "offset" && (
          <div className="mt-1.5 flex items-center gap-1">
            <input
              type="number"
              value={from_offset}
              onChange={(e) =>
                updateOperationParams({ from_offset: Number(e.target.value) })
              }
              data-testid="extrude-from-offset"
              className={inputClass}
              step={0.5}
            />
            <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">
              {unitSystem}
            </span>
          </div>
        )}
        {(from_condition === "surface" || from_condition === "vertex") && (
          <div className="mt-2">
            <button
              onClick={() => {
                const store = useEditorStore.getState();
                store.setMode(store.mode === "select-face" ? "view" : "select-face");
              }}
              data-testid="extrude-from-face-select"
              className="w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-secondary)] hover:bg-[var(--cad-bg-hover)] transition-colors"
            >
              {from_face_index != null ? `Face ${from_face_index} selected` : "Click to select face..."}
            </button>
          </div>
        )}
      </div>

      {/* Direction section */}
      <div>
        <h4 className={sectionHeaderClass}>Direction</h4>
        <div className="flex items-center gap-1">
          <select
            data-testid="extrude-end-condition"
            className={inputClass}
            value={end_condition}
            onChange={(e) => {
              updateOperationParams({ end_condition: e.target.value });
            }}
          >
            <option value="blind">Blind</option>
            <option value="through_all">Through All</option>
            <option value="up_to_next">Up To Next</option>
            <option value="up_to_surface">Up To Surface</option>
            <option value="offset_from_surface">Offset From Surface</option>
            <option value="up_to_vertex">Up To Vertex</option>
          </select>
          <button
            onClick={flipDirection}
            data-testid="extrude-flip-direction"
            className="flex-shrink-0 rounded border border-[var(--cad-border)] p-1 text-[var(--cad-text-secondary)] transition-colors hover:bg-[var(--cad-bg-hover)] hover:text-[var(--cad-text-primary)]"
            title="Flip direction"
          >
            <ArrowUpDown size={14} />
          </button>
        </div>
        {(end_condition === "up_to_surface" || end_condition === "offset_from_surface" || end_condition === "up_to_vertex") && (
          <div className="mt-2">
            <button
              onClick={() => {
                const store = useEditorStore.getState();
                store.setMode(store.mode === "select-face" ? "view" : "select-face");
              }}
              data-testid="extrude-select-face"
              className={`w-full rounded border px-2 py-1 text-xs transition-colors border-[var(--cad-border)] bg-[var(--cad-bg-panel)] text-[var(--cad-text-secondary)] hover:bg-[var(--cad-bg-hover)]`}
            >
              {target_face_index != null ? `Face ${target_face_index} selected` : "Click to select face..."}
            </button>
          </div>
        )}
        {end_condition === "offset_from_surface" && (
          <div className="mt-2 flex items-center gap-1">
            <input
              type="number"
              value={surface_offset}
              onChange={(e) => updateOperationParams({ surface_offset: Number(e.target.value) })}
              data-testid="extrude-surface-offset"
              className={inputClass}
              step={0.5}
            />
            <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
          </div>
        )}
      </div>

      {/* Depth (hidden when Through All or Up To Next) */}
      {end_condition === "blind" && (
        <div>
          <label className={sectionHeaderClass}>Depth</label>
          <div className="flex items-center gap-1">
            <input
              type="number"
              value={depth}
              onChange={(e) =>
                updateOperationParams({ depth: Number(e.target.value) })
              }
              data-testid="extrude-depth"
              className={inputClass}
              min={0.1}
              step={0.5}
            />
            <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">
              {unitSystem}
            </span>
          </div>
        </div>
      )}

      {/* Mid Plane (Symmetric) */}
      <div>
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="extrude-symmetric"
            checked={symmetric}
            disabled={direction2_enabled}
            onChange={(e) =>
              updateOperationParams({ symmetric: e.target.checked })
            }
            data-testid="extrude-symmetric"
            className="rounded border-[var(--cad-border)]"
          />
          <label
            htmlFor="extrude-symmetric"
            className={`text-xs ${direction2_enabled ? "text-[var(--cad-text-muted)]" : "text-[var(--cad-text-secondary)]"}`}
          >
            Mid Plane
          </label>
        </div>
        <p className="mt-0.5 pl-5 text-[10px] text-[var(--cad-text-muted)]">
          Extrude equally in both directions
        </p>
      </div>

      {/* Flip side to cut */}
      {activeOperation.type === "cut_extrude" && (
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="extrude-flip-side"
            checked={flip_side_to_cut}
            onChange={(e) => updateOperationParams({ flip_side_to_cut: e.target.checked })}
            data-testid="extrude-flip-side-to-cut"
            className="rounded border-[var(--cad-border)]"
          />
          <label htmlFor="extrude-flip-side" className="text-xs text-[var(--cad-text-secondary)]">
            Flip side to cut
          </label>
        </div>
      )}

      {/* Direction 2 */}
      <div>
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="extrude-direction2-enabled"
            checked={direction2_enabled}
            disabled={symmetric}
            onChange={(e) =>
              updateOperationParams({ direction2_enabled: e.target.checked })
            }
            data-testid="extrude-direction2-enabled"
            className="rounded border-[var(--cad-border)]"
          />
          <label
            htmlFor="extrude-direction2-enabled"
            className={`text-xs ${symmetric ? "text-[var(--cad-text-muted)]" : "text-[var(--cad-text-secondary)]"}`}
          >
            Direction 2
          </label>
        </div>
        <p className="mt-0.5 pl-5 text-[10px] text-[var(--cad-text-muted)]">
          Extrude in the opposite direction
        </p>

        {direction2_enabled && (
          <div className="mt-2 space-y-2 border-l-2 border-[var(--cad-border)] pl-3">
            {/* End condition 2 */}
            <div>
              <label className={sectionHeaderClass}>End Condition</label>
              <select
                data-testid="extrude-end-condition2"
                className={inputClass}
                value={end_condition2}
                onChange={(e) => {
                  updateOperationParams({ end_condition2: e.target.value });
                }}
              >
                <option value="blind">Blind</option>
                <option value="through_all">Through All</option>
                <option value="up_to_next">Up To Next</option>
                <option value="up_to_surface">Up To Surface</option>
                <option value="offset_from_surface">Offset From Surface</option>
                <option value="up_to_vertex">Up To Vertex</option>
              </select>
            </div>

            {/* Depth 2 (hidden when Through All or Up To Next) */}
            {end_condition2 === "blind" && (
              <div>
                <label className={sectionHeaderClass}>Depth</label>
                <div className="flex items-center gap-1">
                  <input
                    type="number"
                    value={depth2}
                    onChange={(e) =>
                      updateOperationParams({ depth2: Number(e.target.value) })
                    }
                    data-testid="extrude-depth2"
                    className={inputClass}
                    min={0.1}
                    step={0.5}
                  />
                  <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">
                    {unitSystem}
                  </span>
                </div>
              </div>
            )}

            {/* Draft angle 2 */}
            <div>
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="extrude-draft-enabled2"
                  checked={draftEnabled2}
                  onChange={(e) => {
                    const checked = e.target.checked;
                    updateOperationParams({
                      _draftEnabled2: checked,
                      draft_angle2: checked ? activeOperation.params._lastDraftAngle2 ?? 5 : 0,
                    });
                  }}
                  data-testid="extrude-draft-enabled2"
                  className="rounded border-[var(--cad-border)]"
                />
                <label
                  htmlFor="extrude-draft-enabled2"
                  className="text-xs text-[var(--cad-text-secondary)]"
                >
                  Draft
                </label>
              </div>

              {draftEnabled2 && (
                <>
                  <div className="mt-1.5 flex items-center gap-1 pl-5">
                    <input
                      type="number"
                      value={Math.abs(draft_angle2)}
                      onChange={(e) => {
                        const raw = Math.min(45, Math.max(0, Number(e.target.value)));
                        updateOperationParams({
                          draft_angle2: draftOutward2 ? -raw : raw,
                          _lastDraftAngle2: raw,
                        });
                      }}
                      data-testid="extrude-draft-angle2"
                      className={inputClass}
                      min={0}
                      max={45}
                      step={0.5}
                    />
                    <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">
                      °
                    </span>
                  </div>
                  <div className="mt-1 flex items-center gap-2 pl-5">
                    <input
                      type="checkbox"
                      id="extrude-draft-outward2"
                      checked={draftOutward2}
                      onChange={(e) => {
                        const outward = e.target.checked;
                        const absAngle = Math.abs(draft_angle2);
                        updateOperationParams({
                          _draftOutward2: outward,
                          draft_angle2: outward ? -absAngle : absAngle,
                        });
                      }}
                      data-testid="extrude-draft-outward2"
                      className="rounded border-[var(--cad-border)]"
                    />
                    <label
                      htmlFor="extrude-draft-outward2"
                      className="text-xs text-[var(--cad-text-secondary)]"
                    >
                      Draft outward
                    </label>
                  </div>
                </>
              )}
            </div>
          </div>
        )}
      </div>

      {/* Draft section (Direction 1) */}
      <div>
        <h4 className={sectionHeaderClass}>Draft</h4>
        <div className="space-y-2">
          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              id="extrude-draft-enabled"
              checked={draftEnabled}
              onChange={(e) => {
                const checked = e.target.checked;
                updateOperationParams({
                  _draftEnabled: checked,
                  draft_angle: checked ? activeOperation.params._lastDraftAngle ?? 5 : 0,
                });
              }}
              data-testid="extrude-draft-enabled"
              className="rounded border-[var(--cad-border)]"
            />
            <label
              htmlFor="extrude-draft-enabled"
              className="text-xs text-[var(--cad-text-secondary)]"
            >
              Draft
            </label>
          </div>

          {draftEnabled && (
            <>
              <div className="flex items-center gap-1 pl-5">
                <input
                  type="number"
                  value={Math.abs(draft_angle)}
                  onChange={(e) => {
                    const raw = Math.min(45, Math.max(0, Number(e.target.value)));
                    updateOperationParams({
                      draft_angle: draftOutward ? -raw : raw,
                      _lastDraftAngle: raw,
                    });
                  }}
                  data-testid="extrude-draft-angle"
                  className={inputClass}
                  min={0}
                  max={45}
                  step={0.5}
                />
                <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">
                  °
                </span>
              </div>
              <div className="flex items-center gap-2 pl-5">
                <input
                  type="checkbox"
                  id="extrude-draft-outward"
                  checked={draftOutward}
                  onChange={(e) => {
                    const outward = e.target.checked;
                    const absAngle = Math.abs(draft_angle);
                    updateOperationParams({
                      _draftOutward: outward,
                      draft_angle: outward ? -absAngle : absAngle,
                    });
                  }}
                  data-testid="extrude-draft-outward"
                  className="rounded border-[var(--cad-border)]"
                />
                <label
                  htmlFor="extrude-draft-outward"
                  className="text-xs text-[var(--cad-text-secondary)]"
                >
                  Draft outward
                </label>
              </div>
            </>
          )}
        </div>
      </div>

      {/* Thin Feature section */}
      <div>
        <h4 className={sectionHeaderClass}>Thin Feature</h4>
        <div className="space-y-2">
          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              id="extrude-thin-feature"
              checked={thin_feature}
              onChange={(e) => updateOperationParams({ thin_feature: e.target.checked })}
              data-testid="extrude-thin-feature"
              className="rounded border-[var(--cad-border)]"
            />
            <label htmlFor="extrude-thin-feature" className="text-xs text-[var(--cad-text-secondary)]">
              Thin Feature
            </label>
          </div>
          {thin_feature && (
            <>
              <div className="flex items-center gap-1 pl-5">
                <input
                  type="number"
                  value={thin_wall_thickness}
                  onChange={(e) => updateOperationParams({ thin_wall_thickness: Math.max(0.1, Number(e.target.value)) })}
                  data-testid="extrude-thin-wall-thickness"
                  className={inputClass}
                  min={0.1}
                  step={0.5}
                />
                <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">
                  {unitSystem}
                </span>
              </div>
              <div className="flex items-center gap-2 pl-5">
                <input
                  type="checkbox"
                  id="extrude-cap-ends"
                  checked={cap_ends}
                  onChange={(e) => updateOperationParams({ cap_ends: e.target.checked })}
                  data-testid="extrude-cap-ends"
                  className="rounded border-[var(--cad-border)]"
                />
                <label htmlFor="extrude-cap-ends" className="text-xs text-[var(--cad-text-secondary)]">
                  Cap ends
                </label>
              </div>
            </>
          )}
        </div>
      </div>

      {/* Selected Contours */}
      <div>
        <h4 className={sectionHeaderClass}>Selected Contours</h4>
        <p className="text-[10px] text-[var(--cad-text-muted)]">
          {contour_index != null ? `Contour ${contour_index} selected` : "All contours (default)"}
        </p>
      </div>
    </div>
  );
}
