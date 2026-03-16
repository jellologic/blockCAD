"""Cross-validate stress test: boolean union + fillet + shell against trimesh.

Geometry: Two 10x5x7 boxes (A at origin, B offset 5 in X) unioned via
csg_union(), then fillet(edge 0, r=0.5), then shell(top face removed, t=0.5).

4-op chain: Extrude -> BooleanUnion -> Fillet -> Shell.

Union result: 15x5x7 box (volume 525).
Fillet removes a small amount of material from one edge.
Shell removes the top face and hollows out the interior with 0.5mm walls.
Expected bounding box: ~15x5x7.
Volume: between 50 and 525 (shelled hollow body).
"""

import pytest


@pytest.mark.xfail(reason="Boolean+fillet+shell produces non-watertight mesh (per-face tessellation limitation)")
def test_stress_boolean_fillet_shell_is_watertight(stress_boolean_fillet_shell):
    mesh, _ = stress_boolean_fillet_shell
    assert mesh.is_watertight, "Stress boolean+fillet+shell mesh should be watertight"


def test_stress_boolean_fillet_shell_volume_bounds(stress_boolean_fillet_shell):
    mesh, _ = stress_boolean_fillet_shell
    vol = abs(mesh.volume)
    assert 50.0 < vol < 525.0, (
        f"Stress boolean+fillet+shell volume should be between 50 and 525, got {vol}"
    )


def test_stress_boolean_fillet_shell_bounding_box(stress_boolean_fillet_shell):
    mesh, _ = stress_boolean_fillet_shell
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Bounding box should be approximately 15x5x7 (union of two offset 10x5x7 boxes)
    assert abs(bbox_max[0] - bbox_min[0] - 15.0) < 1.0, (
        f"Bbox X extent should be ~15, got {bbox_max[0] - bbox_min[0]}"
    )
    assert abs(bbox_max[1] - bbox_min[1] - 5.0) < 1.0, (
        f"Bbox Y extent should be ~5, got {bbox_max[1] - bbox_min[1]}"
    )
    assert abs(bbox_max[2] - bbox_min[2] - 7.0) < 1.0, (
        f"Bbox Z extent should be ~7, got {bbox_max[2] - bbox_min[2]}"
    )


def test_stress_boolean_fillet_shell_matches_kernel_volume(stress_boolean_fillet_shell):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = stress_boolean_fillet_shell
    assert abs(abs(mesh.volume) - abs(props["volume"])) < 10.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
