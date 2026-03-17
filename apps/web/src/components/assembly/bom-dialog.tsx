import { useState } from "react";
import { X, Download, Search } from "lucide-react";
import { useAssemblyStore } from "@/stores/assembly-store";

export function BomDialog() {
  const bomData = useAssemblyStore((s) => s.bomData);
  const hideBom = useAssemblyStore((s) => s.hideBom);
  const exportBomCsv = useAssemblyStore((s) => s.exportBomCsv);

  const [filter, setFilter] = useState("");

  if (!bomData) return null;

  const filtered = filter
    ? bomData.filter((e) => e.part_name.toLowerCase().includes(filter.toLowerCase()))
    : bomData;

  const totalParts = filtered.reduce((sum, e) => sum + e.quantity, 0);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" data-testid="bom-dialog">
      <div className="w-[450px] rounded-lg border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-[var(--cad-border)] px-4 py-3">
          <span className="text-sm font-medium text-[var(--cad-text-primary)]">Bill of Materials</span>
          <div className="flex items-center gap-1">
            <button
              onClick={exportBomCsv}
              className="rounded p-1 hover:bg-white/10"
              title="Export CSV"
              data-testid="bom-export-csv"
            >
              <Download size={14} className="text-[var(--cad-text-muted)]" />
            </button>
            <button onClick={hideBom} data-testid="bom-close" className="rounded p-1 hover:bg-white/10">
              <X size={16} className="text-[var(--cad-text-muted)]" />
            </button>
          </div>
        </div>

        {/* Filter */}
        <div className="px-4 py-2 border-b border-[var(--cad-border)]">
          <div className="relative">
            <Search size={12} className="absolute left-2 top-1/2 -translate-y-1/2 text-[var(--cad-text-muted)]" />
            <input
              type="text"
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              placeholder="Filter parts..."
              className="w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] pl-6 pr-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none"
              data-testid="bom-filter"
            />
          </div>
        </div>

        {/* Table */}
        <div className="p-4">
          <table className="w-full text-xs" data-testid="bom-table">
            <thead>
              <tr className="border-b border-[var(--cad-border)]">
                <th className="py-1 text-left text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]">#</th>
                <th className="py-1 text-left text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]">Part</th>
                <th className="py-1 text-right text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]">Qty</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((entry, i) => (
                <tr key={entry.part_id} className="border-b border-[var(--cad-border)]/50">
                  <td className="py-1.5 text-[var(--cad-text-muted)]">{i + 1}</td>
                  <td className="py-1.5 text-[var(--cad-text-primary)]">{entry.part_name}</td>
                  <td className="py-1.5 text-right text-[var(--cad-text-primary)]">{entry.quantity}</td>
                </tr>
              ))}
            </tbody>
          </table>

          {filtered.length === 0 && (
            <p className="py-4 text-center text-[var(--cad-text-muted)]">
              {bomData.length === 0 ? "No components in assembly" : "No matching parts"}
            </p>
          )}

          <div className="mt-3 flex justify-between text-[10px] text-[var(--cad-text-muted)]">
            <span>{filtered.length} unique parts</span>
            <span>{totalParts} total components</span>
          </div>
        </div>
      </div>
    </div>
  );
}
