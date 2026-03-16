"""Cross-validate cylinder (circle extrude) geometry."""

import math


def test_cylinder_is_watertight(cylinder):
    mesh, _ = cylinder
    assert mesh.is_watertight, "Cylinder mesh should be watertight"


def test_cylinder_volume(cylinder):
    mesh, _ = cylinder
    expected = math.pi * 25.0 * 10.0  # π * r² * h ≈ 785.4
    assert abs(mesh.volume - expected) < 30.0, (
        f"Cylinder volume should be ~{expected:.0f}, got {mesh.volume:.1f}"
    )


def test_cylinder_bounding_box(cylinder):
    mesh, _ = cylinder
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Cylinder r=5 centered at origin, height 10 along z
    assert bbox_min[0] < -4.5, f"Bbox min x should be ~-5, got {bbox_min[0]}"
    assert bbox_max[0] > 4.5, f"Bbox max x should be ~5, got {bbox_max[0]}"
    assert abs(bbox_max[2] - 10.0) < 0.5, f"Height should be ~10, got {bbox_max[2]}"


def test_cylinder_matches_kernel_volume(cylinder):
    mesh, props = cylinder
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume:.1f}) should match kernel ({props['volume']:.1f})"
    )


def test_cylinder_surface_area(cylinder):
    mesh, _ = cylinder
    # Exact: 2πr² + 2πrh = 2π*25 + 2π*5*10 = 50π + 100π = 150π ≈ 471.2
    expected = 2 * math.pi * 25 + 2 * math.pi * 5 * 10
    assert abs(mesh.area - expected) < 30.0, (
        f"Cylinder surface area should be ~{expected:.0f}, got {mesh.area:.1f}"
    )
