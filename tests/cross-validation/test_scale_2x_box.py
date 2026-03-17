"""Cross-validate scale body: 5x5x5 box scaled 2x uniformly.

Expected volume: 125 * 8 = 1000.
Expected bounding box: approximately 10x10x10.
"""


def test_watertight(scale_2x_box):
    mesh, _ = scale_2x_box
    assert mesh.is_watertight, "Scaled box mesh should be watertight"


def test_volume_8x(scale_2x_box):
    mesh, _ = scale_2x_box
    expected = 1000.0  # 5^3 * 2^3 = 125 * 8
    assert abs(mesh.volume - expected) < 50.0, (
        f"Scaled 2x box volume should be ~{expected}, got {mesh.volume}"
    )


def test_bounding_box(scale_2x_box):
    mesh, _ = scale_2x_box
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # After 2x scale from origin, a 5x5x5 box at origin should become 10x10x10
    extent_x = bbox_max[0] - bbox_min[0]
    extent_y = bbox_max[1] - bbox_min[1]
    extent_z = bbox_max[2] - bbox_min[2]
    assert abs(extent_x - 10.0) < 1.0, f"X extent should be ~10, got {extent_x}"
    assert abs(extent_y - 10.0) < 1.0, f"Y extent should be ~10, got {extent_y}"
    assert abs(extent_z - 10.0) < 1.0, f"Z extent should be ~10, got {extent_z}"


def test_kernel_volume_match(scale_2x_box):
    """trimesh volume should match blockCAD kernel's divergence-theorem volume."""
    mesh, props = scale_2x_box
    assert abs(mesh.volume - props["volume"]) < 10.0, (
        f"trimesh volume ({mesh.volume:.2f}) should match kernel ({props['volume']:.2f})"
    )
