import { useRef, useState } from "react";
import { useEditorStore } from "@/stores/editor-store";

const FACES: { label: string; target: [number, number, number]; style: React.CSSProperties }[] = [
  { label: "Front", target: [0, 0, 30], style: { transform: "rotateY(0deg) translateZ(30px)" } },
  { label: "Back", target: [0, 0, -30], style: { transform: "rotateY(180deg) translateZ(30px)" } },
  { label: "Right", target: [30, 0, 0], style: { transform: "rotateY(90deg) translateZ(30px)" } },
  { label: "Left", target: [-30, 0, 0], style: { transform: "rotateY(-90deg) translateZ(30px)" } },
  { label: "Top", target: [0, 30, 0], style: { transform: "rotateX(90deg) translateZ(30px)" } },
  { label: "Bottom", target: [0, -30, 0], style: { transform: "rotateX(-90deg) translateZ(30px)" } },
];

export function ViewCube() {
  const setCameraTarget = useEditorStore((s) => s.setCameraTarget);
  const [hovered, setHovered] = useState<string | null>(null);
  const cubeRef = useRef<HTMLDivElement>(null);

  return (
    <div className="absolute top-3 right-3 z-10 select-none" style={{ perspective: "300px" }}>
      <div
        ref={cubeRef}
        className="relative"
        style={{
          width: 60,
          height: 60,
          transformStyle: "preserve-3d",
          transform: "rotateX(-25deg) rotateY(-35deg)",
        }}
      >
        {FACES.map((face) => (
          <button
            key={face.label}
            onClick={() => setCameraTarget(face.target)}
            onMouseEnter={() => setHovered(face.label)}
            onMouseLeave={() => setHovered(null)}
            className={`absolute flex items-center justify-center border transition-colors ${
              hovered === face.label
                ? "bg-[var(--cad-accent)]/40 border-[var(--cad-accent)] text-white"
                : "bg-[#2a2a2e]/80 border-[#555] text-white/70 hover:bg-[#3a3a3e]"
            }`}
            style={{
              ...face.style,
              width: 60,
              height: 60,
              marginLeft: -30,
              marginTop: -30,
              fontSize: 9,
              fontWeight: 600,
              backfaceVisibility: "hidden",
            }}
          >
            {face.label}
          </button>
        ))}
      </div>
    </div>
  );
}
