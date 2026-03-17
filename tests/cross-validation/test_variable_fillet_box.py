"""Cross-validate variable fillet on 10x10x10 box.

Variable fillet with radius 1.0 at start to 2.0 at end on edge 0.
The fillet removes material, so volume should be less than the original 1000.
"""


def test_watertight(variable_fillet_box):
    mesh, _ = variable_fillet_box
    assert mesh.is_watertight, "Variable fillet mesh should be watertight"


def test_volume_reduced(variable_fillet_box):
    mesh, _ = variable_fillet_box
    assert mesh.volume < 1000.0, f"Variable fillet should reduce volume below 1000, got {mesh.volume}"
    assert mesh.volume > 500.0, f"Variable fillet volume should be > 500, got {mesh.volume}"


def test_bounding_box(variable_fillet_box):
    mesh, _ = variable_fillet_box
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Bounding box should still be approximately 10x10x10
    assert bbox_max[0] - bbox_min[0] < 12.0, f"X extent should be < 12, got {bbox_max[0] - bbox_min[0]}"
    assert bbox_max[1] - bbox_min[1] < 12.0, f"Y extent should be < 12, got {bbox_max[1] - bbox_min[1]}"
    assert bbox_max[2] - bbox_min[2] < 12.0, f"Z extent should be < 12, got {bbox_max[2] - bbox_min[2]}"


def test_more_triangles_than_box(variable_fillet_box):
    """Variable fillet adds arc faces -> more triangles than a plain box."""
    mesh, _ = variable_fillet_box
    # A plain box has 12 triangles; variable fillet should have many more
    assert len(mesh.faces) > 12, (
        f"Variable fillet should have more faces than a plain box, got {len(mesh.faces)}"
    )


def test_kernel_volume_match(variable_fillet_box):
    """trimesh volume should match blockCAD kernel's divergence-theorem volume."""
    mesh, props = variable_fillet_box
    assert abs(mesh.volume - props["volume"]) < 10.0, (
        f"trimesh volume ({mesh.volume:.2f}) should match kernel ({props['volume']:.2f})"
    )
