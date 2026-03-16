"""Cross-validate partial revolve (180 degrees) against trimesh.

The revolve_half fixture is a 180-degree revolution of a rectangle (x=[5,10], y=[0,3])
around the Y-axis, producing a half-annulus solid.

Analytical properties:
  - Volume: pi * (R_outer^2 - R_inner^2) * height / 2 = pi * 75 * 3 / 2 ~ 353.43
  - Bounding box: [-10, 10] x [0, 3] x [0, 10]  (half-annulus in +Z half-space)
"""

import math


# Analytical expected values
EXPECTED_VOLUME = math.pi * 75.0 * 3.0 / 2.0  # ~353.43


def test_revolve_half_is_watertight(revolve_half):
    mesh, _ = revolve_half
    assert mesh.is_watertight, "Half-revolve mesh should be watertight (closed solid)"


def test_revolve_half_volume(revolve_half):
    mesh, _ = revolve_half
    # Use abs(volume) since winding may be inverted for revolution geometry
    assert abs(abs(mesh.volume) - EXPECTED_VOLUME) < 20.0, (
        f"Half-revolve volume should be ~{EXPECTED_VOLUME:.1f}, got {mesh.volume:.1f}"
    )


def test_revolve_half_bounding_box(revolve_half):
    mesh, _ = revolve_half
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # X: [-10, 10] (the half-annulus spans full X range)
    assert abs(bbox_min[0] - (-10.0)) < 0.5, f"Bbox min x should be ~-10, got {bbox_min[0]}"
    assert abs(bbox_max[0] - 10.0) < 0.5, f"Bbox max x should be ~10, got {bbox_max[0]}"
    # Y: [0, 3]
    assert abs(bbox_min[1] - 0.0) < 0.5, f"Bbox min y should be ~0, got {bbox_min[1]}"
    assert abs(bbox_max[1] - 3.0) < 0.5, f"Bbox max y should be ~3, got {bbox_max[1]}"
    # Z: [-10, 0] (180-degree revolve around Y rotates profile from XZ plane into -Z half)
    assert abs(bbox_min[2] - (-10.0)) < 0.5, f"Bbox min z should be ~-10, got {bbox_min[2]}"
    assert abs(bbox_max[2] - 0.0) < 0.5, f"Bbox max z should be ~0, got {bbox_max[2]}"


def test_revolve_half_matches_kernel_volume(revolve_half):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = revolve_half
    assert abs(abs(mesh.volume) - abs(props["volume"])) < 5.0, (
        f"trimesh volume ({mesh.volume:.1f}) should match kernel ({props['volume']:.1f})"
    )
