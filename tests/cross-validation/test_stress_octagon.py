"""Stress test: Regular octagon (8 vertices, circumradius=5) extruded 5mm.

Tests the kernel's ability to handle an 8-sided polygon sketch with
vertices at 45-degree intervals (r=5), connected by 8 line segments,
extruded along Z.

Regular octagon area = 2 * sqrt(2) * r^2 = 2 * 1.41421356 * 25 = 70.71
Volume = area * height = 70.71 * 5 = 353.6
Bounding box: x in [-5, 5], y in [-5, 5], z in [0, 5].
"""

import math


def test_watertight(stress_octagon):
    mesh, _ = stress_octagon
    assert mesh.is_watertight, "Octagon extrude mesh should be watertight"


def test_volume(stress_octagon):
    mesh, _ = stress_octagon
    expected = 2.0 * math.sqrt(2) * 25.0 * 5.0  # ~353.6
    assert abs(mesh.volume - expected) < 10.0, (
        f"Octagon volume ({mesh.volume:.1f}) should be ~{expected:.1f}"
    )


def test_bounding_box(stress_octagon):
    mesh, _ = stress_octagon
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Octagon circumradius = 5, centered at origin, extruded 5mm along Z
    assert abs(bbox_min[0] - (-5.0)) < 0.5, f"Bbox min x should be ~-5, got {bbox_min[0]}"
    assert abs(bbox_max[0] - 5.0) < 0.5, f"Bbox max x should be ~5, got {bbox_max[0]}"
    assert abs(bbox_min[1] - (-5.0)) < 0.5, f"Bbox min y should be ~-5, got {bbox_min[1]}"
    assert abs(bbox_max[1] - 5.0) < 0.5, f"Bbox max y should be ~5, got {bbox_max[1]}"
    assert abs(bbox_min[2]) < 0.5, f"Bbox min z should be ~0, got {bbox_min[2]}"
    assert abs(bbox_max[2] - 5.0) < 0.5, f"Bbox max z should be ~5, got {bbox_max[2]}"


def test_kernel_volume_match(stress_octagon):
    """trimesh volume should match blockCAD kernel divergence-theorem volume."""
    mesh, props = stress_octagon
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume:.1f}) should match kernel ({props['volume']:.1f})"
    )
