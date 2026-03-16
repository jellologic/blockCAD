"""Stress test: Extrude 10x5x7 box -> Fillet(edge 0, r=2.0).

Large radius (2.0) approaching the 5mm edge of the box.
Box volume = 350, fillet removes noticeable material.
Expected volume noticeably less than 350. Bounding box should still be ~10x5x7.
"""


def test_stress_large_fillet_is_watertight(stress_large_fillet):
    mesh, _ = stress_large_fillet
    assert mesh.is_watertight, "Stress large-fillet mesh should be watertight"


def test_stress_large_fillet_volume_less_than_350(stress_large_fillet):
    mesh, _ = stress_large_fillet
    assert mesh.volume < 350.0, (
        f"Volume ({mesh.volume}) should be noticeably less than 350 (box with large fillet)"
    )


def test_stress_large_fillet_bounding_box(stress_large_fillet):
    mesh, _ = stress_large_fillet
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Outer bounding box should still be ~10x5x7
    assert all(abs(v) < 0.5 for v in bbox_min), f"Bbox min should be near origin, got {bbox_min}"
    assert abs(bbox_max[0] - 10.0) < 0.5, f"Bbox max x should be ~10, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 5.0) < 0.5, f"Bbox max y should be ~5, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_stress_large_fillet_matches_kernel_volume(stress_large_fillet):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = stress_large_fillet
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
