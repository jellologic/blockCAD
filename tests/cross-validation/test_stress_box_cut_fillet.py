"""Stress test: Extrude 10x5x7 box -> CutExtrude(4x2x3 blind pocket) -> Fillet(edge 0, r=0.3).

The fillet is applied to an edge of the outer box, NOT a pocket edge.
Box volume = 350, pocket removes 4*2*3 = 24, fillet removes a small amount.
Expected volume < 326. Bounding box should still be ~10x5x7.
"""


def test_stress_box_cut_fillet_is_watertight(stress_box_cut_fillet):
    mesh, _ = stress_box_cut_fillet
    assert mesh.is_watertight, "Stress box-cut-fillet mesh should be watertight"


def test_stress_box_cut_fillet_volume_less_than_326(stress_box_cut_fillet):
    mesh, _ = stress_box_cut_fillet
    assert mesh.volume < 326.0, (
        f"Volume ({mesh.volume}) should be < 326 (box minus pocket)"
    )


def test_stress_box_cut_fillet_bounding_box(stress_box_cut_fillet):
    mesh, _ = stress_box_cut_fillet
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Outer bounding box should still be ~10x5x7
    assert all(abs(v) < 0.5 for v in bbox_min), f"Bbox min should be near origin, got {bbox_min}"
    assert abs(bbox_max[0] - 10.0) < 0.5, f"Bbox max x should be ~10, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 5.0) < 0.5, f"Bbox max y should be ~5, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_stress_box_cut_fillet_matches_kernel_volume(stress_box_cut_fillet):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = stress_box_cut_fillet
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
