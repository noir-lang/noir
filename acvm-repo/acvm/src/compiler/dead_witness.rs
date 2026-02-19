use acir::{
    AcirField,
    circuit::{Circuit, Opcode, brillig::BrilligOutputs, opcodes::BlockId},
    native_types::{Expression, Witness},
};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use super::simulator::CircuitSimulator;

/// Find witnesses in a circuit that are "dead" — not transitively connected
/// to any public input or return value through the constraint graph.
///
/// A dead witness can take any value without violating any circuit constraint,
/// which is a soundness concern.
///
/// The algorithm models the circuit as an undirected graph where nodes are
/// witnesses and edges connect witnesses that share a constraint. BrilligCall
/// opcodes do not create constraint edges (they are unconstrained execution).
/// BFS from seed witnesses (public parameters ∪ return values) determines
/// reachability; unreachable witnesses are dead.
pub fn find_dead_witnesses<F: AcirField + Send + Sync>(circuit: &Circuit<F>) -> HashSet<Witness> {
    let graph = build_graph(circuit);
    find_dead(&graph, circuit)
}

/// Per-opcode extraction result, produced in parallel and then merged.
struct OpcodeEdges {
    /// Witnesses that should be fully connected (share a constraint).
    /// Empty for opcodes that don't create constraint edges (e.g. BrilligCall).
    connected: Vec<Witness>,
    /// New witnesses to register (all witnesses seen in the opcode).
    witnesses: Vec<Witness>,
    /// If this opcode touches a memory block, the block ID and associated witnesses.
    block: Option<(BlockId, Vec<Witness>)>,
}

struct Graph {
    adjacency: HashMap<Witness, HashSet<Witness>>,
    all_witnesses: HashSet<Witness>,
    /// Reverse index: witness -> set of block IDs it belongs to.
    witness_blocks: HashMap<Witness, Vec<BlockId>>,
    /// Forward index: block ID -> all witnesses in that block.
    block_witnesses: HashMap<BlockId, HashSet<Witness>>,
}

/// Extract edges from a single opcode (pure, no shared state).
fn extract_opcode_edges<F: AcirField>(opcode: &Opcode<F>) -> OpcodeEdges {
    match opcode {
        Opcode::AssertZero(expr) => {
            let witnesses: Vec<Witness> = expr_witnesses(expr).collect();
            OpcodeEdges { connected: witnesses.clone(), witnesses, block: None }
        }
        Opcode::BlackBoxFuncCall(black_box_func_call) => {
            let mut witnesses: Vec<Witness> =
                black_box_func_call.get_input_witnesses().into_iter().collect();
            witnesses.extend(black_box_func_call.get_outputs_vec());
            OpcodeEdges { connected: witnesses.clone(), witnesses, block: None }
        }
        Opcode::Call { inputs, outputs, predicate, .. } => {
            let mut witnesses: Vec<Witness> =
                inputs.iter().copied().chain(outputs.iter().copied()).collect();
            witnesses.extend(expr_witnesses(predicate));
            OpcodeEdges { connected: witnesses.clone(), witnesses, block: None }
        }
        Opcode::BrilligCall { outputs, .. } => {
            // BrilligCall does NOT create constraint edges (unconstrained execution).
            // Inputs are already-constrained witnesses, so only outputs are new.
            let mut witnesses = Vec::new();
            for output in outputs {
                match output {
                    BrilligOutputs::Simple(witness) => witnesses.push(*witness),
                    BrilligOutputs::Array(arr) => witnesses.extend(arr.iter().copied()),
                }
            }
            OpcodeEdges { connected: Vec::new(), witnesses, block: None }
        }
        Opcode::MemoryInit { block_id, init, .. } => {
            let witnesses: Vec<Witness> = init.to_vec();
            OpcodeEdges {
                connected: Vec::new(),
                witnesses: witnesses.clone(),
                block: Some((*block_id, witnesses)),
            }
        }
        Opcode::MemoryOp { block_id, op } => {
            let witnesses: Vec<Witness> = expr_witnesses(&op.index)
                .chain(expr_witnesses(&op.value))
                .chain(expr_witnesses(&op.operation))
                .collect();
            OpcodeEdges {
                connected: Vec::new(),
                witnesses: witnesses.clone(),
                block: Some((*block_id, witnesses)),
            }
        }
    }
}

fn build_graph<F: AcirField + Send + Sync>(circuit: &Circuit<F>) -> Graph {
    // Parallel map: extract edges from each opcode independently
    let opcode_edges: Vec<OpcodeEdges> =
        circuit.opcodes.par_iter().map(extract_opcode_edges).collect();

    // Sequential reduce: merge into the graph
    let mut adjacency: HashMap<Witness, HashSet<Witness>> = HashMap::new();
    let mut all_witnesses: HashSet<Witness> = circuit
        .private_parameters
        .iter()
        .chain(circuit.public_parameters.0.iter())
        .chain(circuit.return_values.0.iter())
        .copied()
        .collect();
    let mut block_witnesses: HashMap<BlockId, HashSet<Witness>> = HashMap::new();

    for edges in opcode_edges {
        all_witnesses.extend(edges.witnesses);

        // Add full connectivity for constraint-bearing opcodes
        let connected = &edges.connected;
        for i in 0..connected.len() {
            for j in (i + 1)..connected.len() {
                adjacency.entry(connected[i]).or_default().insert(connected[j]);
                adjacency.entry(connected[j]).or_default().insert(connected[i]);
            }
        }

        // Accumulate block witnesses
        if let Some((block_id, block_ws)) = edges.block {
            block_witnesses.entry(block_id).or_default().extend(block_ws);
        }
    }

    // Build reverse index: witness -> block IDs
    let mut witness_blocks: HashMap<Witness, Vec<BlockId>> = HashMap::new();
    for (block_id, witnesses) in &block_witnesses {
        for witness in witnesses {
            witness_blocks.entry(*witness).or_default().push(*block_id);
        }
    }

    Graph { adjacency, all_witnesses, witness_blocks, block_witnesses }
}

/// BFS from seeds to find all reachable witnesses, return unreachable ones.
fn find_dead<F: AcirField>(graph: &Graph, circuit: &Circuit<F>) -> HashSet<Witness> {
    let mut visited: HashSet<Witness> = HashSet::new();
    let mut queue: VecDeque<Witness> = VecDeque::new();

    // Seeds: public parameters + return values
    for witness in circuit.public_parameters.0.iter().chain(circuit.return_values.0.iter()) {
        if graph.all_witnesses.contains(witness) && visited.insert(*witness) {
            queue.push_back(*witness);
        }
    }

    // BFS
    while let Some(current) = queue.pop_front() {
        // Direct adjacency edges
        if let Some(neighbors) = graph.adjacency.get(&current) {
            for neighbor in neighbors {
                if visited.insert(*neighbor) {
                    queue.push_back(*neighbor);
                }
            }
        }

        // Block-based connectivity via reverse index (O(1) lookup)
        if let Some(block_ids) = graph.witness_blocks.get(&current) {
            for block_id in block_ids {
                if let Some(block_witnesses) = graph.block_witnesses.get(block_id) {
                    for witness in block_witnesses {
                        if visited.insert(*witness) {
                            queue.push_back(*witness);
                        }
                    }
                }
            }
        }
    }

    graph.all_witnesses.difference(&visited).copied().collect()
}

/// Extract all witnesses from an expression.
fn expr_witnesses<F>(expr: &Expression<F>) -> impl Iterator<Item = Witness> + '_ {
    CircuitSimulator::expr_witness(expr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use acir::circuit::Circuit;
    use acir::native_types::Witness;

    /// Helper: parse a circuit from the text format.
    fn parse_circuit(src: &str) -> Circuit<acir::FieldElement> {
        Circuit::from_str(src).unwrap()
    }

    #[test]
    fn all_live_simple() {
        // w0 is public, w1 is connected to w0 via AssertZero, w1 is a return value
        let circuit = parse_circuit(
            "
            private parameters: []
            public parameters: [w0]
            return values: [w1]
            ASSERT w1 = w0
            ",
        );
        let dead = find_dead_witnesses(&circuit);
        assert!(dead.is_empty(), "Expected no dead witnesses, got: {dead:?}");
    }

    #[test]
    fn witness_constrained_to_constant_is_dead() {
        // w0 is private and constrained to equal 5, but it has no connection
        // to any public parameter or return value, so it is considered dead.
        let circuit = parse_circuit(
            "
            private parameters: [w0]
            public parameters: []
            return values: []
            ASSERT w0 = 5
            ",
        );
        let dead = find_dead_witnesses(&circuit);
        assert_eq!(dead, HashSet::from([Witness(0)]));
    }

    #[test]
    fn isolated_witness_is_dead() {
        // w0 is public, w1 is connected to w0, but w2 is private and in no constraint
        let circuit = parse_circuit(
            "
            private parameters: [w2]
            public parameters: [w0]
            return values: [w1]
            ASSERT w1 = w0
            ",
        );
        let dead = find_dead_witnesses(&circuit);
        assert_eq!(dead, HashSet::from([Witness(2)]));
    }

    #[test]
    fn chain_of_dead_witnesses() {
        // w0 is public, w1 is return. w0->w1 is live.
        // w2->w3 via AssertZero but neither connects to public I/O → both dead
        let circuit = parse_circuit(
            "
            private parameters: [w2, w3]
            public parameters: [w0]
            return values: [w1]
            ASSERT w1 = w0
            ASSERT w3 = w2
            ",
        );
        let dead = find_dead_witnesses(&circuit);
        assert_eq!(dead, HashSet::from([Witness(2), Witness(3)]));
    }

    #[test]
    fn brillig_does_not_create_edges() {
        // w0 is public input, w1 is Brillig output.
        // Brillig doesn't create constraint edges, so w1 is dead
        // unless connected via another ACIR constraint.
        let circuit = parse_circuit(
            "
            private parameters: []
            public parameters: [w0]
            return values: []
            BRILLIG CALL func: 0, predicate: 1, inputs: [w0], outputs: [w1]
            ",
        );
        let dead = find_dead_witnesses(&circuit);
        assert!(dead.contains(&Witness(1)), "Brillig output w1 should be dead");
    }

    #[test]
    fn brillig_output_live_through_assert() {
        // w0 is public (seed, trivially live), Brillig produces w1,
        // w1 is connected to w2 (return) via AssertZero → all live.
        let circuit = parse_circuit(
            "
            private parameters: []
            public parameters: [w0]
            return values: [w2]
            BRILLIG CALL func: 0, predicate: 1, inputs: [w0], outputs: [w1]
            ASSERT w2 = w1
            ",
        );
        let dead = find_dead_witnesses(&circuit);
        assert!(dead.is_empty(), "All witnesses should be live: {dead:?}");
    }

    #[test]
    fn brillig_inputs_dead_when_not_otherwise_constrained() {
        // w0 is private (not public), feeds into Brillig which produces w1.
        // w1 is connected to w2 (return) via AssertZero.
        // w0 is dead because it's not a seed and only appears in Brillig (no constraint edges).
        let circuit = parse_circuit(
            "
            private parameters: [w0]
            public parameters: []
            return values: [w2]
            BRILLIG CALL func: 0, predicate: 1, inputs: [w0], outputs: [w1]
            ASSERT w2 = w1
            ",
        );
        let dead = find_dead_witnesses(&circuit);
        assert_eq!(dead, HashSet::from([Witness(0)]));
    }

    #[test]
    fn memory_read_propagates_liveness() {
        // w0 is public, stored in memory block b0.
        // MemoryOp reads from b0 into w1 (value).
        // w1 is connected to w2 (return) via AssertZero.
        // All should be live because w0 and w1 share block b0.
        let circuit = parse_circuit(
            "
            private parameters: []
            public parameters: [w0]
            return values: [w2]
            INIT b0 = [w0]
            READ w1 = b0[0]
            ASSERT w2 = w1
            ",
        );
        let dead = find_dead_witnesses(&circuit);
        assert!(dead.is_empty(), "All witnesses should be live via memory read: {dead:?}");
    }

    #[test]
    fn memory_write_propagates_liveness() {
        // w0 is public, w1 is private. b0 is initialized with w0.
        // w1 is written into b0, then w2 is read from b0.
        // w2 is connected to w3 (return) via AssertZero.
        // All should be live because w0, w1, and w2 share block b0.
        let circuit = parse_circuit(
            "
            private parameters: [w1]
            public parameters: [w0]
            return values: [w3]
            INIT b0 = [w0]
            WRITE b0[0] = w1
            READ w2 = b0[0]
            ASSERT w3 = w2
            ",
        );
        let dead = find_dead_witnesses(&circuit);
        assert!(dead.is_empty(), "All witnesses should be live via memory write: {dead:?}");
    }

    #[test]
    fn memory_block_isolated_from_public_is_dead() {
        // w0 is public (seed), w1 is return, connected via AssertZero → live.
        // w2 and w3 are private, stored in b0 and read back — but b0 has
        // no connection to any public/return witness, so w2 and w3 are dead.
        let circuit = parse_circuit(
            "
            private parameters: [w2]
            public parameters: [w0]
            return values: [w1]
            ASSERT w1 = w0
            INIT b0 = [w2]
            READ w3 = b0[0]
            ",
        );
        let dead = find_dead_witnesses(&circuit);
        assert!(dead.contains(&Witness(2)), "w2 should be dead: {dead:?}");
        assert!(dead.contains(&Witness(3)), "w3 should be dead: {dead:?}");
    }

    #[test]
    fn blackbox_connects_all_witnesses() {
        // w0 is public, AND produces w2 from w0 and w1.
        // w2 is a return value. w1 should be live because blackbox connects all.
        let circuit = parse_circuit(
            "
            private parameters: [w1]
            public parameters: [w0]
            return values: [w2]
            BLACKBOX::AND lhs: w0, rhs: w1, output: w2, bits: 32
            ",
        );
        let dead = find_dead_witnesses(&circuit);
        assert!(dead.is_empty(), "All witnesses should be live: {dead:?}");
    }

    #[test]
    fn call_connects_all_witnesses() {
        // w0 is public, Call takes w0 as input and produces w1.
        // w1 is a return value. All live.
        let circuit = parse_circuit(
            "
            private parameters: []
            public parameters: [w0]
            return values: [w1]
            CALL func: 0, predicate: 1, inputs: [w0], outputs: [w1]
            ",
        );
        let dead = find_dead_witnesses(&circuit);
        assert!(dead.is_empty(), "All witnesses should be live: {dead:?}");
    }

    #[test]
    fn empty_circuit() {
        let circuit = parse_circuit(
            "
            private parameters: []
            public parameters: []
            return values: []
            ",
        );
        let dead = find_dead_witnesses(&circuit);
        assert!(dead.is_empty());
    }
}
