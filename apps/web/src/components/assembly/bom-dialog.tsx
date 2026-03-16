import { X } from "lucide-react";
import { useAssemblyStore } from "@/stores/assembly-store";

export function BomDialog() {
  const bomData = useAssemblyStore((s) => s.bomData);
  const hideBom = useAssemblyStore((s) => s.hideBom);

  if (!bomData) return null;

  const totalParts = bomData.reduce((sum, e) => sum + e.quantity, 0);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" data-testid="bom-dialog">
      <div className="w-[400px] rounded-lg border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-[var(--cad-border)] px-4 py-3">
          <span className="text-sm font-medium text-[var(--cad-text-primary)]">Bill of Materials</span>
          <button onClick={hideBom} data-testid="bom-close" className="rounded p-1 hover:bg-white/10">
            <X size={16} className="text-[var(--cad-text-muted)]" />
          </button>
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
              {bomData.map((entry, i) => (
                <tr key={entry.part_id} className="border-b border-[var(--cad-border)]/50">
                  <td className="py-1.5 text-[var(--cad-text-muted)]">{i + 1}</td>
                  <td className="py-1.5 text-[var(--cad-text-primary)]">{entry.part_name}</td>
                  <td className="py-1.5 text-right text-[var(--cad-text-primary)]">{entry.quantity}</td>
                </tr>
              ))}
            </tbody>
          </table>

          {bomData.length === 0 && (
            <p className="py-4 text-center text-[var(--cad-text-muted)]">No components in assembly</p>
          )}

          <div className="mt-3 flex justify-between text-[10px] text-[var(--cad-text-muted)]">
            <span>{bomData.length} unique parts</span>
            <span>{totalParts} total components</span>
          </div>
        </div>
      </div>
    </div>
  );
}
