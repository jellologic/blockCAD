import { useState, useRef, useEffect } from "react";
import { useEditorStore } from "@/stores/editor-store";

interface MenuItem {
  label: string;
  shortcut?: string;
  disabled?: boolean;
  separator?: boolean;
  submenu?: MenuItem[];
  onClick?: () => void;
}

function MenuDropdown({
  items,
  onClose,
}: {
  items: MenuItem[];
  onClose: () => void;
}) {
  return (
    <div className="absolute left-0 top-full z-[100] min-w-[200px] rounded-b-md border border-t-0 border-[var(--cad-border)] bg-[var(--cad-bg-panel)] shadow-lg py-1">
      {items.map((item, i) => {
        if (item.separator) {
          return <div key={i} className="my-1 h-px bg-[var(--cad-border)]" />;
        }
        return (
          <button
            key={item.label}
            onClick={() => {
              item.onClick?.();
              onClose();
            }}
            disabled={item.disabled}
            className={`flex w-full items-center justify-between px-3 py-1.5 text-xs transition-colors ${
              item.disabled
                ? "text-[var(--cad-text-muted)] cursor-not-allowed"
                : "text-[var(--cad-text-secondary)] hover:bg-white/10 hover:text-[var(--cad-text-primary)]"
            }`}
          >
            <span>{item.label}</span>
            {item.shortcut && (
              <span className="ml-4 text-[10px] text-[var(--cad-text-muted)]">{item.shortcut}</span>
            )}
          </button>
        );
      })}
    </div>
  );
}

export function MenuBar() {
  const [openMenu, setOpenMenu] = useState<string | null>(null);
  const barRef = useRef<HTMLDivElement>(null);

  const hasMesh = useEditorStore((s) => s.meshData !== null);
  const kernel = useEditorStore((s) => s.kernel);
  const exportSTL = useEditorStore((s) => s.exportSTL);
  const exportOBJ = useEditorStore((s) => s.exportOBJ);
  const export3MF = useEditorStore((s) => s.export3MF);
  const exportGLB = useEditorStore((s) => s.exportGLB);
  const exportSTEP = useEditorStore((s) => s.exportSTEP);
  const setCameraTarget = useEditorStore((s) => s.setCameraTarget);
  const fitAll = useEditorStore((s) => s.fitAll);

  useEffect(() => {
    if (!openMenu) return;
    const handler = (e: MouseEvent) => {
      if (barRef.current && !barRef.current.contains(e.target as Node)) {
        setOpenMenu(null);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [openMenu]);

  const handleNew = () => {
    if (!confirm("Create a new part? Unsaved changes will be lost.")) return;
    window.location.reload();
  };

  const handleOpen = () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".blockcad,.json";
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;
      const text = await file.text();
      try {
        const { KernelClient } = await import("@blockCAD/kernel");
        const fresh = KernelClient.deserialize(text);
        const mesh = fresh.tessellate();
        useEditorStore.setState({
          kernel: fresh,
          meshData: mesh,
          features: fresh.featureList,
          selectedFeatureId: null,
        });
      } catch (err) {
        alert("Failed to open file: " + (err instanceof Error ? err.message : String(err)));
      }
    };
    input.click();
  };

  const handleSave = () => {
    if (!kernel) return;
    const json = kernel.serialize();
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "part.blockcad";
    a.click();
    URL.revokeObjectURL(url);
  };

  const menus: { label: string; items: MenuItem[] }[] = [
    {
      label: "File",
      items: [
        { label: "New", shortcut: "Ctrl+N", onClick: handleNew },
        { label: "Open...", shortcut: "Ctrl+O", onClick: handleOpen },
        { label: "Save", shortcut: "Ctrl+S", onClick: handleSave, disabled: !kernel },
        { separator: true, label: "" },
        { label: "Export STL", onClick: () => exportSTL(true), disabled: !hasMesh },
        { label: "Export OBJ", onClick: () => exportOBJ(), disabled: !hasMesh },
        { label: "Export 3MF", onClick: () => export3MF(), disabled: !hasMesh },
        { label: "Export GLB", onClick: () => exportGLB(), disabled: !hasMesh },
        { label: "Export STEP", onClick: () => exportSTEP(), disabled: !hasMesh },
      ],
    },
    {
      label: "Edit",
      items: [
        { label: "Undo", shortcut: "Ctrl+Z", disabled: true },
        { label: "Redo", shortcut: "Ctrl+Y", disabled: true },
      ],
    },
    {
      label: "View",
      items: [
        { label: "Front", shortcut: "1", onClick: () => setCameraTarget([0, 0, 30]) },
        { label: "Back", onClick: () => setCameraTarget([0, 0, -30]) },
        { label: "Left", onClick: () => setCameraTarget([-30, 0, 0]) },
        { label: "Right", shortcut: "3", onClick: () => setCameraTarget([30, 0, 0]) },
        { label: "Top", shortcut: "5", onClick: () => setCameraTarget([0, 30, 0]) },
        { label: "Bottom", onClick: () => setCameraTarget([0, -30, 0]) },
        { separator: true, label: "" },
        { label: "Isometric", shortcut: "0", onClick: () => setCameraTarget([20, 15, 20]) },
        { label: "Fit All", shortcut: ".", onClick: fitAll },
      ],
    },
    {
      label: "Help",
      items: [
        {
          label: "Keyboard Shortcuts",
          onClick: () => {
            useEditorStore.setState({ showShortcutsHelp: true } as any);
          },
        },
      ],
    },
  ];

  return (
    <div ref={barRef} className="flex items-center bg-[var(--cad-bg-panel)] border-b border-[var(--cad-border)] h-6">
      {menus.map((menu) => (
        <div key={menu.label} className="relative">
          <button
            onClick={() => setOpenMenu(openMenu === menu.label ? null : menu.label)}
            onMouseEnter={() => {
              if (openMenu) setOpenMenu(menu.label);
            }}
            className={`px-3 py-1 text-[11px] transition-colors ${
              openMenu === menu.label
                ? "bg-white/10 text-[var(--cad-text-primary)]"
                : "text-[var(--cad-text-secondary)] hover:bg-white/5 hover:text-[var(--cad-text-primary)]"
            }`}
          >
            {menu.label}
          </button>
          {openMenu === menu.label && (
            <MenuDropdown items={menu.items} onClose={() => setOpenMenu(null)} />
          )}
        </div>
      ))}
    </div>
  );
}
