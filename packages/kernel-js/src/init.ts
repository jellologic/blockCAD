/**
 * Dual-target WASM initializer.
 * Handles the browser vs Node/Bun init difference.
 */

let initialized = false;

export async function initKernel(): Promise<void> {
  if (initialized) return;

  if (typeof window !== "undefined") {
    // Browser: use fetch-based init from wasm-pack --target web
    // const { default: init } = await import("@blockCAD/kernel-wasm");
    // await init();
  } else {
    // Bun/Node: use fs-based loading
    // const { default: init } = await import("@blockCAD/kernel-wasm/node");
    // await init();
  }

  initialized = true;
}
