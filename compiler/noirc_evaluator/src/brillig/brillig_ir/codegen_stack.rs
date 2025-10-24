use std::collections::{BTreeMap, BTreeSet, HashSet};

use acvm::{AcirField, acir::brillig::MemoryAddress};

use super::{BrilligContext, debug_show::DebugToString, registers::RegisterAllocator};

/// Map sources to potentially multiple destination.
///
/// Using a BTree so we get deterministic loop detection.
type MovementsMap = BTreeMap<MemoryAddress, BTreeSet<MemoryAddress>>;

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// This function moves values from a set of registers to another set of registers.
    ///
    /// The only requirement is that every destination needs to be written at most once.
    /// The same source can be copied to multiple destinations.
    ///
    /// The method detects cycles in the movements and breaks them by introducing
    /// temporary registers.
    pub(crate) fn codegen_mov_registers_to_registers(
        &mut self,
        sources: Vec<MemoryAddress>,
        destinations: Vec<MemoryAddress>,
    ) {
        assert_eq!(sources.len(), destinations.len());
        // Remove all no-ops
        let movements: Vec<_> = sources
            .into_iter()
            .zip(destinations)
            .filter(|(source, destination)| source != destination)
            .collect();

        // Now we need to detect all cycles.
        // First build a map of the movements. Note that a source could have multiple destinations
        let mut movements_map: MovementsMap =
            movements.into_iter().fold(BTreeMap::default(), |mut map, (source, destination)| {
                map.entry(source).or_default().insert(destination);
                map
            });

        // Unique addresses that get anything moved into them.
        let destinations_set: BTreeSet<_> = movements_map.values().flatten().copied().collect();

        // Ensure that all destinations are unique.
        assert_eq!(
            destinations_set.len(),
            movements_map.values().flatten().count(),
            "Multiple moves to the same register found"
        );

        let loops = LoopDetector::detect_loops(&movements_map);

        // In order to break the loops we need to store one register from each in a temporary and then use that temporary as source.
        let mut temporaries = Vec::with_capacity(loops.len());

        for loop_found in loops {
            let temp_register = self.allocate_register();
            let first_source = loop_found.get_min().unwrap();
            self.mov_instruction(*temp_register, *first_source);
            let destinations_of_temp = movements_map.remove(first_source).unwrap();
            movements_map.insert(*temp_register, destinations_of_temp);
            temporaries.push(temp_register);
        }

        // After removing loops we should have an DAG with each node having only one ancestor (but could have multiple successors)
        // Now we should be able to move the registers just by performing a DFS on the movements map.
        // Start from the heads, which are not destinations; anything else should be reachable from them by following the paths.
        let heads: Vec<_> = movements_map
            .keys()
            .filter(|source| !destinations_set.contains(source))
            .copied()
            .collect();

        for head in heads {
            self.perform_movements(&movements_map, head);
        }
    }

    /// Starting from the head do a DFS through the movements to find the last destination,
    /// then generate movement opcodes _backwards_, unraveling the DFS path.
    ///
    /// By doing so, we can have a series of moves such as `[1->2, 2->3]` which become opcodes
    /// [3<-2, 2<-1], without having 2 overwritten by 1 first, before it could be copied to 3.
    fn perform_movements(&mut self, movements: &MovementsMap, current_source: MemoryAddress) {
        if let Some(destinations) = movements.get(&current_source) {
            for destination in destinations {
                self.perform_movements(movements, *destination);
            }
            for destination in destinations {
                self.mov_instruction(*destination, current_source);
            }
        }
    }
}

/// Addresses visited when a loop was found in the movements.
///
/// It might contain addresses that aren't actually part of the loop,
/// e.g. A->B->C->D->C would contain [A,B,C,D] even though the loop is just [C,D].
///
/// They are ordered by their value, not their appearance in the loop.
/// The order provides determinism when we try to break the loop.
///
/// Using an immutable data structure for structural sharing during DFS.
type AddressLoop = im::OrdSet<MemoryAddress>;

struct LoopDetector {
    visited_sources: HashSet<MemoryAddress>,
    loops: Vec<AddressLoop>,
}

impl LoopDetector {
    fn detect_loops(
        movements: &BTreeMap<MemoryAddress, BTreeSet<MemoryAddress>>,
    ) -> Vec<AddressLoop> {
        let mut detector = Self { visited_sources: Default::default(), loops: Default::default() };
        detector.collect_loops(movements);
        detector.loops
    }

    fn collect_loops(&mut self, movements: &MovementsMap) {
        for source in movements.keys() {
            self.find_loop_recursive(*source, movements, im::OrdSet::default());
        }
    }

    fn find_loop_recursive(
        &mut self,
        source: MemoryAddress,
        movements: &MovementsMap,
        mut previous_sources: AddressLoop,
    ) {
        // Mark as visited
        if !self.visited_sources.insert(source) {
            return;
        }

        previous_sources.insert(source);
        // Get all destinations (this can be empty when we treat destinations as sources during recursion).
        if let Some(destinations) = movements.get(&source) {
            for destination in destinations {
                if previous_sources.contains(destination) {
                    // Found a loop
                    let loop_sources = previous_sources.clone();
                    self.loops.push(loop_sources);
                } else {
                    self.find_loop_recursive(*destination, movements, previous_sources.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use acvm::{
        FieldElement,
        acir::brillig::{MemoryAddress, Opcode},
    };

    use crate::{
        brillig::{
            BrilligOptions,
            brillig_ir::{
                BrilligContext, LayoutConfig,
                artifact::Label,
                codegen_stack::{LoopDetector, MovementsMap},
                registers::Stack,
            },
        },
        ssa::ir::function::FunctionId,
    };

    // Tests for the loop finder

    /// Generate a movements map from test data, turning numbers into relative addresses.
    fn generate_movements_map(movements: Vec<(usize, usize)>) -> MovementsMap {
        movements.into_iter().fold(BTreeMap::default(), |mut map, (source, destination)| {
            map.entry(MemoryAddress::relative(source))
                .or_default()
                .insert(MemoryAddress::relative(destination));
            map
        })
    }

    #[test]
    fn test_loop_detector_basic_loop() {
        let movements = vec![(0, 1), (1, 2), (2, 3), (3, 0)];
        let movements_map = generate_movements_map(movements);
        let loops = LoopDetector::detect_loops(&movements_map);
        assert_eq!(loops.len(), 1);
        assert_eq!(loops[0].len(), 4);
    }

    #[test]
    fn test_loop_detector_loop_with_init() {
        // 0->1->2->3->2
        let movements = vec![(0, 1), (1, 2), (2, 3), (3, 2)];
        let movements_map = generate_movements_map(movements);
        let loops = LoopDetector::detect_loops(&movements_map);
        assert_eq!(loops.len(), 1);
        assert_eq!(
            loops[0],
            im::OrdSet::from_iter([0, 1, 2, 3].map(MemoryAddress::relative)),
            "0 and 1 are in the detection set, despite not being part of the loop body"
        );
    }

    #[test]
    fn test_loop_detector_no_loop() {
        let movements = vec![(0, 1), (1, 2), (2, 3), (3, 4)];
        let movements_map = generate_movements_map(movements);
        let loops = LoopDetector::detect_loops(&movements_map);
        assert_eq!(loops.len(), 0);
    }

    #[test]
    fn test_loop_detector_loop_with_branch() {
        let movements = vec![(0, 1), (1, 2), (2, 0), (0, 3), (3, 4)];
        let movements_map = generate_movements_map(movements);
        let loops = LoopDetector::detect_loops(&movements_map);
        assert_eq!(loops.len(), 1);
        assert_eq!(loops[0].len(), 3);
    }

    #[test]
    fn test_loop_detector_two_loops() {
        let movements = vec![(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3)];
        let movements_map = generate_movements_map(movements);
        let loops = LoopDetector::detect_loops(&movements_map);
        assert_eq!(loops.len(), 2);
        assert_eq!(loops[0].len(), 3);
        assert_eq!(loops[1].len(), 3);
    }

    // Tests for mov_registers_to_registers

    /// Generate `Opcode::Mov` for a sequence of expected `dst <- src` moves.
    ///
    /// Note that this is the opposite order of expected by `generate_movements_map`.
    fn generate_opcodes(movements: Vec<(usize, usize)>) -> Vec<Opcode<FieldElement>> {
        movements
            .into_iter()
            .map(|(dst, src)| Opcode::Mov {
                destination: MemoryAddress::relative(dst),
                source: MemoryAddress::relative(src),
            })
            .collect()
    }

    fn movements_to_source_and_destinations(
        movements: Vec<(usize, usize)>,
    ) -> (Vec<MemoryAddress>, Vec<MemoryAddress>) {
        let sources =
            movements.iter().map(|(source, _)| MemoryAddress::relative(*source)).collect();
        let destinations = movements
            .iter()
            .map(|(_, destination)| MemoryAddress::relative(*destination))
            .collect();
        (sources, destinations)
    }

    pub(crate) fn create_context() -> BrilligContext<FieldElement, Stack> {
        let options = BrilligOptions {
            enable_debug_trace: true,
            enable_debug_assertions: true,
            enable_array_copy_counter: false,
            layout: LayoutConfig::default(),
        };
        let mut context = BrilligContext::new("test", &options);
        context.enter_context(Label::function(FunctionId::test_new(0)));
        context
    }

    fn assert_generated_opcodes(
        // (src, dst)
        movements: Vec<(usize, usize)>,
        // (dst, src)
        expected_moves: Vec<(usize, usize)>,
    ) {
        let (sources, destinations) = movements_to_source_and_destinations(movements);

        let mut context = create_context();
        context.codegen_mov_registers_to_registers(sources, destinations);

        let opcodes = context.artifact().byte_code;

        assert_eq!(opcodes, generate_opcodes(expected_moves));
    }

    #[test]
    #[should_panic(expected = "Multiple moves to the same register found")]
    fn test_mov_registers_to_registers_overwrite() {
        let movements = vec![(10, 11), (12, 11), (10, 13)];
        assert_generated_opcodes(movements, vec![]);
    }

    #[test]
    fn test_mov_registers_to_registers_no_loop() {
        let movements = vec![(10, 11), (11, 12), (12, 13), (13, 14)];
        let expected_moves = vec![(14, 13), (13, 12), (12, 11), (11, 10)];
        assert_generated_opcodes(movements, expected_moves);
    }
    #[test]
    fn test_mov_registers_to_registers_no_op_filter() {
        let movements = vec![(10, 11), (11, 11), (11, 12)];
        let expected_moves = vec![(12, 11), (11, 10)];
        assert_generated_opcodes(movements, expected_moves);
    }

    #[test]
    fn test_mov_registers_to_registers_loop() {
        let movements = vec![(10, 11), (11, 12), (12, 13), (13, 10)];
        let expected_moves = vec![(1, 10), (10, 13), (13, 12), (12, 11), (11, 1)];
        assert_generated_opcodes(movements, expected_moves);
    }

    #[test]
    fn test_mov_registers_to_registers_loop_and_branch() {
        let movements = vec![(10, 11), (11, 12), (12, 10), (10, 13), (13, 14)];
        let expected_moves = vec![
            (1, 10),  // Temporary
            (10, 12), // Branch
            (12, 11), // Loop
            (14, 13), // Loop
            (11, 1),  // Finish branch
            (13, 1),  // Finish loop
        ];
        assert_generated_opcodes(movements, expected_moves);
    }
}
