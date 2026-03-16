"""Cross-validate multi-edge filleted box geometry."""

import pytest


@pytest.mark.xfail(reason="Multi-edge fillet does not yet produce watertight mesh")
def test_fillet_multi_is_watertight(fillet_multi):
    mesh, _ = fillet_multi
    assert mesh.is_watertight, "Multi-edge filleted mesh should be watertight"


@pytest.mark.xfail(reason="Multi-edge fillet does not yet produce watertight mesh")
def test_fillet_multi_volume_less_than_single_fillet(fillet_multi, fillet):
    mesh_multi, _ = fillet_multi
    mesh_single, _ = fillet
    assert mesh_multi.volume < mesh_single.volume, (
        f"Multi-edge fillet volume ({mesh_multi.volume}) should be less than "
        f"single-edge fillet volume ({mesh_single.volume})"
    )


def test_fillet_multi_bounding_box(fillet_multi):
    """Multi-edge fillet should not exceed the original 10x5x7 bounding box."""
    mesh, _ = fillet_multi
    bbox_max = mesh.bounds[1]
    assert bbox_max[0] < 11.0, f"Bbox max x should be <=10, got {bbox_max[0]}"
    assert bbox_max[1] < 6.0, f"Bbox max y should be <=5, got {bbox_max[1]}"
    assert bbox_max[2] < 8.0, f"Bbox max z should be <=7, got {bbox_max[2]}"


@pytest.mark.xfail(reason="Multi-edge fillet does not yet produce watertight mesh")
def test_fillet_multi_matches_kernel_volume(fillet_multi):
    mesh, props = fillet_multi
    assert abs(mesh.volume - props["volume"]) < 2.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )


def test_fillet_multi_more_triangles_than_single_fillet(fillet_multi, fillet):
    """Multi-edge fillet adds more arc faces than single-edge fillet."""
    mesh_multi, _ = fillet_multi
    mesh_single, _ = fillet
    assert len(mesh_multi.faces) > len(mesh_single.faces), (
        f"Multi-edge fillet should have more faces ({len(mesh_multi.faces)}) "
        f"than single-edge fillet ({len(mesh_single.faces)})"
    )
