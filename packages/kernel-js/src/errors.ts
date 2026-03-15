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
   * Parse a JSON error string from the WASM boundary.
   */
  static fromWasm(json: string): KernelError {
    const parsed = JSON.parse(json) as { kind: KernelErrorKind; message: string };
    return new KernelError(parsed.kind, parsed.message);
  }
}
