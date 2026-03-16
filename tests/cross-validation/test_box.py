"""Cross-validate 10x5x7 box geometry against trimesh (independent mesh library)."""

import math


def test_box_is_watertight(box):
    mesh, _ = box
    assert mesh.is_watertight, "Box mesh should be watertight (closed solid)"


def test_box_volume(box):
    mesh, _ = box
    assert abs(mesh.volume - 350.0) < 2.0, f"Box volume should be ~350, got {mesh.volume}"


def test_box_surface_area(box):
    mesh, _ = box
    # 2*(10*5 + 10*7 + 5*7) = 2*(50+70+35) = 310
    assert abs(mesh.area - 310.0) < 5.0, f"Box area should be ~310, got {mesh.area}"


def test_box_bounding_box(box):
    mesh, _ = box
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    assert all(abs(v) < 0.5 for v in bbox_min), f"Bbox min should be near origin, got {bbox_min}"
    assert abs(bbox_max[0] - 10.0) < 0.5, f"Bbox max x should be ~10, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 5.0) < 0.5, f"Bbox max y should be ~5, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_box_center_of_mass(box):
    mesh, _ = box
    com = mesh.center_mass
    assert abs(com[0] - 5.0) < 0.5, f"CoM x should be ~5, got {com[0]}"
    assert abs(com[1] - 2.5) < 0.5, f"CoM y should be ~2.5, got {com[1]}"
    assert abs(com[2] - 3.5) < 0.5, f"CoM z should be ~3.5, got {com[2]}"


def test_box_matches_kernel_volume(box):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = box
    assert abs(mesh.volume - props["volume"]) < 1.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )


def test_box_matches_kernel_area(box):
    mesh, props = box
    assert abs(mesh.area - props["surface_area"]) < 2.0, (
        f"trimesh area ({mesh.area}) should match kernel ({props['surface_area']})"
    )
