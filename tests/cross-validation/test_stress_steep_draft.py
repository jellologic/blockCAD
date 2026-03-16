"""Cross-validate stress test: Extrude 10x5x7 box -> Draft(2 side faces, 15 deg).

A steep 15-degree draft angle on two side faces significantly alters the
box geometry, tilting those faces outward. The volume increases from 350
to approximately 445.
"""


def test_watertight(stress_steep_draft):
    mesh, _ = stress_steep_draft
    assert mesh.is_watertight, "Steep draft mesh should be watertight"


def test_volume_bounds(stress_steep_draft):
    mesh, _ = stress_steep_draft
    # 15-degree draft on 2 side faces tilts them outward, increasing volume to ~445.
    assert mesh.volume > 350.0, f"Volume should be > 350 (original box), got {mesh.volume}"
    assert mesh.volume < 600.0, f"Volume should be < 600, got {mesh.volume}"


def test_bounding_box(stress_steep_draft):
    mesh, _ = stress_steep_draft
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Draft tilts sides outward; top is wider than base. Bounding box grows.
    assert bbox_max[0] - bbox_min[0] > 8.0, f"X extent should be > 8, got {bbox_max[0] - bbox_min[0]}"
    assert bbox_max[0] - bbox_min[0] < 18.0, f"X extent should be < 18, got {bbox_max[0] - bbox_min[0]}"
    assert bbox_max[1] - bbox_min[1] > 3.0, f"Y extent should be > 3, got {bbox_max[1] - bbox_min[1]}"
    assert bbox_max[1] - bbox_min[1] < 12.0, f"Y extent should be < 12, got {bbox_max[1] - bbox_min[1]}"
    assert bbox_max[2] - bbox_min[2] > 5.0, f"Z extent should be > 5, got {bbox_max[2] - bbox_min[2]}"
    assert bbox_max[2] - bbox_min[2] < 9.0, f"Z extent should be < 9, got {bbox_max[2] - bbox_min[2]}"


def test_kernel_volume_match(stress_steep_draft):
    """trimesh volume should match blockCAD kernel's divergence-theorem volume."""
    mesh, props = stress_steep_draft
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume:.2f}) should match kernel ({props['volume']:.2f})"
    )
