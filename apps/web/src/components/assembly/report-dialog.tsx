import { X, Download } from "lucide-react";
import { useAssemblyStore } from "@/stores/assembly-store";

export function ReportDialog() {
  const reportHtml = useAssemblyStore((s) => s.reportHtml);
  const hideReport = useAssemblyStore((s) => s.hideReport);

  if (!reportHtml) return null;

  const handleDownload = () => {
    const blob = new Blob([reportHtml], { type: "text/html" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "assembly-report.html";
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" data-testid="report-dialog">
      <div className="w-[600px] max-h-[80vh] rounded-lg border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] shadow-2xl flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-[var(--cad-border)] px-4 py-3">
          <span className="text-sm font-medium text-[var(--cad-text-primary)]">Assembly Report</span>
          <div className="flex items-center gap-1">
            <button onClick={handleDownload} className="rounded p-1 hover:bg-white/10" title="Download HTML" data-testid="report-download">
              <Download size={14} className="text-[var(--cad-text-muted)]" />
            </button>
            <button onClick={hideReport} data-testid="report-close" className="rounded p-1 hover:bg-white/10">
              <X size={16} className="text-[var(--cad-text-muted)]" />
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4">
          <iframe
            srcDoc={reportHtml}
            className="w-full h-96 border-0 bg-white rounded"
            title="Assembly Report"
            data-testid="report-iframe"
          />
        </div>
      </div>
    </div>
  );
}
