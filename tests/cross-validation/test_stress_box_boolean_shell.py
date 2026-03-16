"""Cross-validate stress test: boolean union + shell against trimesh.

Geometry: Two 10x5x7 boxes (A at origin, B offset 5 in X) unioned, then
shelled with top face removed and wall thickness 0.5.

Union result: 15x5x7 box.
Shell removes the top face and hollows out with t=0.5 walls.
Expected bounding box: ~15x5x7.
Volume: outer 525 minus inner cavity, should be between 100 and 500.
"""


def test_stress_box_boolean_shell_is_watertight(stress_box_boolean_shell):
    mesh, _ = stress_box_boolean_shell
    assert mesh.is_watertight, "Stress boolean+shell mesh should be watertight"


def test_stress_box_boolean_shell_volume_bounds(stress_box_boolean_shell):
    mesh, _ = stress_box_boolean_shell
    vol = abs(mesh.volume)
    assert 50.0 < vol < 525.0, (
        f"Stress boolean+shell volume should be between 50 and 525, got {vol}"
    )


def test_stress_box_boolean_shell_bounding_box(stress_box_boolean_shell):
    mesh, _ = stress_box_boolean_shell
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Bounding box should be approximately 15x5x7
    assert abs(bbox_max[0] - bbox_min[0] - 15.0) < 1.0, (
        f"Bbox X extent should be ~15, got {bbox_max[0] - bbox_min[0]}"
    )
    assert abs(bbox_max[1] - bbox_min[1] - 5.0) < 1.0, (
        f"Bbox Y extent should be ~5, got {bbox_max[1] - bbox_min[1]}"
    )
    assert abs(bbox_max[2] - bbox_min[2] - 7.0) < 1.0, (
        f"Bbox Z extent should be ~7, got {bbox_max[2] - bbox_min[2]}"
    )


def test_stress_box_boolean_shell_matches_kernel_volume(stress_box_boolean_shell):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = stress_box_boolean_shell
    assert abs(abs(mesh.volume) - abs(props["volume"])) < 10.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
