import { usePreferencesStore } from "@/stores/preferences-store";

interface UnitInputProps {
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
  step?: number;
  label?: string;
  disabled?: boolean;
  className?: string;
  testId?: string;
}

export function UnitInput({
  value,
  onChange,
  min,
  max,
  step = 0.1,
  label,
  disabled = false,
  className = "",
  testId,
}: UnitInputProps) {
  const unitSystem = usePreferencesStore((s) => s.unitSystem);

  return (
    <div className={`flex items-center gap-1 ${className}`}>
      {label && (
        <label className="text-[10px] text-[var(--cad-text-muted)] min-w-[60px]">{label}</label>
      )}
      <div className="relative flex-1">
        <input
          type="number"
          value={value}
          onChange={(e) => onChange(parseFloat(e.target.value) || 0)}
          min={min}
          max={max}
          step={step}
          disabled={disabled}
          data-testid={testId}
          className="w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-input)] px-2 py-1 pr-8 text-xs text-[var(--cad-text-primary)] outline-none focus:border-[var(--cad-accent)] disabled:opacity-50"
        />
        <span className="absolute right-2 top-1/2 -translate-y-1/2 text-[10px] text-[var(--cad-text-muted)] pointer-events-none">
          {unitSystem}
        </span>
      </div>
    </div>
  );
}

interface AngleInputProps {
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
  step?: number;
  label?: string;
  disabled?: boolean;
  className?: string;
}

export function AngleInput({
  value,
  onChange,
  min,
  max,
  step = 1,
  label,
  disabled = false,
  className = "",
}: AngleInputProps) {
  return (
    <div className={`flex items-center gap-1 ${className}`}>
      {label && (
        <label className="text-[10px] text-[var(--cad-text-muted)] min-w-[60px]">{label}</label>
      )}
      <div className="relative flex-1">
        <input
          type="number"
          value={value}
          onChange={(e) => onChange(parseFloat(e.target.value) || 0)}
          min={min}
          max={max}
          step={step}
          disabled={disabled}
          className="w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-input)] px-2 py-1 pr-6 text-xs text-[var(--cad-text-primary)] outline-none focus:border-[var(--cad-accent)] disabled:opacity-50"
        />
        <span className="absolute right-2 top-1/2 -translate-y-1/2 text-[10px] text-[var(--cad-text-muted)] pointer-events-none">
          °
        </span>
      </div>
    </div>
  );
}
