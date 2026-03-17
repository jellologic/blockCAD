/**
 * Main-thread proxy that communicates with the WASM kernel Web Worker.
 * All kernel operations are async and non-blocking.
 */
import { parseMeshBytes } from "@blockCAD/kernel";
import type { MeshData, FeatureEntry, FeatureParams, MassProperties } from "@blockCAD/kernel";

interface PendingRequest {
  resolve: (value: WorkerResult) => void;
  reject: (reason: Error) => void;
}

interface WorkerResult {
  meshData?: MeshData;
  features?: FeatureEntry[];
  exportBuffer?: ArrayBuffer;
  data?: unknown;
}

export class KernelProxy {
  private worker: Worker;
  private pending = new Map<number, PendingRequest>();
  private nextId = 0;
  private _ready = false;

  constructor() {
    this.worker = new Worker(
      new URL("../workers/kernel-worker.ts", import.meta.url),
      { type: "module" },
    );
    this.worker.onmessage = (e) => this.handleMessage(e);
    this.worker.onerror = (e) => {
      console.error("[KernelProxy] Worker error:", e);
    };
  }

  get ready(): boolean {
    return this._ready;
  }

  private handleMessage(e: MessageEvent): void {
    const { id, success, error, meshBuffer, exportBuffer, data, features } = e.data;
    const pending = this.pending.get(id);
    if (!pending) return;
    this.pending.delete(id);

    if (!success) {
      pending.reject(new Error(error));
      return;
    }

    // Parse mesh data on main thread if buffer was transferred
    let meshData: MeshData | undefined;
    if (meshBuffer) {
      meshData = parseMeshBytes(meshBuffer);
    }

    pending.resolve({ meshData, features, exportBuffer, data });
  }

  private send(type: string, payload?: Record<string, unknown>): Promise<WorkerResult> {
    const id = this.nextId++;
    return new Promise<WorkerResult>((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.worker.postMessage({ type, id, ...payload });
    });
  }

  async init(): Promise<void> {
    await this.send("init");
    this._ready = true;
  }

  async addFeature(
    kind: string,
    name: string,
    params: FeatureParams,
  ): Promise<{ meshData?: MeshData; features?: FeatureEntry[] }> {
    const result = await this.send("addFeature", { kind, name, params });
    return { meshData: result.meshData, features: result.features };
  }

  async tessellate(
    chordTolerance?: number,
    angleTolerance?: number,
  ): Promise<{ meshData: MeshData; features: FeatureEntry[] }> {
    const result = await this.send("tessellate", { chordTolerance, angleTolerance });
    return { meshData: result.meshData!, features: result.features! };
  }

  async suppress(index: number): Promise<{ meshData?: MeshData; features?: FeatureEntry[] }> {
    const result = await this.send("suppress", { index });
    return { meshData: result.meshData, features: result.features };
  }

  async unsuppress(index: number): Promise<{ meshData?: MeshData; features?: FeatureEntry[] }> {
    const result = await this.send("unsuppress", { index });
    return { meshData: result.meshData, features: result.features };
  }

  async featureList(): Promise<FeatureEntry[]> {
    const result = await this.send("featureList");
    return result.data as FeatureEntry[];
  }

  async serialize(): Promise<string> {
    const result = await this.send("serialize");
    return result.data as string;
  }

  async deserialize(json: string): Promise<void> {
    await this.send("deserialize", { json });
  }

  async replayAndAdd(
    features: Array<{ type: string; name: string; params: FeatureParams }>,
    newKind: string,
    newName: string,
    newParams: FeatureParams,
  ): Promise<{ meshData: MeshData; features: FeatureEntry[] }> {
    const result = await this.send("replayAndAdd", {
      features,
      newKind,
      newName,
      newParams,
    });
    return { meshData: result.meshData!, features: result.features! };
  }

  async exportSTL(
    binary: boolean,
    chordTolerance?: number,
    angleTolerance?: number,
    options?: Record<string, unknown>,
  ): Promise<ArrayBuffer | string> {
    if (binary) {
      const result = await this.send("exportSTL", { chordTolerance, angleTolerance });
      return result.exportBuffer!;
    }
    const result = await this.send("exportSTLAscii", { options, chordTolerance, angleTolerance });
    return result.data as string;
  }

  async exportOBJ(
    options?: Record<string, unknown>,
    chordTolerance?: number,
    angleTolerance?: number,
  ): Promise<string> {
    const result = await this.send("exportOBJ", { options, chordTolerance, angleTolerance });
    return result.data as string;
  }

  async export3MF(
    options?: Record<string, unknown>,
    chordTolerance?: number,
    angleTolerance?: number,
  ): Promise<ArrayBuffer> {
    const result = await this.send("export3MF", { options, chordTolerance, angleTolerance });
    return result.exportBuffer!;
  }

  async exportGLB(
    options?: Record<string, unknown>,
    chordTolerance?: number,
    angleTolerance?: number,
  ): Promise<ArrayBuffer> {
    const result = await this.send("exportGLB", { options, chordTolerance, angleTolerance });
    return result.exportBuffer!;
  }

  async exportSTEP(
    options?: Record<string, unknown>,
    chordTolerance?: number,
    angleTolerance?: number,
  ): Promise<string> {
    const result = await this.send("exportSTEP", { options, chordTolerance, angleTolerance });
    return result.data as string;
  }

  async computeMassProperties(
    density?: number,
    chordTolerance?: number,
    angleTolerance?: number,
  ): Promise<MassProperties> {
    const result = await this.send("computeMassProperties", { density, chordTolerance, angleTolerance });
    return result.data as MassProperties;
  }

  terminate(): void {
    this.worker.terminate();
    this.pending.clear();
  }
}
