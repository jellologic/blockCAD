"""Shared fixtures for cross-validation tests."""

import json
import os
import trimesh
import pytest

FIXTURES_DIR = os.path.join(os.path.dirname(__file__), "fixtures")


def load_stl(name: str) -> trimesh.Trimesh:
    """Load an STL fixture file as a trimesh."""
    path = os.path.join(FIXTURES_DIR, f"{name}.stl")
    return trimesh.load(path)


def load_kernel_props(name: str) -> dict:
    """Load blockCAD kernel mass properties JSON."""
    path = os.path.join(FIXTURES_DIR, f"{name}.json")
    with open(path) as f:
        return json.load(f)


@pytest.fixture
def box():
    """10x5x7 extruded box."""
    return load_stl("box_10x5x7"), load_kernel_props("box_10x5x7")


@pytest.fixture
def fillet():
    """10x5x7 box with r=1 fillet on one edge."""
    return load_stl("box_fillet_r1"), load_kernel_props("box_fillet_r1")


@pytest.fixture
def chamfer():
    """10x5x7 box with d=1 chamfer on one edge."""
    return load_stl("box_chamfer_d1"), load_kernel_props("box_chamfer_d1")


@pytest.fixture
def cylinder():
    """Cylinder r=5, h=10."""
    return load_stl("cylinder_r5_h10"), load_kernel_props("cylinder_r5_h10")


@pytest.fixture
def sweep():
    """4x4 square swept 10 units along Z axis."""
    return load_stl("sweep_straight"), load_kernel_props("sweep_straight")


@pytest.fixture
def sweep_twisted():
    """4x4 square swept 10 units along Z with 90-degree twist."""
    return load_stl("sweep_twisted"), load_kernel_props("sweep_twisted")


@pytest.fixture
def cut_extrude():
    """10x5x7 box with a 4x2x3 blind pocket cut from the bottom."""
    return load_stl("box_cut_pocket"), load_kernel_props("box_cut_pocket")


@pytest.fixture
def compound_cut_chamfer():
    """10x5x7 box with d=0.5 chamfer on one edge and 4x2x3 pocket."""
    return load_stl("compound_cut_chamfer"), load_kernel_props("compound_cut_chamfer")


@pytest.fixture
def shell():
    """10x5x7 box shelled with t=0.5, top face removed."""
    return load_stl("box_shell_t05"), load_kernel_props("box_shell_t05")


@pytest.fixture
def draft():
    """10x5x7 box with 5-degree draft on two side faces."""
    return load_stl("box_draft_5deg"), load_kernel_props("box_draft_5deg")


@pytest.fixture
def revolve():
    """Full 360-degree revolve of rectangle around Y-axis (annular solid)."""
    return load_stl("revolve_full"), load_kernel_props("revolve_full")


@pytest.fixture
def revolve_half():
    """180-degree revolve of rectangle around Y-axis (half-annulus)."""
    return load_stl("revolve_half"), load_kernel_props("revolve_half")


@pytest.fixture
def mirror():
    """10x5x7 box mirrored across YZ plane (X=0)."""
    return load_stl("mirror_box"), load_kernel_props("mirror_box")


@pytest.fixture
def linear_pattern():
    """10x5x7 box patterned 3x along X with spacing 15."""
    return load_stl("linear_pattern_3x"), load_kernel_props("linear_pattern_3x")


@pytest.fixture
def linear_pattern_2d():
    """10x5x7 box patterned in 2x3 grid (X spacing 15, Y spacing 8)."""
    return load_stl("linear_pattern_2d"), load_kernel_props("linear_pattern_2d")


@pytest.fixture
def circular_pattern():
    """2x2x5 box patterned 4x around Z axis at 90 degree intervals."""
    return load_stl("circular_pattern_4x"), load_kernel_props("circular_pattern_4x")


@pytest.fixture
def loft():
    """Loft between 4x4 square at z=0 and 2x2 square at z=10 (tapered prism)."""
    return load_stl("loft_taper"), load_kernel_props("loft_taper")


@pytest.fixture
def loft_3section():
    """Loft between 4x4, 3x3, 2x2 squares at z=0, 5, 10."""
    return load_stl("loft_3section"), load_kernel_props("loft_3section")


@pytest.fixture
def boolean_intersect():
    """Intersection of two 10x10x10 boxes offset by (5,5,0). Result: 5x5x10."""
    return load_stl("boolean_intersect"), load_kernel_props("boolean_intersect")


@pytest.fixture
def boolean_subtract():
    """10x5x7 box with a 4x3x10 box subtracted at offset (3,1,-1)."""
    return load_stl("boolean_subtract"), load_kernel_props("boolean_subtract")


@pytest.fixture
def boolean_union():
    """Union of two 10x5x7 boxes, second offset by 5 in X."""
    return load_stl("boolean_union"), load_kernel_props("boolean_union")


@pytest.fixture
def compound_fillet_shell():
    """10x5x7 box with r=1 fillet on one edge, then shelled (t=0.5, top removed)."""
    return load_stl("compound_fillet_shell"), load_kernel_props("compound_fillet_shell")


@pytest.fixture
def fillet_multi():
    """10x5x7 box with r=1 fillet on three edges."""
    return load_stl("box_fillet_multi"), load_kernel_props("box_fillet_multi")


@pytest.fixture
def extrude_draft():
    """10x5 rectangle extruded 7mm with 5-degree draft angle."""
    return load_stl("box_extrude_draft"), load_kernel_props("box_extrude_draft")


@pytest.fixture
def symmetric_extrude():
    """10x5 rectangle extruded 7mm symmetrically (centered on z=0)."""
    return load_stl("box_symmetric"), load_kernel_props("box_symmetric")


@pytest.fixture
def through_hole():
    """10x5x7 box with 4x2 through-hole along Z."""
    return load_stl("box_through_hole"), load_kernel_props("box_through_hole")


@pytest.fixture
def revolve_shell():
    """Full 360-degree revolve of rectangle around Y-axis, then shelled (t=0.5, 1 face removed)."""
    return load_stl("revolve_shell"), load_kernel_props("revolve_shell")


@pytest.fixture
def stress_revolve_fillet():
    """Full 360-degree revolve of rectangle around Y-axis, then fillet(edge 0, r=0.5)."""
    return load_stl("stress_revolve_fillet"), load_kernel_props("stress_revolve_fillet")


@pytest.fixture
def l_shape():
    """L-shaped profile (10x10 minus 5x5 corner) extruded 5mm."""
    return load_stl("l_shape_extrude"), load_kernel_props("l_shape_extrude")


@pytest.fixture
def stress_box_fillet_chamfer():
    """10x5x7 box with r=1 fillet on edge 0, then d=0.5 chamfer on edge 4."""
    return load_stl("stress_box_fillet_chamfer"), load_kernel_props("stress_box_fillet_chamfer")


@pytest.fixture
def stress_box_shell_draft():
    """10x5x7 box -> Shell(top removed, t=0.5) -> Draft(2 side faces, 5 deg)."""
    return load_stl("stress_box_shell_draft"), load_kernel_props("stress_box_shell_draft")


@pytest.fixture
def stress_box_mirror_fillet():
    """10x5x7 box mirrored across YZ plane, then fillet(edge 0, r=0.5)."""
    return load_stl("stress_box_mirror_fillet"), load_kernel_props("stress_box_mirror_fillet")


@pytest.fixture
def stress_box_pattern_shell():
    """10x5x7 box patterned 3x along X (spacing 15), then shelled (top removed, t=0.3)."""
    return load_stl("stress_box_pattern_shell"), load_kernel_props("stress_box_pattern_shell")


@pytest.fixture
def stress_loft_mirror():
    """Loft (4x4 -> 2x2, z=0..10) mirrored across XY plane at z=0."""
    return load_stl("stress_loft_mirror"), load_kernel_props("stress_loft_mirror")


@pytest.fixture
def stress_sweep_pattern():
    """4x4 square swept 10 units along Z, then circular pattern 3x at 120 deg around Z (axis at x=-15)."""
    return load_stl("stress_sweep_pattern"), load_kernel_props("stress_sweep_pattern")


@pytest.fixture
def stress_box_cut_fillet():
    """10x5x7 box with 4x2x3 blind pocket, then r=0.3 fillet on outer box edge."""
    return load_stl("stress_box_cut_fillet"), load_kernel_props("stress_box_cut_fillet")


@pytest.fixture
def stress_box_boolean_shell():
    """Union of two 10x5x7 boxes (offset 5 in X), then shelled (t=0.5, top removed)."""
    return load_stl("stress_box_boolean_shell"), load_kernel_props("stress_box_boolean_shell")


@pytest.fixture
def l_shape_shell():
    """L-shaped profile (10x10 minus 5x5 corner) extruded 5mm, shelled t=0.5, top removed."""
    return load_stl("l_shape_shell"), load_kernel_props("l_shape_shell")


@pytest.fixture
def cylinder_chamfer():
    """Cylinder r=5, h=10 with d=0.5 chamfer on edge 0."""
    return load_stl("cylinder_chamfer_d05"), load_kernel_props("cylinder_chamfer_d05")


@pytest.fixture
def stress_thin_shell():
    """20x20x20 box shelled with t=0.2, top face removed. Very thin walls."""
    return load_stl("stress_thin_shell"), load_kernel_props("stress_thin_shell")


@pytest.fixture
def stress_large_fillet():
    """10x5x7 box with r=2.0 fillet on edge 0. Large radius approaching the 5mm edge."""
    return load_stl("stress_large_fillet"), load_kernel_props("stress_large_fillet")


@pytest.fixture
def stress_asymmetric_chamfer():
    """10x5x7 box with asymmetric chamfer (d1=1.0, d2=0.5) on edge 0."""
    return load_stl("stress_asymmetric_chamfer"), load_kernel_props("stress_asymmetric_chamfer")


@pytest.fixture
def stress_tall_thin_extrude():
    """2x2 rectangle extruded 50mm — high aspect ratio (25:1). Volume = 200."""
    return load_stl("stress_tall_thin_extrude"), load_kernel_props("stress_tall_thin_extrude")


@pytest.fixture
def stress_multi_face_shell():
    """10x5x7 box -> Shell(remove top AND front, t=0.5). Two openings."""
    return load_stl("stress_multi_face_shell"), load_kernel_props("stress_multi_face_shell")


@pytest.fixture
def stress_steep_draft():
    """10x5x7 box with 15-degree draft on two side faces."""
    return load_stl("stress_steep_draft"), load_kernel_props("stress_steep_draft")


@pytest.fixture
def stress_thick_shell():
    """10x10x10 box -> Shell(top removed, t=4.0). Nearly solid — inner cavity 2x2x6."""
    return load_stl("stress_thick_shell"), load_kernel_props("stress_thick_shell")


@pytest.fixture
def stress_flat_extrude():
    """20x20 rectangle extruded 0.5mm — very flat geometry. Volume = 200."""
    return load_stl("stress_flat_extrude"), load_kernel_props("stress_flat_extrude")


@pytest.fixture
def stress_octagon():
    """Regular octagon (8 vertices, circumradius=5) extruded 5mm."""
    return load_stl("stress_octagon"), load_kernel_props("stress_octagon")


@pytest.fixture
def stress_hexagon():
    """Regular hexagon (6 vertices, circumradius=5) extruded 5mm."""
    return load_stl("stress_hexagon"), load_kernel_props("stress_hexagon")


@pytest.fixture
def stress_tiny_fillet():
    """10x5x7 box with tiny fillet r=0.1 on edge 0."""
    return load_stl("stress_tiny_fillet"), load_kernel_props("stress_tiny_fillet")


@pytest.fixture
def stress_high_count_pattern():
    """2x2x2 box patterned 10x along X with spacing 3."""
    return load_stl("stress_high_count_pattern"), load_kernel_props("stress_high_count_pattern")
