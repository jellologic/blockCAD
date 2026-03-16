"""Cross-validate stress test: Extrude 10x5x7 box -> Shell(top removed, t=0.5) -> Draft(2 side faces, 5 deg).

The shell hollows out the box (outer 10x5x7, inner 9x4x6.5, open top), producing
a volume of approximately 116. The draft then tilts 2 side faces by 5 degrees,
which slightly changes the volume but keeps the bounding box close to 10x5x7.
"""


def test_watertight(stress_box_shell_draft):
    mesh, _ = stress_box_shell_draft
    assert mesh.is_watertight, "Stress shell+draft mesh should be watertight"


def test_volume_bounds(stress_box_shell_draft):
    mesh, _ = stress_box_shell_draft
    # Shell volume ~116; draft modifies geometry but stays in a reasonable range.
    assert mesh.volume > 50.0, f"Volume should be > 50, got {mesh.volume}"
    assert mesh.volume < 200.0, f"Volume should be < 200, got {mesh.volume}"


def test_bounding_box(stress_box_shell_draft):
    mesh, _ = stress_box_shell_draft
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Draft may widen/shift the bbox slightly, but it should stay close to 10x5x7.
    assert bbox_max[0] - bbox_min[0] > 8.0, f"X extent should be > 8, got {bbox_max[0] - bbox_min[0]}"
    assert bbox_max[0] - bbox_min[0] < 14.0, f"X extent should be < 14, got {bbox_max[0] - bbox_min[0]}"
    assert bbox_max[1] - bbox_min[1] > 3.0, f"Y extent should be > 3, got {bbox_max[1] - bbox_min[1]}"
    assert bbox_max[1] - bbox_min[1] < 9.0, f"Y extent should be < 9, got {bbox_max[1] - bbox_min[1]}"
    assert bbox_max[2] - bbox_min[2] > 5.0, f"Z extent should be > 5, got {bbox_max[2] - bbox_min[2]}"
    assert bbox_max[2] - bbox_min[2] < 9.0, f"Z extent should be < 9, got {bbox_max[2] - bbox_min[2]}"


def test_kernel_volume_match(stress_box_shell_draft):
    """trimesh volume should match blockCAD kernel's divergence-theorem volume."""
    mesh, props = stress_box_shell_draft
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume:.2f}) should match kernel ({props['volume']:.2f})"
    )
