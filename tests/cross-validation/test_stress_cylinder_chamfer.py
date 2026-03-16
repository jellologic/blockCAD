"""Stress test: Cylinder (circle r=5, extruded h=10 along Z) with Chamfer(edge 0, d=0.5)."""

import math


def test_watertight(cylinder_chamfer):
    mesh, _ = cylinder_chamfer
    assert mesh.is_watertight, "Chamfered cylinder mesh should be watertight"


def test_volume_less_than_full_cylinder(cylinder_chamfer):
    mesh, _ = cylinder_chamfer
    full_cylinder_vol = math.pi * 25.0 * 10.0  # ~785.4
    assert mesh.volume < full_cylinder_vol, (
        f"Chamfered cylinder volume ({mesh.volume:.1f}) should be < full cylinder ({full_cylinder_vol:.1f})"
    )


def test_bounding_box(cylinder_chamfer):
    mesh, _ = cylinder_chamfer
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Cylinder r=5 centered at origin, height 10 along Z
    assert bbox_min[0] < -4.5, f"Bbox min x should be ~-5, got {bbox_min[0]}"
    assert bbox_max[0] > 4.5, f"Bbox max x should be ~5, got {bbox_max[0]}"
    assert bbox_max[2] > 9.0, f"Bbox max z should be ~10, got {bbox_max[2]}"
    assert bbox_min[2] < 0.5, f"Bbox min z should be ~0, got {bbox_min[2]}"


def test_kernel_volume_match(cylinder_chamfer):
    mesh, props = cylinder_chamfer
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume:.1f}) should match kernel ({props['volume']:.1f})"
    )
