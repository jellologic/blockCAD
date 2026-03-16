export type KernelErrorKind =
  | "geometry"
  | "topology"
  | "constraint_solver"
  | "operation"
  | "serialization"
  | "migration"
  | "invalid_parameter"
  | "not_found"
  | "over_constrained"
  | "under_constrained"
  | "internal";

/**
 * Typed error from the WASM kernel.
 * Constructed by parsing the JSON error string from the WASM boundary.
 */
export class KernelError extends Error {
  public readonly kind: KernelErrorKind;

  constructor(kind: KernelErrorKind, message: string) {
    super(message);
    this.name = "KernelError";
    this.kind = kind;
  }

  /**
   * Parse an error from the WASM boundary.
   * Handles both JSON error objects and plain text error strings.
   */
  static fromWasm(errorStr: string): KernelError {
    try {
      const parsed = JSON.parse(errorStr) as { kind: KernelErrorKind; message: string };
      return new KernelError(parsed.kind, parsed.message);
    } catch {
      // Not JSON — treat as plain error message
      return new KernelError("internal", errorStr);
    }
  }
}
