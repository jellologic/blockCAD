"""Cross-validate stress test: Extrude 10x5x7 box -> Fillet(edge 0, r=0.5)
-> Chamfer(edge 4, d=0.3) -> Shell(top removed, t=0.4) -> Draft(2 side faces, 3 deg).

Five finishing operations applied in sequence. The shell hollows the box
(outer ~10x5x7, wall thickness 0.4, open top), producing a volume well below
the original 350. The fillet, chamfer, and draft further modify the geometry
but keep the bounding box close to 10x5x7.
"""


def test_watertight(stress_box_5ops):
    mesh, _ = stress_box_5ops
    assert mesh.is_watertight, "5-op stress mesh should be watertight"


def test_volume_bounds(stress_box_5ops):
    mesh, _ = stress_box_5ops
    # Original box 350; shell hollows significantly; fillet/chamfer/draft tweak further.
    assert mesh.volume > 50.0, f"Volume should be > 50, got {mesh.volume}"
    assert mesh.volume < 300.0, f"Volume should be < 300, got {mesh.volume}"


def test_bounding_box(stress_box_5ops):
    mesh, _ = stress_box_5ops
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Draft may widen/shift the bbox slightly, but it should stay close to 10x5x7.
    assert bbox_max[0] - bbox_min[0] > 8.0, f"X extent should be > 8, got {bbox_max[0] - bbox_min[0]}"
    assert bbox_max[0] - bbox_min[0] < 14.0, f"X extent should be < 14, got {bbox_max[0] - bbox_min[0]}"
    assert bbox_max[1] - bbox_min[1] > 3.0, f"Y extent should be > 3, got {bbox_max[1] - bbox_min[1]}"
    assert bbox_max[1] - bbox_min[1] < 9.0, f"Y extent should be < 9, got {bbox_max[1] - bbox_min[1]}"
    assert bbox_max[2] - bbox_min[2] > 5.0, f"Z extent should be > 5, got {bbox_max[2] - bbox_min[2]}"
    assert bbox_max[2] - bbox_min[2] < 9.0, f"Z extent should be < 9, got {bbox_max[2] - bbox_min[2]}"


def test_kernel_volume_match(stress_box_5ops):
    """trimesh volume should match blockCAD kernel's divergence-theorem volume."""
    mesh, props = stress_box_5ops
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume:.2f}) should match kernel ({props['volume']:.2f})"
    )
