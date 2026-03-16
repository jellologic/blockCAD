"""Cross-validate shelled box geometry against trimesh (independent mesh library).

Geometry: 10x5x7 box, top face removed, wall thickness 0.5.
Inner cavity: 9x4x6.5 (offset 0.5 on 5 kept faces, open at top).
Expected shell volume: 350 - 234 = 116.
"""


def test_shell_is_watertight(shell):
    mesh, _ = shell
    assert mesh.is_watertight, "Shell mesh should be watertight (closed solid)"


def test_shell_volume(shell):
    mesh, _ = shell
    # outer 10*5*7 = 350, inner 9*4*6.5 = 234, shell = 116
    assert abs(mesh.volume - 116.0) < 5.0, f"Shell volume should be ~116, got {mesh.volume}"


def test_shell_volume_less_than_solid(shell, box):
    mesh, _ = shell
    box_mesh, _ = box
    assert mesh.volume < box_mesh.volume, (
        f"Shell volume ({mesh.volume}) should be less than solid box ({box_mesh.volume})"
    )


def test_shell_surface_area(shell):
    mesh, _ = shell
    # Outer: 2*(10*5 + 10*7 + 5*7) = 310, minus top face (10*5=50) = 260
    # Inner: 2*(9*4 + 9*6.5 + 4*6.5) = 2*(36+58.5+26) = 241, minus top (9*4=36) = 205
    # Rim: 4 faces around the opening, each is a thin strip:
    #   front/back rims: 10 * 0.5 and 9 * something... actually the rim faces
    #   connect outer edge to inner edge at the opening.
    # Use a loose tolerance — exact area depends on rim geometry
    assert mesh.area > 400.0, f"Shell area should be > 400, got {mesh.area}"
    assert mesh.area < 600.0, f"Shell area should be < 600, got {mesh.area}"


def test_shell_bounding_box(shell):
    mesh, _ = shell
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Outer bounding box should still be 10x5x7
    assert all(abs(v) < 0.5 for v in bbox_min), f"Bbox min should be near origin, got {bbox_min}"
    assert abs(bbox_max[0] - 10.0) < 0.5, f"Bbox max x should be ~10, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 5.0) < 0.5, f"Bbox max y should be ~5, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_shell_matches_kernel_volume(shell):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = shell
    assert abs(mesh.volume - props["volume"]) < 2.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )


def test_shell_matches_kernel_area(shell):
    mesh, props = shell
    assert abs(mesh.area - props["surface_area"]) < 5.0, (
        f"trimesh area ({mesh.area}) should match kernel ({props['surface_area']})"
    )
