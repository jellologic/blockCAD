"""Cross-validate stress test: flat extrude against trimesh.

Geometry: 20x20 rectangle sketch extruded 0.5mm along Z.
Very flat geometry (aspect ratio 40:1) stresses tessellation and
normal computation.

Expected volume: 20 * 20 * 0.5 = 200.
Bounding box: ~20 x 20 x 0.5.
"""


def test_stress_flat_extrude_is_watertight(stress_flat_extrude):
    mesh, _ = stress_flat_extrude
    assert mesh.is_watertight, "Flat extrude mesh should be watertight"


def test_stress_flat_extrude_volume(stress_flat_extrude):
    mesh, _ = stress_flat_extrude
    vol = abs(mesh.volume)
    assert abs(vol - 200.0) < 5.0, (
        f"Flat extrude volume should be ~200, got {vol}"
    )


def test_stress_flat_extrude_bounding_box(stress_flat_extrude):
    mesh, _ = stress_flat_extrude
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Bounding box should be approximately 20x20x0.5
    assert abs(bbox_max[0] - bbox_min[0] - 20.0) < 1.0, (
        f"Bbox X extent should be ~20, got {bbox_max[0] - bbox_min[0]}"
    )
    assert abs(bbox_max[1] - bbox_min[1] - 20.0) < 1.0, (
        f"Bbox Y extent should be ~20, got {bbox_max[1] - bbox_min[1]}"
    )
    assert abs(bbox_max[2] - bbox_min[2] - 0.5) < 0.1, (
        f"Bbox Z extent should be ~0.5, got {bbox_max[2] - bbox_min[2]}"
    )


def test_stress_flat_extrude_matches_kernel_volume(stress_flat_extrude):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = stress_flat_extrude
    assert abs(abs(mesh.volume) - abs(props["volume"])) < 5.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
