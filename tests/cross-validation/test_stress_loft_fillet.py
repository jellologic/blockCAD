"""Stress test: Loft(3 sections: 4x4->3x3->2x2 at z=0,5,10) -> Fillet(edge 0, r=0.3).

Geometry: A tapered solid lofted through three square cross-sections, then
one edge is filleted with r=0.3.

Expected volume: close to the 3-section loft frustum sum (~93.3) minus a
small fillet correction.
  Lower frustum: 5/3 * (16 + 9 + sqrt(16*9)) = 61.67
  Upper frustum: 5/3 * (9 + 4 + sqrt(9*4)) = 31.67
  Total loft: ~93.3

Bounding box: approximately [-2, -2, 0] to [2, 2, 10] (centred profiles).
"""

import math


def test_stress_loft_fillet_is_watertight(stress_loft_fillet):
    mesh, _ = stress_loft_fillet
    assert mesh.is_watertight, "Loft+fillet mesh should be watertight (closed solid)"


def test_stress_loft_fillet_volume(stress_loft_fillet):
    mesh, _ = stress_loft_fillet
    lower = 5.0 / 3.0 * (16.0 + 9.0 + math.sqrt(16.0 * 9.0))
    upper = 5.0 / 3.0 * (9.0 + 4.0 + math.sqrt(9.0 * 4.0))
    loft_volume = lower + upper  # ~93.3
    # Fillet removes a small amount; volume should be reasonably close
    assert abs(mesh.volume) > 50.0, (
        f"Loft+fillet volume ({mesh.volume:.1f}) should be > 50"
    )
    assert abs(mesh.volume) < loft_volume + 5.0, (
        f"Loft+fillet volume ({mesh.volume:.1f}) should be < {loft_volume + 5.0:.1f}"
    )


def test_stress_loft_fillet_bounding_box(stress_loft_fillet):
    mesh, _ = stress_loft_fillet
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # X and Y bounded by the largest 4x4 profile: ~[-2, 2]
    assert abs(bbox_min[0] - (-2.0)) < 0.5, f"Bbox min x should be ~-2, got {bbox_min[0]}"
    assert abs(bbox_min[1] - (-2.0)) < 0.5, f"Bbox min y should be ~-2, got {bbox_min[1]}"
    # Z spans 0 to 10
    assert abs(bbox_min[2] - 0.0) < 0.5, f"Bbox min z should be ~0, got {bbox_min[2]}"
    assert abs(bbox_max[0] - 2.0) < 0.5, f"Bbox max x should be ~2, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 2.0) < 0.5, f"Bbox max y should be ~2, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 10.0) < 0.5, f"Bbox max z should be ~10, got {bbox_max[2]}"


def test_stress_loft_fillet_matches_kernel_volume(stress_loft_fillet):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = stress_loft_fillet
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume:.2f}) should match kernel ({props['volume']:.2f})"
    )
