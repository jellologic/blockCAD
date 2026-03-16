"""Cross-validate stress test: Extrude 20x20x20 box -> Shell(top removed, t=0.2).

Very thin walls (0.2mm) stress the shell operation and tessellator. The solid
box volume is 8000; the thin shell should be much less (approx 398, computed as
outer 20x20x20 minus inner 19.6x19.6x19.8).
"""


def test_watertight(stress_thin_shell):
    mesh, _ = stress_thin_shell
    assert mesh.is_watertight, "Thin shell mesh should be watertight"


def test_volume_bounds(stress_thin_shell):
    mesh, _ = stress_thin_shell
    # Approximate shell volume ~398. Allow generous bounds.
    assert mesh.volume > 100.0, f"Volume should be > 100, got {mesh.volume}"
    assert mesh.volume < 1000.0, f"Volume should be < 1000 (solid box is 8000), got {mesh.volume}"


def test_bounding_box(stress_thin_shell):
    mesh, _ = stress_thin_shell
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Bounding box should be approximately 20x20x20.
    for axis, label in enumerate(["X", "Y", "Z"]):
        extent = bbox_max[axis] - bbox_min[axis]
        assert extent > 18.0, f"{label} extent should be > 18, got {extent}"
        assert extent < 22.0, f"{label} extent should be < 22, got {extent}"


def test_kernel_volume_match(stress_thin_shell):
    """trimesh volume should match blockCAD kernel's divergence-theorem volume."""
    mesh, props = stress_thin_shell
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume:.2f}) should match kernel ({props['volume']:.2f})"
    )
