//! AEC wall-move DNA — matched SignalKind surfaces only.

use biomimicry::genesis::{
    Cistron, DimensionVector, EndpointPolarity, Grn, Primitive, PrimitiveNode, endpoint,
};

use crate::kinds::WALL_MOVE;

/// Gene regulatory network for the wall-move reflex path (+ thin effector).
#[must_use]
pub fn aec_dna() -> Grn {
    let mut g = Grn::new();

    let receptor = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 0]));
    let expr = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, 0]));
    let transduction = PrimitiveNode::new(Primitive::Transduction, DimensionVector::new([6, 0]));
    let r2 = PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 10]));
    let e2 = PrimitiveNode::new(Primitive::Expression, DimensionVector::new([1, 10]));

    for n in [
        receptor.clone(),
        expr.clone(),
        transduction.clone(),
        r2.clone(),
        e2.clone(),
    ] {
        g.add_node(n).expect("add node");
    }

    g.add_cistron(Cistron::new(
        "wall_move_path",
        vec![
            endpoint(&receptor, EndpointPolarity::Positive, WALL_MOVE, None),
            endpoint(&expr, EndpointPolarity::Positive, "activate", None),
            endpoint(&transduction, EndpointPolarity::Positive, "produce", None),
        ],
    ));

    g.add_cistron(Cistron::new(
        "aec_effector",
        vec![
            endpoint(&r2, EndpointPolarity::Positive, "gate", None),
            endpoint(&e2, EndpointPolarity::Positive, "out", None),
        ],
    ));

    g
}
