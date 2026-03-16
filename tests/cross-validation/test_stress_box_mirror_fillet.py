"""Cross-validate stress test: 10x5x7 box mirrored across YZ plane, then filleted.

Geometry: 10x5x7 box extruded from sketch, mirrored across YZ plane at x=0,
then fillet applied (edge 0, r=0.5).
Combined solid spans [-10,10] x [0,5] x [0,7] before fillet.
Volume: ~700 minus a small fillet removal.
"""

import pytest


@pytest.mark.xfail(reason="Mirror+fillet produces non-watertight mesh (kernel limitation)")
def test_stress_box_mirror_fillet_is_watertight(stress_box_mirror_fillet):
    mesh, _ = stress_box_mirror_fillet
    assert mesh.is_watertight, "Mirror+fillet mesh should be watertight (closed solid)"


def test_stress_box_mirror_fillet_volume(stress_box_mirror_fillet):
    mesh, _ = stress_box_mirror_fillet
    # Mirrored box = 700, fillet removes a small amount with r=0.5
    assert 650.0 < mesh.volume < 700.0, (
        f"Mirror+fillet volume should be ~700 minus fillet, got {mesh.volume}"
    )


def test_stress_box_mirror_fillet_bounding_box(stress_box_mirror_fillet):
    mesh, _ = stress_box_mirror_fillet
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # X spans [-10, 10] (mirrored)
    assert abs(bbox_min[0] - (-10.0)) < 0.5, f"Bbox min x should be ~-10, got {bbox_min[0]}"
    assert abs(bbox_max[0] - 10.0) < 0.5, f"Bbox max x should be ~10, got {bbox_max[0]}"
    # Y spans [0, 5]
    assert abs(bbox_min[1] - 0.0) < 0.5, f"Bbox min y should be ~0, got {bbox_min[1]}"
    assert abs(bbox_max[1] - 5.0) < 0.5, f"Bbox max y should be ~5, got {bbox_max[1]}"
    # Z spans [0, 7]
    assert abs(bbox_min[2] - 0.0) < 0.5, f"Bbox min z should be ~0, got {bbox_min[2]}"
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_stress_box_mirror_fillet_matches_kernel_volume(stress_box_mirror_fillet):
    """trimesh volume should match blockCAD kernel divergence-theorem volume."""
    mesh, props = stress_box_mirror_fillet
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
