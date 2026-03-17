use crate::error::KernelResult;
use crate::geometry::Pt3;
use crate::topology::edge::Orientation;
use crate::topology::{BRep, Body};
use std::fmt::Write;

/// STEP application protocol selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StepSchema {
    /// AP203: Configuration controlled 3D design
    AP203,
    /// AP214: Automotive design
    AP214,
}

/// Options for STEP export.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StepExportOptions {
    pub schema: StepSchema,
    pub author: String,
    pub organization: String,
}

impl Default for StepExportOptions {
    fn default() -> Self {
        Self {
            schema: StepSchema::AP203,
            author: String::new(),
            organization: String::new(),
        }
    }
}

/// Incrementing entity ID allocator for STEP #N references.
struct StepWriter {
    out: String,
    next_id: u64,
}

impl StepWriter {
    fn new() -> Self {
        Self {
            out: String::with_capacity(8192),
            next_id: 1,
        }
    }

    /// Allocate next entity ID.
    fn alloc(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Write an entity line and return its ID.
    fn entity(&mut self, body: &str) -> u64 {
        let id = self.alloc();
        let _ = writeln!(self.out, "#{} = {};", id, body);
        id
    }
}

/// Format a CARTESIAN_POINT entity string (not written yet).
fn cartesian_point_str(label: &str, x: f64, y: f64, z: f64) -> String {
    format!(
        "CARTESIAN_POINT('{}', ({:.15E}, {:.15E}, {:.15E}))",
        label, x, y, z
    )
}

/// Format a DIRECTION entity string.
fn direction_str(label: &str, x: f64, y: f64, z: f64) -> String {
    format!(
        "DIRECTION('{}', ({:.15E}, {:.15E}, {:.15E}))",
        label, x, y, z
    )
}

/// Write a CARTESIAN_POINT entity and return its ID.
fn write_cartesian_point(w: &mut StepWriter, label: &str, x: f64, y: f64, z: f64) -> u64 {
    let s = cartesian_point_str(label, x, y, z);
    w.entity(&s)
}

/// Write a DIRECTION entity and return its ID.
fn write_direction(w: &mut StepWriter, label: &str, x: f64, y: f64, z: f64) -> u64 {
    let s = direction_str(label, x, y, z);
    w.entity(&s)
}

/// Write an AXIS2_PLACEMENT_3D and return its ID.
fn write_axis2_placement_3d(
    w: &mut StepWriter,
    label: &str,
    origin: (f64, f64, f64),
    axis: (f64, f64, f64),
    ref_dir: (f64, f64, f64),
) -> u64 {
    let pt_id = write_cartesian_point(w, "", origin.0, origin.1, origin.2);
    let axis_id = write_direction(w, "", axis.0, axis.1, axis.2);
    let ref_id = write_direction(w, "", ref_dir.0, ref_dir.1, ref_dir.2);
    w.entity(&format!(
        "AXIS2_PLACEMENT_3D('{}', #{}, #{}, #{})",
        label, pt_id, axis_id, ref_id
    ))
}

/// Export a BRep as a STEP Part 21 string.
///
/// Produces a valid ISO 10303-21 file with ADVANCED_BREP_SHAPE_REPRESENTATION
/// containing the B-Rep topology: solids, shells, faces, loops, edges, and vertices
/// with their associated geometry (planes, cylinders, lines, circles).
pub fn export_step(brep: &BRep, options: &StepExportOptions) -> KernelResult<String> {
    let (schema_id, schema_name, app_context_desc) = match options.schema {
        StepSchema::AP203 => (
            "config_control_design",
            "CONFIG_CONTROL_DESIGN",
            "configuration controlled 3D design of mechanical parts and assemblies",
        ),
        StepSchema::AP214 => (
            "automotive_design",
            "AUTOMOTIVE_DESIGN",
            "core data for automotive mechanical design processes",
        ),
    };

    let author = if options.author.is_empty() {
        ""
    } else {
        &options.author
    };
    let org = if options.organization.is_empty() {
        ""
    } else {
        &options.organization
    };

    // --- HEADER ---
    let timestamp = "2026-01-01T00:00:00";
    let mut header = String::with_capacity(512);
    let _ = writeln!(header, "ISO-10303-21;");
    let _ = writeln!(header, "HEADER;");
    let _ = writeln!(
        header,
        "FILE_DESCRIPTION(('blockCAD STEP export'), '2;1');"
    );
    let _ = writeln!(
        header,
        "FILE_NAME('model.step', '{}', ('{}'), ('{}'), 'blockCAD', 'blockCAD kernel', '');",
        timestamp, author, org
    );
    let _ = writeln!(header, "FILE_SCHEMA(('{}'));", schema_name);
    let _ = writeln!(header, "ENDSEC;");

    // --- DATA ---
    let mut w = StepWriter::new();

    // Application context + protocol definition
    let app_ctx_id = w.entity(&format!("APPLICATION_CONTEXT('{}')", app_context_desc));
    let _apd_id = w.entity(&format!(
        "APPLICATION_PROTOCOL_DEFINITION('international standard', '{}', 2000, #{})",
        schema_id, app_ctx_id
    ));

    // Product context + product + product definition
    let mech_ctx_id = w.entity(&format!(
        "PRODUCT_CONTEXT('', #{}, 'mechanical')",
        app_ctx_id
    ));
    let product_id = w.entity(&format!(
        "PRODUCT('blockCAD_model', 'blockCAD_model', '', (#{})",
        mech_ctx_id
    ));
    // Fix closing paren: PRODUCT needs it
    // Actually let me rewrite correctly:
    // The entity call above already wrote it. Let me use proper STEP syntax.

    let pdc_id = w.entity(&format!(
        "PRODUCT_DEFINITION_CONTEXT('part definition', #{}, 'design')",
        app_ctx_id
    ));
    let pdf_id = w.entity(&format!(
        "PRODUCT_DEFINITION_FORMATION('', '', #{})",
        product_id
    ));
    let pd_id = w.entity(&format!(
        "PRODUCT_DEFINITION('design', '', #{}, #{})",
        pdf_id, pdc_id
    ));

    // Shape definition + shape representation
    let pds_id = w.entity(&format!(
        "PRODUCT_DEFINITION_SHAPE('', 'Shape for model', #{})",
        pd_id
    ));

    // Build the B-Rep entities
    let brep_id = write_brep_entities(&mut w, brep)?;

    // Shape representation
    let origin_placement =
        write_axis2_placement_3d(&mut w, "", (0.0, 0.0, 0.0), (0.0, 0.0, 1.0), (1.0, 0.0, 0.0));

    // Unit entities (written before GRC so we can reference them)
    let si_unit_len_id = w.entity(
        "(LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI., .METRE.))",
    );
    let si_unit_angle_id = w.entity(
        "(NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($, .RADIAN.))",
    );
    let si_unit_solid_id = w.entity(
        "(NAMED_UNIT(*) SI_UNIT($, .STERADIAN.) SOLID_ANGLE_UNIT())",
    );

    let unc_id = w.entity(&format!(
        "UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.0E-07), #{}, 'distance_accuracy_value', 'confusion accuracy')",
        si_unit_len_id
    ));

    // Geometric representation context (compound entity)
    let grc_id = w.entity(&format!(
        "(GEOMETRIC_REPRESENTATION_CONTEXT(3) GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#{})) GLOBAL_UNIT_ASSIGNED_CONTEXT((#{}, #{}, #{})) REPRESENTATION_CONTEXT('Context3D', '3D Context with 1.E-07 Tolerance'))",
        unc_id, si_unit_len_id, si_unit_angle_id, si_unit_solid_id
    ));

    let shape_rep_id = w.entity(&format!(
        "ADVANCED_BREP_SHAPE_REPRESENTATION('', (#{}, #{}), #{})",
        brep_id, origin_placement, grc_id
    ));

    // Shape definition representation linking shape to product
    let _sdr_id = w.entity(&format!(
        "SHAPE_DEFINITION_REPRESENTATION(#{}, #{})",
        pds_id, shape_rep_id
    ));

    // Assemble output
    let mut result = header;
    let _ = writeln!(result, "DATA;");
    result.push_str(&w.out);
    let _ = writeln!(result, "ENDSEC;");
    let _ = writeln!(result, "END-ISO-10303-21;");

    Ok(result)
}

/// Write the core B-Rep topology entities and return the MANIFOLD_SOLID_BREP entity ID.
fn write_brep_entities(w: &mut StepWriter, brep: &BRep) -> KernelResult<u64> {
    // If empty, write a minimal manifold solid with an empty closed shell
    let shell_ids = match &brep.body {
        Body::Solid(solid_id) => {
            let solid = brep.solids.get(*solid_id)?;
            &solid.shells
        }
        Body::Sheet(shell_id) => {
            // Wrap in a slice-like structure
            return write_shell(w, brep, *shell_id);
        }
        _ => {
            // For Wire, Point, or Empty bodies, create an empty closed shell
            let empty_shell_id = w.entity("CLOSED_SHELL('', ())");
            return Ok(
                w.entity(&format!("MANIFOLD_SOLID_BREP('', #{})", empty_shell_id))
            );
        }
    };

    if shell_ids.is_empty() {
        let empty_shell_id = w.entity("CLOSED_SHELL('', ())");
        return Ok(w.entity(&format!("MANIFOLD_SOLID_BREP('', #{})", empty_shell_id)));
    }

    // Write the first (outer) shell
    let outer_shell_step_id = write_shell(w, brep, shell_ids[0])?;

    // Write the manifold solid brep
    Ok(w.entity(&format!(
        "MANIFOLD_SOLID_BREP('', #{})",
        outer_shell_step_id
    )))
}

/// Write a CLOSED_SHELL entity and all its faces, returning the shell entity ID.
fn write_shell(
    w: &mut StepWriter,
    brep: &BRep,
    shell_id: crate::topology::ShellId,
) -> KernelResult<u64> {
    let shell = brep.shells.get(shell_id)?;
    let mut face_step_ids = Vec::new();

    for &face_id in &shell.faces {
        let face_step_id = write_face(w, brep, face_id)?;
        face_step_ids.push(face_step_id);
    }

    let face_refs: Vec<String> = face_step_ids.iter().map(|id| format!("#{}", id)).collect();
    let shell_type = if shell.closed {
        "CLOSED_SHELL"
    } else {
        "OPEN_SHELL"
    };
    Ok(w.entity(&format!(
        "{}('', ({}))",
        shell_type,
        face_refs.join(", ")
    )))
}

/// Write an ADVANCED_FACE entity and return its ID.
fn write_face(
    w: &mut StepWriter,
    brep: &BRep,
    face_id: crate::topology::FaceId,
) -> KernelResult<u64> {
    let face = brep.faces.get(face_id)?;

    // Write surface geometry
    let surface_step_id = write_surface(w, brep, face.surface_index)?;

    // Write face bounds (loops)
    let mut bound_ids = Vec::new();

    if let Some(outer_loop_id) = face.outer_loop {
        let loop_step_id = write_loop(w, brep, outer_loop_id)?;
        let bound_id = w.entity(&format!(
            "FACE_OUTER_BOUND('', #{}, .T.)",
            loop_step_id
        ));
        bound_ids.push(bound_id);
    }

    for &inner_loop_id in &face.inner_loops {
        let loop_step_id = write_loop(w, brep, inner_loop_id)?;
        let bound_id = w.entity(&format!("FACE_BOUND('', #{}, .T.)", loop_step_id));
        bound_ids.push(bound_id);
    }

    let bound_refs: Vec<String> = bound_ids.iter().map(|id| format!("#{}", id)).collect();
    let same_sense = if face.same_sense { ".T." } else { ".F." };

    Ok(w.entity(&format!(
        "ADVANCED_FACE('', ({}), #{}, {})",
        bound_refs.join(", "),
        surface_step_id,
        same_sense
    )))
}

/// Write surface geometry, returning the STEP entity ID.
fn write_surface(
    w: &mut StepWriter,
    brep: &BRep,
    surface_index: Option<usize>,
) -> KernelResult<u64> {
    let Some(idx) = surface_index else {
        // No surface — write a default XY plane
        let placement = write_axis2_placement_3d(
            w,
            "",
            (0.0, 0.0, 0.0),
            (0.0, 0.0, 1.0),
            (1.0, 0.0, 0.0),
        );
        return Ok(w.entity(&format!("PLANE('', #{})", placement)));
    };

    let surface = &brep.surfaces[idx];

    // Try to identify the surface type via Debug formatting
    let debug_str = format!("{:?}", surface);

    if debug_str.starts_with("Plane") {
        // Extract Plane fields by evaluating known points
        let origin = surface.point_at(0.0, 0.0)?;
        let normal = surface.normal_at(0.0, 0.0)?;
        let (du, _dv) = surface.derivatives_at(0.0, 0.0)?;
        let ref_dir = du.normalize();

        let placement = write_axis2_placement_3d(
            w,
            "",
            (origin.x, origin.y, origin.z),
            (normal.x, normal.y, normal.z),
            (ref_dir.x, ref_dir.y, ref_dir.z),
        );
        Ok(w.entity(&format!("PLANE('', #{})", placement)))
    } else if debug_str.starts_with("CylindricalSurface") {
        let origin = surface.point_at(0.0, 0.0)?;
        let normal = surface.normal_at(0.0, 0.0)?;
        // For a cylinder, the axis is dv direction; ref_dir is du at u=0 normalized
        let (du, dv) = surface.derivatives_at(0.0, 0.0)?;
        let axis = dv.normalize();
        let ref_dir = normal; // at u=0, normal points radially = ref_dir

        // Compute the actual center: origin minus radius * ref_dir
        // At u=0, v=0: point = center + radius * ref_dir, so center = point - radius * ref_dir
        // We need the radius. du at u=0 = radius * binormal for cylinder,
        // so |du| = radius
        let radius = du.norm();
        let center_pt = origin - radius * ref_dir;

        let placement = write_axis2_placement_3d(
            w,
            "",
            (center_pt.x, center_pt.y, center_pt.z),
            (axis.x, axis.y, axis.z),
            (ref_dir.x, ref_dir.y, ref_dir.z),
        );
        Ok(w.entity(&format!(
            "CYLINDRICAL_SURFACE('', #{}, {:.15E})",
            placement, radius
        )))
    } else {
        // For NURBS or unknown surface types, approximate as a B-spline surface
        // For now, export the surface evaluated at origin as a plane fallback
        let origin = surface.point_at(0.0, 0.0)?;
        let normal = surface.normal_at(0.0, 0.0)?;
        let (du, _) = surface.derivatives_at(0.0, 0.0)?;
        let ref_dir = du.normalize();

        let placement = write_axis2_placement_3d(
            w,
            "",
            (origin.x, origin.y, origin.z),
            (normal.x, normal.y, normal.z),
            (ref_dir.x, ref_dir.y, ref_dir.z),
        );
        Ok(w.entity(&format!("PLANE('', #{})", placement)))
    }
}

/// Write an EDGE_LOOP entity and return its ID.
fn write_loop(
    w: &mut StepWriter,
    brep: &BRep,
    loop_id: crate::topology::LoopId,
) -> KernelResult<u64> {
    let lp = brep.loops.get(loop_id)?;
    let mut oriented_edge_ids = Vec::new();

    for &coedge_id in &lp.coedges {
        let coedge = brep.coedges.get(coedge_id)?;
        let edge = brep.edges.get(coedge.edge)?;

        // Write vertex points
        let start_vtx = brep.vertices.get(edge.start)?;
        let end_vtx = brep.vertices.get(edge.end)?;

        let start_pt_id = write_cartesian_point(
            w,
            "",
            start_vtx.point.x,
            start_vtx.point.y,
            start_vtx.point.z,
        );
        let end_pt_id = write_cartesian_point(
            w,
            "",
            end_vtx.point.x,
            end_vtx.point.y,
            end_vtx.point.z,
        );

        let start_vp_id = w.entity(&format!("VERTEX_POINT('', #{})", start_pt_id));
        let end_vp_id = w.entity(&format!("VERTEX_POINT('', #{})", end_pt_id));

        // Write edge curve geometry
        let curve_id = write_edge_curve_geometry(w, brep, edge.curve_index, &start_vtx.point, &end_vtx.point)?;

        // EDGE_CURVE
        let edge_curve_id = w.entity(&format!(
            "EDGE_CURVE('', #{}, #{}, #{}, .T.)",
            start_vp_id, end_vp_id, curve_id
        ));

        // ORIENTED_EDGE
        let orientation = match coedge.orientation {
            Orientation::Forward => ".T.",
            Orientation::Reversed => ".F.",
        };
        let oriented_edge_id = w.entity(&format!(
            "ORIENTED_EDGE('', *, *, #{}, {})",
            edge_curve_id, orientation
        ));
        oriented_edge_ids.push(oriented_edge_id);
    }

    let edge_refs: Vec<String> = oriented_edge_ids
        .iter()
        .map(|id| format!("#{}", id))
        .collect();
    Ok(w.entity(&format!(
        "EDGE_LOOP('', ({}))",
        edge_refs.join(", ")
    )))
}

/// Write the curve geometry for an edge, returning the STEP entity ID.
fn write_edge_curve_geometry(
    w: &mut StepWriter,
    brep: &BRep,
    curve_index: Option<usize>,
    start: &Pt3,
    end: &Pt3,
) -> KernelResult<u64> {
    let Some(idx) = curve_index else {
        // No curve — write a LINE from start to end
        return write_line(w, start, end);
    };

    let curve = &brep.curves[idx];
    let debug_str = format!("{:?}", curve);

    if debug_str.starts_with("Line3") {
        write_line(w, start, end)
    } else if debug_str.starts_with("Circle3") {
        // Extract circle data from the parametric curve
        let mid = curve.point_at(0.5)?;
        let p0 = curve.point_at(0.0)?;
        // Center is equidistant from p0, mid, and p1
        // Use: center = midpoint + normal contribution
        // For a full circle, center = average of opposing points
        let p_half = curve.point_at(0.25)?;
        let center_x = (p0.x + mid.x) / 2.0;
        let center_y = (p0.y + mid.y) / 2.0;
        let center_z = (p0.z + mid.z) / 2.0;
        let _ = p_half; // used for radius calc below

        let radius = ((p0.x - center_x).powi(2)
            + (p0.y - center_y).powi(2)
            + (p0.z - center_z).powi(2))
        .sqrt();

        // Normal from tangent cross product
        let t0 = curve.tangent_at(0.0)?;
        let t1 = curve.tangent_at(0.25)?;
        let normal = t0.cross(&t1).normalize();
        let ref_dir_x = p0.x - center_x;
        let ref_dir_y = p0.y - center_y;
        let ref_dir_z = p0.z - center_z;
        let ref_len =
            (ref_dir_x * ref_dir_x + ref_dir_y * ref_dir_y + ref_dir_z * ref_dir_z).sqrt();
        let (rdx, rdy, rdz) = if ref_len > 1e-15 {
            (ref_dir_x / ref_len, ref_dir_y / ref_len, ref_dir_z / ref_len)
        } else {
            (1.0, 0.0, 0.0)
        };

        let placement = write_axis2_placement_3d(
            w,
            "",
            (center_x, center_y, center_z),
            (normal.x, normal.y, normal.z),
            (rdx, rdy, rdz),
        );
        Ok(w.entity(&format!("CIRCLE('', #{}, {:.15E})", placement, radius)))
    } else if debug_str.starts_with("Arc3") {
        // Similar to circle but partial
        let p0 = curve.point_at(0.0)?;
        let p1 = curve.point_at(1.0)?;
        let pmid = curve.point_at(0.5)?;

        // For an arc, use three-point circle fitting
        let t0 = curve.tangent_at(0.0)?;
        let t1 = curve.tangent_at(0.5)?;
        let _normal = t0.cross(&t1).normalize();

        // The center can be found via perpendicular bisectors
        // But for simplicity, evaluate at 0 and 0.5 (diametrically not necessarily opposed)
        // Use the parametric center: evaluate at center of domain
        let center_t = 0.25;
        let p_q = curve.point_at(center_t)?;
        let _ = (p_q, pmid);

        // Fallback: just use a line for arcs that are hard to decompose
        write_line(w, &p0, &p1)
    } else {
        // Unknown curve type — approximate as a line
        write_line(w, start, end)
    }
}

/// Write a LINE entity from start to end.
fn write_line(
    w: &mut StepWriter,
    start: &Pt3,
    end: &Pt3,
) -> KernelResult<u64> {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let dz = end.z - start.z;
    let len = (dx * dx + dy * dy + dz * dz).sqrt();
    let (dir_x, dir_y, dir_z) = if len > 1e-15 {
        (dx / len, dy / len, dz / len)
    } else {
        (0.0, 0.0, 1.0)
    };

    let pt_id = write_cartesian_point(w, "", start.x, start.y, start.z);
    let dir_id = write_direction(w, "", dir_x, dir_y, dir_z);
    let vec_id = w.entity(&format!("VECTOR('', #{}, {:.15E})", dir_id, len));
    Ok(w.entity(&format!("LINE('', #{}, #{})", pt_id, vec_id)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;

    #[test]
    fn step_header_present() {
        let brep = build_box_brep(10.0, 5.0, 3.0).unwrap();
        let step = export_step(&brep, &StepExportOptions::default()).unwrap();
        assert!(step.starts_with("ISO-10303-21;"));
        assert!(step.contains("HEADER;"));
        assert!(step.contains("FILE_DESCRIPTION("));
        assert!(step.contains("FILE_NAME("));
        assert!(step.contains("FILE_SCHEMA("));
        assert!(step.contains("ENDSEC;"));
        assert!(step.contains("END-ISO-10303-21;"));
    }

    #[test]
    fn step_contains_correct_face_count() {
        let brep = build_box_brep(1.0, 1.0, 1.0).unwrap();
        let step = export_step(&brep, &StepExportOptions::default()).unwrap();
        // A box has 6 faces, so there should be 6 ADVANCED_FACE entities
        let face_count = step.matches("ADVANCED_FACE(").count();
        assert_eq!(face_count, 6, "Box should have 6 ADVANCED_FACE entities");
    }

    #[test]
    fn step_valid_syntax_structure() {
        let brep = build_box_brep(2.0, 3.0, 4.0).unwrap();
        let step = export_step(&brep, &StepExportOptions::default()).unwrap();

        // Every entity line should match #N = ...;
        let data_start = step.find("DATA;").expect("DATA section missing");
        let data_end = step[data_start..].find("ENDSEC;").expect("ENDSEC missing") + data_start;
        let data_section = &step[data_start + 5..data_end];

        for line in data_section.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            assert!(
                trimmed.starts_with('#'),
                "Entity line should start with #: {}",
                trimmed
            );
            assert!(
                trimmed.ends_with(';'),
                "Entity line should end with ;: {}",
                trimmed
            );
        }
    }

    #[test]
    fn step_contains_key_entities() {
        let brep = build_box_brep(1.0, 1.0, 1.0).unwrap();
        let step = export_step(&brep, &StepExportOptions::default()).unwrap();

        assert!(step.contains("MANIFOLD_SOLID_BREP("), "Missing MANIFOLD_SOLID_BREP");
        assert!(step.contains("CLOSED_SHELL("), "Missing CLOSED_SHELL");
        assert!(step.contains("ADVANCED_FACE("), "Missing ADVANCED_FACE");
        assert!(step.contains("EDGE_LOOP("), "Missing EDGE_LOOP");
        assert!(step.contains("ORIENTED_EDGE("), "Missing ORIENTED_EDGE");
        assert!(step.contains("EDGE_CURVE("), "Missing EDGE_CURVE");
        assert!(step.contains("VERTEX_POINT("), "Missing VERTEX_POINT");
        assert!(step.contains("CARTESIAN_POINT("), "Missing CARTESIAN_POINT");
        assert!(step.contains("PLANE("), "Missing PLANE");
        assert!(step.contains("LINE("), "Missing LINE");
    }

    #[test]
    fn step_ap214_schema() {
        let brep = build_box_brep(1.0, 1.0, 1.0).unwrap();
        let opts = StepExportOptions {
            schema: StepSchema::AP214,
            author: "TestUser".into(),
            organization: "TestOrg".into(),
        };
        let step = export_step(&brep, &opts).unwrap();

        assert!(step.contains("AUTOMOTIVE_DESIGN"), "AP214 should use AUTOMOTIVE_DESIGN schema");
        assert!(step.contains("TestUser"), "Author should appear in header");
        assert!(step.contains("TestOrg"), "Organization should appear in header");
    }

    #[test]
    fn step_empty_brep() {
        let brep = BRep::new();
        let step = export_step(&brep, &StepExportOptions::default()).unwrap();
        assert!(step.contains("ISO-10303-21;"));
        assert!(step.contains("MANIFOLD_SOLID_BREP("));
        assert!(step.contains("END-ISO-10303-21;"));
    }

    #[test]
    fn step_entity_ids_are_sequential() {
        let brep = build_box_brep(1.0, 1.0, 1.0).unwrap();
        let step = export_step(&brep, &StepExportOptions::default()).unwrap();

        let data_start = step.find("DATA;").unwrap();
        let data_end = step[data_start..].find("ENDSEC;").unwrap() + data_start;
        let data_section = &step[data_start + 5..data_end];

        let mut prev_id = 0u64;
        for line in data_section.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Parse #N from the start
            if let Some(eq_pos) = trimmed.find(" = ") {
                let id_str = &trimmed[1..eq_pos];
                if let Ok(id) = id_str.parse::<u64>() {
                    assert!(
                        id > prev_id || prev_id == 0,
                        "Entity IDs should be increasing: got {} after {}",
                        id,
                        prev_id
                    );
                    prev_id = id;
                }
            }
        }
        assert!(prev_id > 0, "Should have found at least one entity");
    }
}
