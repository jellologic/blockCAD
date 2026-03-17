import { useEffect } from "react";
import { useEditorStore } from "@/stores/editor-store";

/**
 * Global keyboard shortcuts for the CAD editor.
 * Suppressed when an input/textarea/select is focused.
 */
export function useKeyboardShortcuts() {
  const startOperation = useEditorStore((s) => s.startOperation);
  const cancelOperation = useEditorStore((s) => s.cancelOperation);
  const confirmOperation = useEditorStore((s) => s.confirmOperation);
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const toggleWireframe = useEditorStore((s) => s.toggleWireframe);
  const toggleEdges = useEditorStore((s) => s.toggleEdges);
  const selectFeature = useEditorStore((s) => s.selectFeature);
  const selectFace = useEditorStore((s) => s.selectFace);
  const setMode = useEditorStore((s) => s.setMode);
  const mode = useEditorStore((s) => s.mode);
  const rebuild = useEditorStore((s) => s.rebuild);
  const startSketchFlow = useEditorStore((s) => s.startSketchFlow);
  const exitSketchMode = useEditorStore((s) => s.exitSketchMode);
  const setSketchTool = useEditorStore((s) => s.setSketchTool);
  const clearPendingPoints = useEditorStore((s) => s.clearPendingPoints);
  const sketchSession = useEditorStore((s) => s.sketchSession);
  const deleteFeature = useEditorStore((s) => s.deleteFeature);
  const features = useEditorStore((s) => s.features);
  const selectedFeatureId = useEditorStore((s) => s.selectedFeatureId);

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      // Don't capture when typing in inputs
      const tag = (e.target as HTMLElement).tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;

      switch (e.key) {
        case "Escape":
          if (mode === "sketch") {
            if (sketchSession?.activeTool) {
              // Level 1: Deactivate current tool, clear pending chain
              setSketchTool(null);
              clearPendingPoints();
            } else if (sketchSession && sketchSession.entities.length > 0) {
              // Level 2: Entities exist, no tool active → save and exit (like SolidWorks)
              exitSketchMode(true);
            } else {
              // Level 3: Empty sketch → just cancel
              exitSketchMode(false);
            }
            e.preventDefault();
          } else if (mode === "select-plane") {
            setMode("view");
            e.preventDefault();
          } else if (activeOperation) {
            cancelOperation();
            e.preventDefault();
          } else {
            selectFeature(null);
            selectFace(null);
            if (mode !== "view") setMode("view");
            e.preventDefault();
          }
          break;

        case "Enter":
          if (mode === "sketch") {
            exitSketchMode(true);
            e.preventDefault();
          } else if (activeOperation) {
            confirmOperation();
            e.preventDefault();
          }
          break;

        case "s":
        case "S":
          if (mode !== "sketch" && !activeOperation && !e.ctrlKey && !e.metaKey) {
            startSketchFlow();
            e.preventDefault();
          }
          break;

        case "l":
        case "L":
          if (mode === "sketch" && !e.ctrlKey && !e.metaKey) {
            setSketchTool("line");
            e.preventDefault();
          }
          break;

        case "r":
        case "R":
          if (mode === "sketch" && !e.ctrlKey && !e.metaKey) {
            setSketchTool("rectangle");
            e.preventDefault();
          }
          break;

        case "c":
        case "C":
          if (mode === "sketch" && !e.ctrlKey && !e.metaKey) {
            setSketchTool("circle");
            e.preventDefault();
          }
          break;

        case "a":
        case "A":
          if (mode === "sketch" && !e.ctrlKey && !e.metaKey) {
            setSketchTool("arc");
            e.preventDefault();
          }
          break;

        case "m":
        case "M":
          if (mode === "sketch" && !e.ctrlKey && !e.metaKey) {
            setSketchTool("measure");
            e.preventDefault();
          }
          break;

        case "t":
        case "T":
          if (mode === "sketch" && !e.ctrlKey && !e.metaKey) {
            setSketchTool("trim");
            e.preventDefault();
          }
          break;

        case "o":
        case "O":
          if (mode === "sketch" && !e.ctrlKey && !e.metaKey) {
            setSketchTool("offset");
            e.preventDefault();
          }
          break;

        case "p":
        case "P":
          if (mode === "sketch" && !e.ctrlKey && !e.metaKey) {
            setSketchTool("polygon");
            e.preventDefault();
          }
          break;

        case "i":
        case "I":
          if (mode === "sketch" && !e.ctrlKey && !e.metaKey) {
            setSketchTool("ellipse");
            e.preventDefault();
          }
          break;

        case "n":
        case "N":
          if (mode === "sketch" && !e.ctrlKey && !e.metaKey) {
            setSketchTool("slot");
            e.preventDefault();
          }
          break;

        case "e":
        case "E":
          if (mode === "sketch" && !e.ctrlKey && !e.metaKey) {
            setSketchTool("extend");
            e.preventDefault();
          } else if (!activeOperation && mode !== "sketch" && !e.ctrlKey && !e.metaKey) {
            startOperation("extrude");
            e.preventDefault();
          }
          break;

        case "x":
        case "X":
          if (!activeOperation && mode !== "sketch" && !e.ctrlKey && !e.metaKey) {
            startOperation("cut_extrude");
            e.preventDefault();
          }
          break;

        case "v":
        case "V":
          if (!activeOperation && mode !== "sketch" && !e.ctrlKey && !e.metaKey) {
            startOperation("revolve");
            e.preventDefault();
          }
          break;

        case "w":
        case "W":
          if (!activeOperation && mode !== "sketch" && !e.ctrlKey && !e.metaKey) {
            toggleWireframe();
            e.preventDefault();
          }
          break;

        case "f":
        case "F":
          if (mode === "sketch" && !e.ctrlKey && !e.metaKey) {
            setSketchTool("sketch-fillet");
            e.preventDefault();
          } else if (!activeOperation && mode !== "sketch" && !e.ctrlKey && !e.metaKey) {
            setMode(mode === "select-face" ? "view" : "select-face");
            e.preventDefault();
          }
          break;

        case "g":
        case "G":
          if (!activeOperation && mode !== "sketch" && !e.ctrlKey && !e.metaKey) {
            startOperation("fillet");
            e.preventDefault();
          }
          break;

        case "h":
        case "H":
          if (mode === "sketch" && !e.ctrlKey && !e.metaKey) {
            setSketchTool("sketch-chamfer");
            e.preventDefault();
          } else if (!activeOperation && mode !== "sketch" && !e.ctrlKey && !e.metaKey) {
            startOperation("chamfer");
            e.preventDefault();
          }
          break;

        case "z":
        case "Z":
          if (mode === "sketch" && (e.ctrlKey || e.metaKey) && !e.shiftKey) {
            useEditorStore.getState().undoSketch();
            e.preventDefault();
          } else if (mode === "sketch" && (e.ctrlKey || e.metaKey) && e.shiftKey) {
            useEditorStore.getState().redoSketch();
            e.preventDefault();
          }
          break;

        case "y":
        case "Y":
          if (mode === "sketch" && (e.ctrlKey || e.metaKey)) {
            useEditorStore.getState().redoSketch();
            e.preventDefault();
          }
          break;

        case "1":
          if (!activeOperation && mode !== "sketch") {
            useEditorStore.getState().setCameraTarget([0, 0, 30]); // Front
            e.preventDefault();
          }
          break;
        case "3":
          if (!activeOperation && mode !== "sketch") {
            useEditorStore.getState().setCameraTarget([30, 0, 0]); // Right
            e.preventDefault();
          }
          break;
        case "5":
          if (!activeOperation && mode !== "sketch") {
            useEditorStore.getState().setCameraTarget([0, 30, 0]); // Top
            e.preventDefault();
          }
          break;
        case "0":
          if (!activeOperation && mode !== "sketch") {
            useEditorStore.getState().setCameraTarget([20, 15, 20]); // Isometric
            e.preventDefault();
          }
          break;
        case ".":
          if (!activeOperation && mode !== "sketch") {
            useEditorStore.getState().fitAll();
            e.preventDefault();
          }
          break;

        case "F5":
          rebuild();
          e.preventDefault();
          break;

        case "Delete":
        case "Backspace":
          if (mode !== "sketch" && !activeOperation && selectedFeatureId) {
            const featureIdx = features.findIndex((f) => f.id === selectedFeatureId);
            if (featureIdx >= 0) {
              if (window.confirm(`Delete "${features[featureIdx].name}"?`)) {
                deleteFeature(featureIdx);
              }
              e.preventDefault();
            }
          }
          // In sketch mode, delete is handled by sketch overlay via selected entity IDs
          break;

        case "F2":
          if (mode !== "sketch" && !activeOperation && selectedFeatureId) {
            // Dispatch a custom event that the feature tree can listen for
            window.dispatchEvent(new CustomEvent("blockcad:rename-feature", {
              detail: { featureId: selectedFeatureId },
            }));
            e.preventDefault();
          }
          break;

        case "d":
        case "D":
          if (mode === "sketch" && !e.ctrlKey && !e.metaKey) {
            setSketchTool("dimension");
            e.preventDefault();
          } else if ((e.ctrlKey || e.metaKey) && !e.shiftKey && mode !== "sketch") {
            // Ctrl+D / Cmd+D: suppress/unsuppress selected feature
            if (selectedFeatureId) {
              const featureIdx = features.findIndex((f) => f.id === selectedFeatureId);
              if (featureIdx >= 0) {
                const feature = features[featureIdx];
                if (feature.suppressed) {
                  useEditorStore.getState().unsuppressFeature(featureIdx);
                } else {
                  useEditorStore.getState().suppressFeature(featureIdx);
                }
                e.preventDefault();
              }
            }
          }
          break;
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [
    activeOperation,
    cancelOperation,
    confirmOperation,
    startSketchFlow,
    exitSketchMode,
    clearPendingPoints,
    mode,
    rebuild,
    selectFace,
    selectFeature,
    setMode,
    setSketchTool,
    sketchSession,
    startOperation,
    toggleEdges,
    toggleWireframe,
    deleteFeature,
    features,
    selectedFeatureId,
  ]);
}
