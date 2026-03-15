import { describe, it, expect } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { useKernel } from "../use-kernel";

describe("useKernel", () => {
  it("starts in loading state", () => {
    const { result } = renderHook(() => useKernel());
    expect(result.current.isLoading).toBe(true);
    expect(result.current.meshData).toBeNull();
    expect(result.current.kernel).toBeNull();
  });

  it("loads mesh data after init", async () => {
    const { result } = renderHook(() => useKernel());
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
    expect(result.current.meshData).not.toBeNull();
    expect(result.current.meshData!.vertexCount).toBe(24);
    expect(result.current.meshData!.triangleCount).toBe(12);
  });

  it("loads features after init", async () => {
    const { result } = renderHook(() => useKernel());
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
    expect(result.current.features).toHaveLength(2);
    expect(result.current.features[0].name).toBe("Base Sketch");
  });

  it("has no error on successful init", async () => {
    const { result } = renderHook(() => useKernel());
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
    expect(result.current.error).toBeNull();
  });

  it("provides kernel client instance", async () => {
    const { result } = renderHook(() => useKernel());
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
    expect(result.current.kernel).not.toBeNull();
    expect(result.current.kernel!.featureCount).toBe(2);
  });
});
