"""Stress test: Extrude 10x5x7 box -> Fillet(edge 0, r=0.5) -> CutExtrude(4x2x3 blind pocket) -> Shell(top removed, t=0.5).

Complete part workflow with 4 operations.
Box volume = 350, fillet removes a small amount, pocket removes 4*2*3 = 24, shell hollows out interior.
Expected volume ~140 (kernel: 140.4). Bounding box should still be ~10x5x7.
"""

import pytest


@pytest.mark.xfail(reason="Known limitation: CutExtrude + Shell combo produces non-watertight mesh")
def test_stress_full_part_1_is_watertight(stress_full_part_1):
    mesh, _ = stress_full_part_1
    assert mesh.is_watertight, "Stress full_part_1 mesh should be watertight"


def test_stress_full_part_1_volume_bounds(stress_full_part_1):
    mesh, _ = stress_full_part_1
    assert mesh.volume > 80.0, (
        f"Volume ({mesh.volume}) should be > 80 (shelled part with walls)"
    )
    assert mesh.volume < 250.0, (
        f"Volume ({mesh.volume}) should be < 250 (shell hollows out most of the interior)"
    )


def test_stress_full_part_1_bounding_box(stress_full_part_1):
    mesh, _ = stress_full_part_1
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Outer bounding box should still be ~10x5x7
    assert all(abs(v) < 0.5 for v in bbox_min), f"Bbox min should be near origin, got {bbox_min}"
    assert abs(bbox_max[0] - 10.0) < 0.5, f"Bbox max x should be ~10, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 5.0) < 0.5, f"Bbox max y should be ~5, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_stress_full_part_1_matches_kernel_volume(stress_full_part_1):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = stress_full_part_1
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
