"""Cross-validate sweep geometry against trimesh (independent mesh library).

Geometry: 4x4 square profile swept 10 units along Z axis (straight path).
Expected shape: rectangular prism 4x4x10, volume = 160.
"""


def test_sweep_is_watertight(sweep):
    mesh, _ = sweep
    assert mesh.is_watertight, "Sweep mesh should be watertight (closed solid)"


def test_sweep_volume(sweep):
    mesh, _ = sweep
    # 4x4 square swept 10 units = 160 cubic units
    assert abs(mesh.volume - 160.0) < 5.0, f"Sweep volume should be ~160, got {mesh.volume}"


def test_sweep_bounding_box(sweep):
    mesh, _ = sweep
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Profile is centered at origin: -2..2 in X and Y, swept 0..10 in Z
    assert abs(bbox_min[0] - (-2.0)) < 0.5, f"Bbox min x should be ~-2, got {bbox_min[0]}"
    assert abs(bbox_min[1] - (-2.0)) < 0.5, f"Bbox min y should be ~-2, got {bbox_min[1]}"
    assert abs(bbox_min[2] - 0.0) < 0.5, f"Bbox min z should be ~0, got {bbox_min[2]}"
    assert abs(bbox_max[0] - 2.0) < 0.5, f"Bbox max x should be ~2, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 2.0) < 0.5, f"Bbox max y should be ~2, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 10.0) < 0.5, f"Bbox max z should be ~10, got {bbox_max[2]}"


def test_sweep_matches_kernel_volume(sweep):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = sweep
    assert abs(mesh.volume - props["volume"]) < 2.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )


def test_sweep_matches_kernel_area(sweep):
    mesh, props = sweep
    assert abs(mesh.area - props["surface_area"]) < 5.0, (
        f"trimesh area ({mesh.area}) should match kernel ({props['surface_area']})"
    )
