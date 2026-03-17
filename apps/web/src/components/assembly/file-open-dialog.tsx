import { useRef } from "react";
import { FolderOpen } from "lucide-react";
import { useAssemblyStore } from "@/stores/assembly-store";
import { toast } from "sonner";

export function FileOpenButton() {
  const openAssemblyFile = useAssemblyStore((s) => s.openAssemblyFile);
  const inputRef = useRef<HTMLInputElement>(null);

  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    try {
      const text = await file.text();
      openAssemblyFile(text);
    } catch (err) {
      toast.error("Failed to read file: " + String(err));
    }

    // Reset input so the same file can be re-selected
    if (inputRef.current) inputRef.current.value = "";
  };

  return (
    <>
      <input
        ref={inputRef}
        type="file"
        accept=".blockcad-assembly,.json"
        onChange={handleFileSelect}
        className="hidden"
        data-testid="assembly-file-input"
      />
      <button
        onClick={() => inputRef.current?.click()}
        className="flex items-center gap-1 rounded px-2 py-1 text-xs text-[var(--cad-text-secondary)] hover:bg-white/5"
        title="Open Assembly File"
        data-testid="assembly-file-open"
      >
        <FolderOpen size={14} />
        <span>Open</span>
      </button>
    </>
  );
}
