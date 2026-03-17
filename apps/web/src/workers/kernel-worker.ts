/// <reference lib="webworker" />
declare const self: DedicatedWorkerGlobalScope;

/**
 * Web Worker that owns the WASM kernel instance.
 * All heavy compute (evaluate, tessellate, export) happens here,
 * keeping the main thread free for rendering.
 *
 * Protocol: main thread sends { type, id, ...data }, worker replies
 * { type: 'result', id, success, ...payload }. Buffers are transferred
 * (zero-copy) via the Transferable list.
 */

import { KernelClient } from "@blockCAD/kernel";
import type { FeatureParams } from "@blockCAD/kernel";

let kernel: KernelClient | null = null;

/** Copy a Uint8Array into a fresh ArrayBuffer suitable for transfer. */
function toTransferable(bytes: Uint8Array): ArrayBuffer {
  const buf = new ArrayBuffer(bytes.byteLength);
  new Uint8Array(buf).set(bytes);
  return buf;
}

self.onmessage = async (e: MessageEvent) => {
  const { type, id, ...data } = e.data;

  try {
    switch (type) {
      case "init": {
        // Dynamic import triggers fetch-based WASM init inside the worker
        const { initKernel } = await import("@blockCAD/kernel");
        await initKernel();
        kernel = new KernelClient();
        self.postMessage({ type: "result", id, success: true });
        break;
      }

      case "addFeature": {
        if (!kernel) throw new Error("Kernel not initialized");
        kernel.addFeature(data.kind, data.name, data.params as FeatureParams);
        // Auto-tessellate after adding feature
        try {
          const bytes = kernel.tessellateRaw(data.chordTolerance ?? 0.01, data.angleTolerance ?? 0.5);
          const buffer = toTransferable(bytes);
          self.postMessage(
            { type: "result", id, success: true, meshBuffer: buffer, features: kernel.featureList },
            [buffer],
          );
        } catch {
          // Tessellation can fail (e.g. sketch-only), still report success for addFeature
          self.postMessage({ type: "result", id, success: true, features: kernel.featureList });
        }
        break;
      }

      case "tessellate": {
        if (!kernel) throw new Error("Kernel not initialized");
        const bytes = kernel.tessellateRaw(data.chordTolerance ?? 0.01, data.angleTolerance ?? 0.5);
        const buffer = toTransferable(bytes);
        self.postMessage(
          { type: "result", id, success: true, meshBuffer: buffer, features: kernel.featureList },
          [buffer],
        );
        break;
      }

      case "serialize": {
        if (!kernel) throw new Error("Kernel not initialized");
        const json = kernel.serialize();
        self.postMessage({ type: "result", id, success: true, data: json });
        break;
      }

      case "deserialize": {
        if (!kernel) throw new Error("Kernel not initialized");
        kernel = KernelClient.deserialize(data.json);
        self.postMessage({ type: "result", id, success: true });
        break;
      }

      case "featureList": {
        if (!kernel) throw new Error("Kernel not initialized");
        const list = kernel.featureList;
        self.postMessage({ type: "result", id, success: true, data: list });
        break;
      }

      case "suppress": {
        if (!kernel) throw new Error("Kernel not initialized");
        kernel.suppressFeature(data.index);
        try {
          const bytes = kernel.tessellateRaw(0.01, 0.5);
          const buffer = toTransferable(bytes);
          self.postMessage(
            { type: "result", id, success: true, meshBuffer: buffer, features: kernel.featureList },
            [buffer],
          );
        } catch {
          self.postMessage({ type: "result", id, success: true, features: kernel.featureList });
        }
        break;
      }

      case "unsuppress": {
        if (!kernel) throw new Error("Kernel not initialized");
        kernel.unsuppressFeature(data.index);
        try {
          const bytes = kernel.tessellateRaw(0.01, 0.5);
          const buffer = toTransferable(bytes);
          self.postMessage(
            { type: "result", id, success: true, meshBuffer: buffer, features: kernel.featureList },
            [buffer],
          );
        } catch {
          self.postMessage({ type: "result", id, success: true, features: kernel.featureList });
        }
        break;
      }

      case "exportSTL": {
        if (!kernel) throw new Error("Kernel not initialized");
        const stlBytes = kernel.exportSTLBinary(data.chordTolerance ?? 0.01, data.angleTolerance ?? 0.5);
        const stlBuf = toTransferable(stlBytes);
        self.postMessage({ type: "result", id, success: true, exportBuffer: stlBuf }, [stlBuf]);
        break;
      }

      case "exportSTLAscii": {
        if (!kernel) throw new Error("Kernel not initialized");
        const ascii = kernel.exportSTLAscii(data.options ?? {}, data.chordTolerance ?? 0.01, data.angleTolerance ?? 0.5);
        self.postMessage({ type: "result", id, success: true, data: ascii });
        break;
      }

      case "exportOBJ": {
        if (!kernel) throw new Error("Kernel not initialized");
        const obj = kernel.exportOBJ(data.options ?? {}, data.chordTolerance ?? 0.01, data.angleTolerance ?? 0.5);
        self.postMessage({ type: "result", id, success: true, data: obj });
        break;
      }

      case "export3MF": {
        if (!kernel) throw new Error("Kernel not initialized");
        const mfBytes = kernel.export3MF(data.options ?? {}, data.chordTolerance ?? 0.01, data.angleTolerance ?? 0.5);
        const mfBuf = toTransferable(mfBytes);
        self.postMessage({ type: "result", id, success: true, exportBuffer: mfBuf }, [mfBuf]);
        break;
      }

      case "exportGLB": {
        if (!kernel) throw new Error("Kernel not initialized");
        const glbBytes = kernel.exportGLB(data.options ?? {}, data.chordTolerance ?? 0.01, data.angleTolerance ?? 0.5);
        const glbBuf = toTransferable(glbBytes);
        self.postMessage({ type: "result", id, success: true, exportBuffer: glbBuf }, [glbBuf]);
        break;
      }

      case "exportSTEP": {
        if (!kernel) throw new Error("Kernel not initialized");
        const step = kernel.exportSTEP(data.options ?? {}, data.chordTolerance ?? 0.01, data.angleTolerance ?? 0.5);
        self.postMessage({ type: "result", id, success: true, data: step });
        break;
      }

      case "computeMassProperties": {
        if (!kernel) throw new Error("Kernel not initialized");
        const props = kernel.computeMassProperties(data.density, data.chordTolerance ?? 0.01, data.angleTolerance ?? 0.5);
        self.postMessage({ type: "result", id, success: true, data: props });
        break;
      }

      case "replayAndAdd": {
        // Replay all features on a fresh kernel, then add a new one.
        // Used by confirmOperation to avoid WASM borrow conflicts.
        if (!kernel) throw new Error("Kernel not initialized");
        const fresh = new KernelClient();
        const feats = data.features as Array<{ type: string; name: string; params: FeatureParams }>;
        for (const feat of feats) {
          fresh.addFeature(feat.type, feat.name, feat.params);
        }
        // Add the new operation feature
        fresh.addFeature(data.newKind, data.newName, data.newParams as FeatureParams);
        // Tessellate
        const tBytes = fresh.tessellateRaw(0.01, 0.5);
        const tBuf = toTransferable(tBytes);
        // Replace the global kernel with this fresh one
        kernel = fresh;
        self.postMessage(
          { type: "result", id, success: true, meshBuffer: tBuf, features: fresh.featureList },
          [tBuf],
        );
        break;
      }

      default:
        self.postMessage({ type: "result", id, success: false, error: `Unknown message type: ${type}` });
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err);
    self.postMessage({ type: "result", id, success: false, error: msg });
  }
};
