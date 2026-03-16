import { describe, it, expect, beforeEach } from "vitest";
import { useAssemblyStore } from "@/stores/assembly-store";

describe("assembly store", () => {
  beforeEach(() => {
    // Reset store to initial state
    useAssemblyStore.setState({
      assembly: null,
      meshData: null,
      parts: [],
      components: [],
      mates: [],
      selectedComponentId: null,
      isAssemblyMode: false,
      isExploded: false,
      bomData: null,
      activeOp: null,
      isLoading: false,
    });
  });

  it("initial state is not in assembly mode", () => {
    expect(useAssemblyStore.getState().isAssemblyMode).toBe(false);
    expect(useAssemblyStore.getState().assembly).toBeNull();
  });

  it("initAssembly sets isAssemblyMode true", async () => {
    await useAssemblyStore.getState().initAssembly();
    expect(useAssemblyStore.getState().isAssemblyMode).toBe(true);
    expect(useAssemblyStore.getState().assembly).not.toBeNull();
  });

  it("exitAssemblyMode clears all state", async () => {
    await useAssemblyStore.getState().initAssembly();
    expect(useAssemblyStore.getState().isAssemblyMode).toBe(true);

    useAssemblyStore.getState().exitAssemblyMode();
    expect(useAssemblyStore.getState().isAssemblyMode).toBe(false);
    expect(useAssemblyStore.getState().assembly).toBeNull();
    expect(useAssemblyStore.getState().parts).toEqual([]);
    expect(useAssemblyStore.getState().components).toEqual([]);
    expect(useAssemblyStore.getState().mates).toEqual([]);
    expect(useAssemblyStore.getState().bomData).toBeNull();
  });

  it("addPart returns part ID and updates parts list", async () => {
    await useAssemblyStore.getState().initAssembly();
    const partId = useAssemblyStore.getState().addPart("Box Part");
    expect(partId).not.toBeNull();
    expect(useAssemblyStore.getState().parts).toHaveLength(1);
    expect(useAssemblyStore.getState().parts[0].name).toBe("Box Part");
  });

  it("addPart returns null without assembly", () => {
    const partId = useAssemblyStore.getState().addPart("Test");
    expect(partId).toBeNull();
  });

  it("toggleExploded flips state", async () => {
    await useAssemblyStore.getState().initAssembly();
    expect(useAssemblyStore.getState().isExploded).toBe(false);

    useAssemblyStore.getState().toggleExploded();
    expect(useAssemblyStore.getState().isExploded).toBe(true);

    useAssemblyStore.getState().toggleExploded();
    expect(useAssemblyStore.getState().isExploded).toBe(false);
  });

  it("hideBom clears bomData", () => {
    useAssemblyStore.setState({ bomData: [{ part_id: "p1", part_name: "Part", quantity: 1 }] as any });
    useAssemblyStore.getState().hideBom();
    expect(useAssemblyStore.getState().bomData).toBeNull();
  });

  it("startOp sets activeOp", () => {
    const op = { type: "insert-component" as const, partId: "p1", name: "Box", x: 0, y: 0, z: 0 };
    useAssemblyStore.getState().startOp(op);
    expect(useAssemblyStore.getState().activeOp).toEqual(op);
  });

  it("cancelOp clears activeOp", () => {
    useAssemblyStore.getState().startOp({ type: "insert-component", partId: "p1", name: "Box", x: 0, y: 0, z: 0 });
    useAssemblyStore.getState().cancelOp();
    expect(useAssemblyStore.getState().activeOp).toBeNull();
  });

  it("selectComponent sets selectedComponentId", async () => {
    await useAssemblyStore.getState().initAssembly();
    useAssemblyStore.getState().selectComponent("comp-1");
    expect(useAssemblyStore.getState().selectedComponentId).toBe("comp-1");

    useAssemblyStore.getState().selectComponent(null);
    expect(useAssemblyStore.getState().selectedComponentId).toBeNull();
  });
});
