"""Stress test: 4x4 square swept 10 units along Z, then Shell(1 face removed, t=0.3).

Geometry: sweep_profile produces a 4x4x10 box-like solid (volume = 160).
After shelling with thickness 0.3 and the top face removed, the solid becomes
a hollow open-top container. The inner cavity is (4-0.6)x(4-0.6)x(10-0.3) =
3.4 x 3.4 x 9.7 = 112.148. So expected shelled volume ~ 160 - 112.148 ~ 47.85.

Bounding box: the sweep spans x=[-2,2], y=[-2,2], z=[0,10]. Shelling does not
change the outer bounding box.
"""

SOLID_SWEEP_VOLUME = 4.0 * 4.0 * 10.0  # 160


def test_watertight(stress_sweep_shell):
    mesh, _ = stress_sweep_shell
    assert mesh.is_watertight, "Sweep+shell mesh should be watertight (closed solid)"


def test_volume_less_than_solid_sweep(stress_sweep_shell):
    mesh, _ = stress_sweep_shell
    assert abs(mesh.volume) < SOLID_SWEEP_VOLUME, (
        f"Shelled sweep volume ({mesh.volume:.1f}) should be < solid sweep ({SOLID_SWEEP_VOLUME:.1f})"
    )


def test_bounding_box(stress_sweep_shell):
    mesh, _ = stress_sweep_shell
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # X: [-2, 2]
    assert abs(bbox_min[0] - (-2.0)) < 0.5, f"Bbox min x should be ~-2, got {bbox_min[0]}"
    assert abs(bbox_max[0] - 2.0) < 0.5, f"Bbox max x should be ~2, got {bbox_max[0]}"
    # Y: [-2, 2]
    assert abs(bbox_min[1] - (-2.0)) < 0.5, f"Bbox min y should be ~-2, got {bbox_min[1]}"
    assert abs(bbox_max[1] - 2.0) < 0.5, f"Bbox max y should be ~2, got {bbox_max[1]}"
    # Z: [0, 10]
    assert abs(bbox_min[2]) < 0.5, f"Bbox min z should be ~0, got {bbox_min[2]}"
    assert abs(bbox_max[2] - 10.0) < 0.5, f"Bbox max z should be ~10, got {bbox_max[2]}"


def test_kernel_volume_match(stress_sweep_shell):
    """trimesh volume should match blockCAD kernel divergence-theorem volume."""
    mesh, props = stress_sweep_shell
    assert abs(abs(mesh.volume) - abs(props["volume"])) < 5.0, (
        f"trimesh volume ({mesh.volume:.1f}) should match kernel ({props['volume']:.1f})"
    )
