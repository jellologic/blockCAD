import { useState } from "react";
import { Plus, Check, Settings } from "lucide-react";
import { useAssemblyStore } from "@/stores/assembly-store";

export function ConfigPanel() {
  const configurations = useAssemblyStore((s) => s.configurations);
  const activeConfig = useAssemblyStore((s) => s.activeConfigIndex);
  const addConfiguration = useAssemblyStore((s) => s.addConfiguration);
  const activateConfiguration = useAssemblyStore((s) => s.activateConfiguration);

  const [newName, setNewName] = useState("");
  const [isAdding, setIsAdding] = useState(false);

  const inputClass =
    "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";

  const handleAdd = () => {
    if (!newName.trim()) return;
    addConfiguration(newName.trim());
    setNewName("");
    setIsAdding(false);
  };

  return (
    <div className="border-b border-[var(--cad-border)] px-3 py-2" data-testid="config-panel">
      <div className="flex items-center gap-1 mb-1">
        <Settings size={12} className="text-[var(--cad-text-muted)]" />
        <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]">
          Configurations
        </span>
        <button
          onClick={() => setIsAdding(true)}
          className="ml-auto p-0.5 rounded hover:bg-white/10"
          title="Add Configuration"
          data-testid="config-add-btn"
        >
          <Plus size={12} className="text-[var(--cad-text-muted)]" />
        </button>
      </div>

      {configurations.length > 0 && (
        <select
          value={activeConfig ?? ""}
          onChange={(e) => {
            const idx = parseInt(e.target.value, 10);
            if (!isNaN(idx)) activateConfiguration(idx);
          }}
          className={inputClass}
          data-testid="config-select"
        >
          <option value="">-- Select Configuration --</option>
          {configurations.map((name, i) => (
            <option key={i} value={i}>
              {name}
            </option>
          ))}
        </select>
      )}

      {configurations.length === 0 && !isAdding && (
        <div className="text-[10px] text-[var(--cad-text-muted)] italic">No configurations</div>
      )}

      {isAdding && (
        <div className="flex gap-1 mt-1">
          <input
            type="text"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            placeholder="Config name..."
            className={inputClass}
            autoFocus
            data-testid="config-name-input"
            onKeyDown={(e) => e.key === "Enter" && handleAdd()}
          />
          <button
            onClick={handleAdd}
            className="rounded p-1 hover:bg-[var(--cad-confirm)]/20"
            data-testid="config-confirm"
          >
            <Check size={14} style={{ color: "var(--cad-confirm)" }} />
          </button>
        </div>
      )}
    </div>
  );
}
