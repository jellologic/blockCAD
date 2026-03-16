"""Cross-validate loft (tapered prism) geometry against trimesh.

Geometry: loft between a 4x4 square at z=0 and a 2x2 square at z=10.
This produces a frustum with square cross-sections.
Expected volume (frustum formula): h/3 * (A1 + A2 + sqrt(A1*A2))
  = 10/3 * (16 + 4 + sqrt(64)) = 10/3 * 28 = 93.33
"""

import math


def test_loft_is_watertight(loft):
    mesh, _ = loft
    assert mesh.is_watertight, "Loft mesh should be watertight (closed solid)"


def test_loft_volume(loft):
    mesh, _ = loft
    expected = 10.0 / 3.0 * (16.0 + 4.0 + math.sqrt(16.0 * 4.0))  # ~93.33
    assert abs(mesh.volume - expected) < 5.0, (
        f"Loft volume should be ~{expected:.1f}, got {mesh.volume:.1f}"
    )


def test_loft_bounding_box(loft):
    mesh, _ = loft
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Bottom profile: 4x4 centered at origin => x,y in [-2, 2]
    # Top profile: 2x2 centered at origin => x,y in [-1, 1]
    # Overall bbox: [-2, -2, 0] to [2, 2, 10]
    assert abs(bbox_min[0] - (-2.0)) < 0.5, f"Bbox min x should be ~-2, got {bbox_min[0]}"
    assert abs(bbox_min[1] - (-2.0)) < 0.5, f"Bbox min y should be ~-2, got {bbox_min[1]}"
    assert abs(bbox_min[2] - 0.0) < 0.5, f"Bbox min z should be ~0, got {bbox_min[2]}"
    assert abs(bbox_max[0] - 2.0) < 0.5, f"Bbox max x should be ~2, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 2.0) < 0.5, f"Bbox max y should be ~2, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 10.0) < 0.5, f"Bbox max z should be ~10, got {bbox_max[2]}"


def test_loft_matches_kernel_volume(loft):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = loft
    assert abs(mesh.volume - props["volume"]) < 2.0, (
        f"trimesh volume ({mesh.volume:.2f}) should match kernel ({props['volume']:.2f})"
    )
