import type { FeatureEntry } from "@blockCAD/kernel";

interface FeatureTreeProps {
  features: FeatureEntry[];
  selectedId: string | null;
  onSelect: (id: string) => void;
}

const FEATURE_ICONS: Record<string, string> = {
  sketch: "S",
  extrude: "E",
  revolve: "R",
  fillet: "F",
  chamfer: "C",
};

export function FeatureTree({
  features,
  selectedId,
  onSelect,
}: FeatureTreeProps) {
  return (
    <div className="flex h-full flex-col border-r border-white/10 bg-[#12121a]">
      <div className="border-b border-white/10 px-4 py-3">
        <h2 className="text-sm font-semibold text-white/70 uppercase tracking-wider">
          Feature Tree
        </h2>
      </div>
      <div className="flex-1 overflow-y-auto p-2">
        {features.map((feature, index) => (
          <button
            key={feature.id}
            onClick={() => onSelect(feature.id)}
            className={`flex w-full items-center gap-3 rounded-md px-3 py-2 text-left text-sm transition-colors ${
              selectedId === feature.id
                ? "bg-blue-600/20 text-blue-400"
                : "text-white/70 hover:bg-white/5 hover:text-white"
            } ${feature.suppressed ? "opacity-40 line-through" : ""}`}
          >
            <span
              className={`flex h-6 w-6 shrink-0 items-center justify-center rounded text-xs font-bold ${
                selectedId === feature.id
                  ? "bg-blue-600/30 text-blue-300"
                  : "bg-white/10 text-white/50"
              }`}
            >
              {FEATURE_ICONS[feature.type] ?? "?"}
            </span>
            <span className="truncate">{feature.name}</span>
            <span className="ml-auto text-xs text-white/30">{index + 1}</span>
          </button>
        ))}
      </div>
      <div className="border-t border-white/10 px-4 py-2">
        <p className="text-xs text-white/30">
          {features.length} feature{features.length !== 1 ? "s" : ""}
        </p>
      </div>
    </div>
  );
}
