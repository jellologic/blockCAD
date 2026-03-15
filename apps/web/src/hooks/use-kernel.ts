import { useState, useEffect } from "react";
import type { MeshData, FeatureEntry } from "@blockCAD/kernel";
import { initMockKernel, type MockKernelClient } from "@blockCAD/kernel";

interface UseKernelResult {
  kernel: MockKernelClient | null;
  meshData: MeshData | null;
  features: FeatureEntry[];
  isLoading: boolean;
  error: Error | null;
}

export function useKernel(): UseKernelResult {
  const [kernel, setKernel] = useState<MockKernelClient | null>(null);
  const [meshData, setMeshData] = useState<MeshData | null>(null);
  const [features, setFeatures] = useState<FeatureEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function init() {
      try {
        const client = await initMockKernel();
        if (cancelled) return;

        const mesh = client.tessellate();
        setKernel(client);
        setMeshData(mesh);
        setFeatures(client.featureList);
        setIsLoading(false);
      } catch (err) {
        if (cancelled) return;
        setError(err instanceof Error ? err : new Error(String(err)));
        setIsLoading(false);
      }
    }

    init();
    return () => {
      cancelled = true;
    };
  }, []);

  return { kernel, meshData, features, isLoading, error };
}
