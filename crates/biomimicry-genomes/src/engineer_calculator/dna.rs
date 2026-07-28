//! Engineering-calculator DNA — Sensor → Gate → Reducer triad per operator.

use biomimicry::genesis::{
    Cistron, DimensionVector, EndpointPolarity, Grn, Primitive, PrimitiveNode, endpoint,
};
use biomimicry::signal::Scope;

use crate::engineer_calculator::kinds::{
    OPERAND_A, OPERAND_B, OP_ADD, OP_MUL, RESULT, SCHEMA_STAMP, VALUE,
};

/// Gene regulatory network for the engineering calculator.
///
/// Families: Sensor (per op), Gate, Reducer (per op), Bound, Boundary, plus a
/// schema stamp. `+` and `×` share the same triad shape — only role labels differ.
#[must_use]
pub fn calculator_dna() -> Grn {
    let mut g = Grn::new();
    let shared = SharedPoles::new();
    let add = TriadPoles::new(10);
    let mul = TriadPoles::new(20);

    for n in shared
        .nodes()
        .into_iter()
        .chain(add.nodes())
        .chain(mul.nodes())
    {
        g.add_node(n).expect("add node");
    }

    add_shared_families(&mut g, &shared);
    add.add_to(&mut g, "add", OP_ADD);
    mul.add_to(&mut g, "mul", OP_MUL);
    g
}

struct SharedPoles {
    gate_a: PrimitiveNode,
    gate_b: PrimitiveNode,
    gate_expr: PrimitiveNode,
    bound: PrimitiveNode,
    boundary: PrimitiveNode,
    stamp: PrimitiveNode,
}

impl SharedPoles {
    fn new() -> Self {
        Self {
            gate_a: PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 0])),
            gate_b: PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, 1])),
            gate_expr: PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, 0])),
            bound: PrimitiveNode::new(Primitive::Transduction, DimensionVector::new([20, 0])),
            boundary: PrimitiveNode::new(Primitive::Signal, DimensionVector::new([30, 0])),
            stamp: PrimitiveNode::new(Primitive::Expression, DimensionVector::new([40, 0])),
        }
    }

    fn nodes(&self) -> [PrimitiveNode; 6] {
        [
            self.gate_a.clone(),
            self.gate_b.clone(),
            self.gate_expr.clone(),
            self.bound.clone(),
            self.boundary.clone(),
            self.stamp.clone(),
        ]
    }
}

struct TriadPoles {
    sensor_r: PrimitiveNode,
    sensor_e: PrimitiveNode,
    red_r: PrimitiveNode,
    red_e: PrimitiveNode,
    red_t: PrimitiveNode,
    red_s: PrimitiveNode,
}

impl TriadPoles {
    fn new(row: i32) -> Self {
        Self {
            sensor_r: PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, row])),
            sensor_e: PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, row])),
            red_r: PrimitiveNode::new(Primitive::Receptor, DimensionVector::new([0, row + 1])),
            red_e: PrimitiveNode::new(Primitive::Expression, DimensionVector::new([2, row + 1])),
            red_t: PrimitiveNode::new(Primitive::Transduction, DimensionVector::new([6, row + 1])),
            red_s: PrimitiveNode::new(Primitive::Signal, DimensionVector::new([4, row + 1])),
        }
    }

    fn nodes(&self) -> [PrimitiveNode; 6] {
        [
            self.sensor_r.clone(),
            self.sensor_e.clone(),
            self.red_r.clone(),
            self.red_e.clone(),
            self.red_t.clone(),
            self.red_s.clone(),
        ]
    }

    fn add_to(&self, g: &mut Grn, op_label: &str, op_kind: &str) {
        g.add_cistron(Cistron::new(
            format!("sensor_{op_label}"),
            vec![
                endpoint(&self.sensor_r, EndpointPolarity::Positive, op_kind, None),
                endpoint(&self.sensor_e, EndpointPolarity::Positive, "activate", None),
            ],
        ));
        g.add_cistron(Cistron::new(
            format!("reducer_{op_label}"),
            vec![
                endpoint(&self.red_r, EndpointPolarity::Positive, op_kind, None),
                endpoint(&self.red_e, EndpointPolarity::Positive, "fold", None),
                endpoint(&self.red_t, EndpointPolarity::Positive, "produce", None),
                endpoint(
                    &self.red_s,
                    EndpointPolarity::Positive,
                    VALUE,
                    Some(Scope::Neighbors),
                ),
            ],
        ));
    }
}

fn add_shared_families(g: &mut Grn, poles: &SharedPoles) {
    g.add_cistron(Cistron::new(
        "genome_stamp",
        vec![
            endpoint(&poles.stamp, EndpointPolarity::Positive, SCHEMA_STAMP, None),
            endpoint(&poles.stamp, EndpointPolarity::Negative, SCHEMA_STAMP, None),
        ],
    ));
    g.add_cistron(Cistron::new(
        "gate",
        vec![
            endpoint(&poles.gate_a, EndpointPolarity::Positive, OPERAND_A, None),
            endpoint(&poles.gate_b, EndpointPolarity::Positive, OPERAND_B, None),
            endpoint(&poles.gate_expr, EndpointPolarity::Positive, "ready", None),
        ],
    ));
    g.add_cistron(Cistron::new(
        "bound",
        vec![
            endpoint(&poles.bound, EndpointPolarity::Positive, "x", None),
            endpoint(&poles.bound, EndpointPolarity::Negative, "x", None),
        ],
    ));
    g.add_cistron(Cistron::new(
        "boundary",
        vec![endpoint(
            &poles.boundary,
            EndpointPolarity::Positive,
            RESULT,
            Some(Scope::Systemwide),
        )],
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use biomimicry::genesis::{GeneOrigin, compile};

    #[test]
    fn g1_calculator_dna_compiles_with_expected_gene_count() {
        let dna = calculator_dna();
        let genome = compile(&dna).expect("compile calculator_dna");

        // Traversed: stamp, gate, bound, boundary, sensor_add, reducer_add,
        // sensor_mul, reducer_mul = 8.
        // Self-complements: stamp + bound = 2 → complements for the other 6 → 8+6=14.
        assert_eq!(dna.cistron_count(), 8);
        assert_eq!(genome.len(), 14);

        let traversed = genome
            .iter()
            .filter(|g| matches!(g.origin, GeneOrigin::Traversed))
            .count();
        assert_eq!(traversed, 8);

        for kind in [
            "genome_stamp",
            "gate",
            "bound",
            "boundary",
            "sensor_add",
            "reducer_add",
            "sensor_mul",
            "reducer_mul",
        ] {
            assert!(
                genome.genes_of_kind(kind).next().is_some(),
                "missing kind {kind}"
            );
        }
    }
}
