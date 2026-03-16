"""Cross-validate stress test: Extrude 10x5x7 box -> Shell(top removed, t=0.5) -> Fillet(edge 0, r=0.3) -> Mirror(YZ at x=0).

Hollow -> round -> double.
Shell produces open-top box with wall thickness 0.5.
Shell volume ~116 (10x5x7 outer - 9x4x6.5 inner).
Fillet rounds one edge (r=0.3), removing a tiny amount of material.
Mirror doubles the body across YZ plane at x=0, spanning [-10,10] x [0,5] x [0,7].
Expected total volume: ~232 (2 * 116 minus small fillet removal).
"""

def test_watertight(stress_full_part_3):
    mesh, _ = stress_full_part_3
    assert mesh.is_watertight, "Shell+fillet+mirror mesh should be watertight"


def test_volume_bounds(stress_full_part_3):
    mesh, _ = stress_full_part_3
    # Mirrored shelled box: ~232, fillet removes a small amount
    assert mesh.volume > 150.0, f"Volume should be > 150, got {mesh.volume}"
    assert mesh.volume < 300.0, f"Volume should be < 300, got {mesh.volume}"


def test_bounding_box(stress_full_part_3):
    mesh, _ = stress_full_part_3
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # X spans [-10, 10] after mirror
    assert abs(bbox_min[0] - (-10.0)) < 0.5, f"Bbox min x should be ~-10, got {bbox_min[0]}"
    assert abs(bbox_max[0] - 10.0) < 0.5, f"Bbox max x should be ~10, got {bbox_max[0]}"
    # Y spans [0, 5]
    assert abs(bbox_min[1] - 0.0) < 0.5, f"Bbox min y should be ~0, got {bbox_min[1]}"
    assert abs(bbox_max[1] - 5.0) < 0.5, f"Bbox max y should be ~5, got {bbox_max[1]}"
    # Z spans [0, 7]
    assert abs(bbox_min[2] - 0.0) < 0.5, f"Bbox min z should be ~0, got {bbox_min[2]}"
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_kernel_volume_match(stress_full_part_3):
    """trimesh volume should match blockCAD kernel divergence-theorem volume."""
    mesh, props = stress_full_part_3
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume:.2f}) should match kernel ({props['volume']:.2f})"
    )
