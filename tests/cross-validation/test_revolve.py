"""Cross-validate revolve geometry against trimesh (independent mesh library).

The revolve fixture is a 360-degree revolution of a rectangle (x=[5,10], y=[0,3])
around the Y-axis, producing an annular solid (washer/tube shape).

Analytical properties:
  - Volume: pi * (R_outer^2 - R_inner^2) * height = pi * (100 - 25) * 3 = 225*pi ~ 706.86
  - Bounding box: [-10, 10] x [0, 3] x [-10, 10]
"""

import math


# Analytical expected values
EXPECTED_VOLUME = math.pi * 75.0 * 3.0  # 225*pi ~ 706.86
# Surface area: inner cylinder + outer cylinder + top annulus + bottom annulus
# = 2*pi*5*3 + 2*pi*10*3 + 2*pi*(100-25) = 30*pi + 60*pi + 150*pi = 240*pi ~ 753.98
EXPECTED_AREA = math.pi * 240.0


def test_revolve_is_watertight(revolve):
    mesh, _ = revolve
    assert mesh.is_watertight, "Revolve mesh should be watertight (closed solid)"


def test_revolve_volume(revolve):
    mesh, _ = revolve
    # Use abs(volume) since winding may be inverted for revolution geometry
    assert abs(abs(mesh.volume) - EXPECTED_VOLUME) < 30.0, (
        f"Revolve volume should be ~{EXPECTED_VOLUME:.1f}, got {mesh.volume:.1f}"
    )


def test_revolve_bounding_box(revolve):
    mesh, _ = revolve
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # X: [-10, 10]
    assert abs(bbox_min[0] - (-10.0)) < 0.5, f"Bbox min x should be ~-10, got {bbox_min[0]}"
    assert abs(bbox_max[0] - 10.0) < 0.5, f"Bbox max x should be ~10, got {bbox_max[0]}"
    # Y: [0, 3]
    assert abs(bbox_min[1] - 0.0) < 0.5, f"Bbox min y should be ~0, got {bbox_min[1]}"
    assert abs(bbox_max[1] - 3.0) < 0.5, f"Bbox max y should be ~3, got {bbox_max[1]}"
    # Z: [-10, 10]
    assert abs(bbox_min[2] - (-10.0)) < 0.5, f"Bbox min z should be ~-10, got {bbox_min[2]}"
    assert abs(bbox_max[2] - 10.0) < 0.5, f"Bbox max z should be ~10, got {bbox_max[2]}"


def test_revolve_matches_kernel_volume(revolve):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = revolve
    assert abs(abs(mesh.volume) - abs(props["volume"])) < 5.0, (
        f"trimesh volume ({mesh.volume:.1f}) should match kernel ({props['volume']:.1f})"
    )


def test_revolve_matches_kernel_area(revolve):
    mesh, props = revolve
    assert abs(mesh.area - props["surface_area"]) < 10.0, (
        f"trimesh area ({mesh.area:.1f}) should match kernel ({props['surface_area']:.1f})"
    )
