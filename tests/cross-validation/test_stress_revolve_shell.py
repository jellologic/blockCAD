"""Stress test: full 360-degree revolve of rectangle (x=[5,10], y=[0,3] around
Y-axis) followed by Shell (1 face removed, thickness=0.5).

The solid revolve produces an annular washer with volume pi*(10^2-5^2)*3 = 225*pi ~ 706.86.
After shelling with t=0.5, material is removed from the interior, so the
resulting volume must be strictly less than 707.

Bounding box of the outer surface is still [-10,10] x [0,3] x [-10,10].
"""

import math


# Full-revolve analytical volume (before shell)
FULL_REVOLVE_VOLUME = math.pi * 75.0 * 3.0  # ~706.86


def test_watertight(revolve_shell):
    mesh, _ = revolve_shell
    assert mesh.is_watertight, "Revolve+shell mesh should be watertight (closed solid)"


def test_volume_less_than_full_revolve(revolve_shell):
    mesh, _ = revolve_shell
    assert abs(mesh.volume) < FULL_REVOLVE_VOLUME, (
        f"Shelled revolve volume ({mesh.volume:.1f}) should be < full revolve ({FULL_REVOLVE_VOLUME:.1f})"
    )


def test_bounding_box(revolve_shell):
    mesh, _ = revolve_shell
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Shell adds t=0.5 outward, so outer radius expands to ~10.5
    # and inner radius shrinks to ~4.5.  Bounding box: ~[-10.5, 10.5] x [-0.5, 3.5] x [-10.5, 10.5]
    # X: [-10.5, 10.5]
    assert abs(bbox_min[0] - (-10.5)) < 1.0, f"Bbox min x should be ~-10.5, got {bbox_min[0]}"
    assert abs(bbox_max[0] - 10.5) < 1.0, f"Bbox max x should be ~10.5, got {bbox_max[0]}"
    # Y: [-0.5, 3.5]  (shell grows outward from top/bottom faces)
    assert abs(bbox_min[1]) < 1.0, f"Bbox min y should be near 0, got {bbox_min[1]}"
    assert abs(bbox_max[1] - 3.5) < 1.0, f"Bbox max y should be ~3.5, got {bbox_max[1]}"
    # Z: [-10.5, 10.5]
    assert abs(bbox_min[2] - (-10.5)) < 1.0, f"Bbox min z should be ~-10.5, got {bbox_min[2]}"
    assert abs(bbox_max[2] - 10.5) < 1.0, f"Bbox max z should be ~10.5, got {bbox_max[2]}"


def test_kernel_volume_match(revolve_shell):
    """trimesh volume should match blockCAD kernel divergence-theorem volume."""
    mesh, props = revolve_shell
    assert abs(abs(mesh.volume) - abs(props["volume"])) < 5.0, (
        f"trimesh volume ({mesh.volume:.1f}) should match kernel ({props['volume']:.1f})"
    )
