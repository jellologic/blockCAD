"""Cross-validate stress test: Extrude 10x10x10 box -> Shell(top removed, t=4.0).

Nearly solid — inner cavity is only 2x2x6.
Kernel volume ~827.
"""


def test_watertight(stress_thick_shell):
    mesh, _ = stress_thick_shell
    assert mesh.is_watertight, "Thick shell mesh should be watertight"


def test_volume_bounds(stress_thick_shell):
    mesh, _ = stress_thick_shell
    # Shell volume ~827: outer 10x10x10 minus inner cavity
    assert mesh.volume > 750.0, f"Volume should be > 750, got {mesh.volume}"
    assert mesh.volume < 900.0, f"Volume should be < 900, got {mesh.volume}"


def test_bounding_box(stress_thick_shell):
    mesh, _ = stress_thick_shell
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Outer box is 10x10x10
    for axis, label in enumerate(["X", "Y", "Z"]):
        extent = bbox_max[axis] - bbox_min[axis]
        assert abs(extent - 10.0) < 1.0, (
            f"{label} extent should be ~10, got {extent}"
        )


def test_kernel_volume_match(stress_thick_shell):
    """trimesh volume should match blockCAD kernel's divergence-theorem volume."""
    mesh, props = stress_thick_shell
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume:.2f}) should match kernel ({props['volume']:.2f})"
    )
