/**
 * Dual-target WASM initializer.
 * Browser: fetch-based async init.
 * Node/Bun/jsdom: reads .wasm from disk and uses initSync.
 */

let initialized = false;

/** Detect real browser vs jsdom/Node/Bun */
function isRealBrowser(): boolean {
  return (
    typeof (globalThis as any).window !== "undefined" &&
    typeof (globalThis as any).navigator !== "undefined" &&
    !((globalThis as any).navigator?.userAgent?.includes?.("jsdom"))
  );
}

export async function initKernel(): Promise<void> {
  if (initialized) return;

  if (isRealBrowser()) {
    // Browser: use fetch-based init from wasm-pack --target web
    const { default: init } = await import("@blockCAD/kernel-wasm");
    await init();
  } else {
    // Node/Bun/jsdom: read .wasm from disk, call initSync
    const { initSync } = await import("@blockCAD/kernel-wasm");
    const { readFileSync } = await import("node:fs");
    const { dirname, join } = await import("node:path");

    // Resolve WASM file path relative to this source file.
    // kernel-js/src/init.ts → ../../kernel/pkg/blockcad_kernel_bg.wasm
    const thisDir = dirname(new URL(import.meta.url).pathname);
    const wasmPath = join(thisDir, "..", "..", "kernel", "pkg", "blockcad_kernel_bg.wasm");
    const bytes = readFileSync(wasmPath);
    initSync({ module: bytes });
  }

  initialized = true;
}

/** Returns true if the WASM kernel has been initialized successfully */
export function isKernelInitialized(): boolean {
  return initialized;
}
