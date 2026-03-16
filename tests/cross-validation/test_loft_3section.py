"""Cross-validate loft with 3 profiles against trimesh.

Geometry: loft between 3 square profiles:
  - 4x4 at z=0
  - 3x3 at z=5
  - 2x2 at z=10

This produces a tapered solid with square cross-sections.
Expected volume (sum of two frustums):
  Lower frustum (z=0..5): h/3 * (A1 + A2 + sqrt(A1*A2)) = 5/3 * (16 + 9 + 12) = 61.67
  Upper frustum (z=5..10): 5/3 * (9 + 4 + 6) = 31.67
  Total ≈ 93.33
"""

import math


def test_loft_3section_is_watertight(loft_3section):
    mesh, _ = loft_3section
    assert mesh.is_watertight, "Loft 3-section mesh should be watertight (closed solid)"


def test_loft_3section_volume(loft_3section):
    mesh, _ = loft_3section
    # Sum of two frustums
    lower = 5.0 / 3.0 * (16.0 + 9.0 + math.sqrt(16.0 * 9.0))  # 61.67
    upper = 5.0 / 3.0 * (9.0 + 4.0 + math.sqrt(9.0 * 4.0))    # 31.67
    expected = lower + upper  # ~93.33
    assert mesh.volume > 80.0, (
        f"Loft 3-section volume should be > 80, got {mesh.volume:.1f}"
    )
    assert mesh.volume < 110.0, (
        f"Loft 3-section volume should be < 110, got {mesh.volume:.1f}"
    )


def test_loft_3section_bounding_box(loft_3section):
    mesh, _ = loft_3section
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Bottom profile: 4x4 centered at origin => x,y in [-2, 2]
    # Middle profile: 3x3 centered at origin => x,y in [-1.5, 1.5]
    # Top profile: 2x2 centered at origin => x,y in [-1, 1]
    # Overall bbox: [-2, -2, 0] to [2, 2, 10]
    assert abs(bbox_min[0] - (-2.0)) < 0.5, f"Bbox min x should be ~-2, got {bbox_min[0]}"
    assert abs(bbox_min[1] - (-2.0)) < 0.5, f"Bbox min y should be ~-2, got {bbox_min[1]}"
    assert abs(bbox_min[2] - 0.0) < 0.5, f"Bbox min z should be ~0, got {bbox_min[2]}"
    assert abs(bbox_max[0] - 2.0) < 0.5, f"Bbox max x should be ~2, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 2.0) < 0.5, f"Bbox max y should be ~2, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 10.0) < 0.5, f"Bbox max z should be ~10, got {bbox_max[2]}"


def test_loft_3section_matches_kernel_volume(loft_3section):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = loft_3section
    assert abs(mesh.volume - props["volume"]) < 2.0, (
        f"trimesh volume ({mesh.volume:.2f}) should match kernel ({props['volume']:.2f})"
    )
