use blockcad_kernel::geometry::Vec3;
use blockcad_kernel::operations::extrude::{ExtrudeOp, ExtrudeParams};
use blockcad_kernel::operations::traits::Operation;
use blockcad_kernel::topology::BRep;

#[test]
fn extrude_op_validates_params() {
    let op = ExtrudeOp;
    let params = ExtrudeParams {
        direction: Vec3::new(0.0, 0.0, 1.0),
        depth: -1.0,
        symmetric: false,
        draft_angle: 0.0,
    };
    let brep = BRep::new();
    let result = op.execute(&params, &brep);
    assert!(result.is_err());
}

#[test]
fn extrude_op_name() {
    let op = ExtrudeOp;
    assert_eq!(op.name(), "Extrude");
}

#[test]
fn client_ops_have_unique_names() {
    use blockcad_kernel::operations::revolve::RevolveOp;
    use blockcad_kernel::operations::fillet::FilletOp;
    use blockcad_kernel::operations::chamfer::ChamferOp;

    let names: Vec<&str> = vec![
        ExtrudeOp.name(),
        RevolveOp.name(),
        FilletOp.name(),
        ChamferOp.name(),
    ];

    for name in &names {
        assert!(!name.is_empty());
    }
    let mut unique = names.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(names.len(), unique.len());
}

#[cfg(feature = "server")]
#[test]
fn server_ops_have_unique_names() {
    use blockcad_kernel::operations::sweep::SweepOp;
    use blockcad_kernel::operations::loft::LoftOp;
    use blockcad_kernel::operations::draft::DraftOp;

    let names: Vec<&str> = vec![
        SweepOp.name(),
        LoftOp.name(),
        DraftOp.name(),
    ];

    for name in &names {
        assert!(!name.is_empty());
    }
    let mut unique = names.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(names.len(), unique.len());
}
