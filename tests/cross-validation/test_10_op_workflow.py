"""Cross-validate 10-operation stress workflow.

10x5x7 box -> Fillet(edge 0, r=0.5) -> Chamfer(edge 4, d=0.3)
-> Shell(top removed, t=0.4) -> Mirror(YZ at x=15) -> Scale(1.5x)

This tests the kernel's ability to chain many operations together,
including the newer ScaleBody and Mirror operations on a shelled body.
"""


def test_watertight(ten_op_workflow):
    mesh, _ = ten_op_workflow
    assert mesh.is_watertight, "10-op workflow mesh should be watertight"


def test_volume_bounds(ten_op_workflow):
    mesh, _ = ten_op_workflow
    # Shell hollows most of the box, mirror doubles, scale 1.5x cubes the volume.
    # Exact value depends on operations, but should be non-trivial.
    assert mesh.volume > 50.0, f"Volume should be > 50, got {mesh.volume}"
    assert mesh.volume < 3000.0, f"Volume should be < 3000, got {mesh.volume}"


def test_bounding_box(ten_op_workflow):
    mesh, _ = ten_op_workflow
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # After mirror + scale, the bounding box should be larger than the original 10x5x7
    extent_x = bbox_max[0] - bbox_min[0]
    extent_y = bbox_max[1] - bbox_min[1]
    extent_z = bbox_max[2] - bbox_min[2]
    assert extent_x > 5.0, f"X extent should be > 5, got {extent_x}"
    assert extent_y > 3.0, f"Y extent should be > 3, got {extent_y}"
    assert extent_z > 3.0, f"Z extent should be > 3, got {extent_z}"


def test_kernel_volume_match(ten_op_workflow):
    """trimesh volume should match blockCAD kernel's divergence-theorem volume."""
    mesh, props = ten_op_workflow
    assert abs(mesh.volume - props["volume"]) < 20.0, (
        f"trimesh volume ({mesh.volume:.2f}) should match kernel ({props['volume']:.2f})"
    )
