"""Cross-validate stress test: 10x5x7 box with fillet(r=1) then chamfer(d=0.5)."""


def test_watertight(stress_box_fillet_chamfer):
    mesh, _ = stress_box_fillet_chamfer
    assert mesh.is_watertight, "Fillet+chamfer mesh should be watertight"


def test_volume_below_350(stress_box_fillet_chamfer):
    mesh, _ = stress_box_fillet_chamfer
    assert mesh.volume < 350.0, f"Fillet+chamfer should reduce volume below 350, got {mesh.volume}"


def test_bounding_box_approx_10x5x7(stress_box_fillet_chamfer):
    mesh, _ = stress_box_fillet_chamfer
    extents = mesh.bounds[1] - mesh.bounds[0]
    assert abs(extents[0] - 10.0) < 1.0, f"X extent should be ~10, got {extents[0]}"
    assert abs(extents[1] - 5.0) < 1.0, f"Y extent should be ~5, got {extents[1]}"
    assert abs(extents[2] - 7.0) < 1.0, f"Z extent should be ~7, got {extents[2]}"


def test_kernel_volume_match(stress_box_fillet_chamfer):
    mesh, props = stress_box_fillet_chamfer
    assert abs(mesh.volume - props["volume"]) < 2.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
