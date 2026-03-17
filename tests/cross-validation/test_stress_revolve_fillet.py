"""Cross-validate stress test: full revolve + fillet.

The geometry is a 360-degree revolution of a rectangle (x=[5,10], y=[0,3])
around the Y-axis, producing an annular solid, followed by Fillet(edge 0, r=0.5).

Analytical properties (plain revolve, before fillet):
  - Volume: pi * (R_outer^2 - R_inner^2) * height = pi * (100 - 25) * 3 = 225*pi ~ 706.86
  - Bounding box: [-10, 10] x [0, 3] z [-10, 10]

The fillet removes a small amount of material, so volume should be ~707 +/- tolerance.
"""

import math
import pytest


# Plain revolve volume (before fillet)
PLAIN_REVOLVE_VOLUME = math.pi * 75.0 * 3.0  # ~706.86
VOLUME_TOLERANCE = 50.0


def test_watertight(stress_revolve_fillet):
    mesh, _ = stress_revolve_fillet
    assert mesh.is_watertight, "Revolve+fillet mesh should be watertight (closed solid)"


def test_volume(stress_revolve_fillet):
    mesh, _ = stress_revolve_fillet
    vol = abs(mesh.volume)
    assert abs(vol - PLAIN_REVOLVE_VOLUME) < VOLUME_TOLERANCE, (
        f"Revolve+fillet volume should be ~{PLAIN_REVOLVE_VOLUME:.1f} (±{VOLUME_TOLERANCE}), "
        f"got {mesh.volume:.1f}"
    )


def test_bounding_box(stress_revolve_fillet):
    mesh, _ = stress_revolve_fillet
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # X: [-10, 10]
    assert abs(bbox_min[0] - (-10.0)) < 1.0, f"Bbox min x should be ~-10, got {bbox_min[0]}"
    assert abs(bbox_max[0] - 10.0) < 1.0, f"Bbox max x should be ~10, got {bbox_max[0]}"
    # Y: [0, 3]
    assert abs(bbox_min[1] - 0.0) < 1.0, f"Bbox min y should be ~0, got {bbox_min[1]}"
    assert abs(bbox_max[1] - 3.0) < 1.0, f"Bbox max y should be ~3, got {bbox_max[1]}"
    # Z: [-10, 10]
    assert abs(bbox_min[2] - (-10.0)) < 1.0, f"Bbox min z should be ~-10, got {bbox_min[2]}"
    assert abs(bbox_max[2] - 10.0) < 1.0, f"Bbox max z should be ~10, got {bbox_max[2]}"


def test_kernel_volume_match(stress_revolve_fillet):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = stress_revolve_fillet
    assert abs(abs(mesh.volume) - abs(props["volume"])) < 20.0, (
        f"trimesh volume ({mesh.volume:.1f}) should match kernel ({props['volume']:.1f})"
    )
