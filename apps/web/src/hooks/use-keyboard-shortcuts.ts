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
  const enterSketchMode = useEditorStore((s) => s.enterSketchMode);
  const exitSketchMode = useEditorStore((s) => s.exitSketchMode);
  const setSketchTool = useEditorStore((s) => s.setSketchTool);
  const sketchSession = useEditorStore((s) => s.sketchSession);

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      // Don't capture when typing in inputs
      const tag = (e.target as HTMLElement).tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;

      switch (e.key) {
        case "Escape":
          if (mode === "sketch") {
            if (sketchSession?.activeTool) {
              setSketchTool(null);
            } else {
              exitSketchMode(false);
            }
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
            enterSketchMode("front");
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

        case "d":
        case "D":
          if (mode === "sketch" && !e.ctrlKey && !e.metaKey) {
            setSketchTool("dimension");
            e.preventDefault();
          }
          break;

        case "e":
        case "E":
          if (!activeOperation && mode !== "sketch" && !e.ctrlKey && !e.metaKey) {
            startOperation("extrude");
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
          if (!activeOperation && mode !== "sketch" && !e.ctrlKey && !e.metaKey) {
            setMode(mode === "select-face" ? "view" : "select-face");
            e.preventDefault();
          }
          break;

        case "F5":
          rebuild();
          e.preventDefault();
          break;

        case "Delete":
        case "Backspace":
          // Placeholder for delete selected feature
          break;
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [
    activeOperation,
    cancelOperation,
    confirmOperation,
    enterSketchMode,
    exitSketchMode,
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
  ]);
}
