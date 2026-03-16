"""Stress test: L-shaped extrude (10x10 minus 5x5 corner, extruded 5mm) shelled
with top face removed and wall thickness 0.5mm.

This tests the kernel's ability to shell a non-convex profile (L-shape),
which is significantly harder than shelling a simple box.

Solid L-shape volume = (10*10 - 5*5) * 5 = 375.
The shelled result must be strictly less than 375.
Bounding box should remain ~10x10x5 (outer dimensions unchanged).
"""


def test_watertight(l_shape_shell):
    mesh, _ = l_shape_shell
    assert mesh.is_watertight, "L-shape shell mesh should be watertight (closed solid)"


def test_volume_less_than_solid(l_shape_shell):
    mesh, _ = l_shape_shell
    assert mesh.volume < 375.0, (
        f"Shell volume ({mesh.volume}) should be less than solid L-shape (375)"
    )


def test_bounding_box(l_shape_shell):
    mesh, _ = l_shape_shell
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    assert all(abs(v) < 0.5 for v in bbox_min), f"Bbox min should be near origin, got {bbox_min}"
    assert abs(bbox_max[0] - 10.0) < 0.5, f"Bbox max x should be ~10, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 10.0) < 0.5, f"Bbox max y should be ~10, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 5.0) < 0.5, f"Bbox max z should be ~5, got {bbox_max[2]}"


def test_kernel_volume_match(l_shape_shell):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = l_shape_shell
    assert abs(mesh.volume - props["volume"]) < 2.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
