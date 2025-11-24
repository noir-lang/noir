use acvm::{AcirField, acir::brillig::MemoryAddress};

use crate::brillig::brillig_ir::registers::Stack;

use super::{BrilligContext, debug_show::DebugToString, registers::RegisterAllocator};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Map sources to potentially multiple destinations.
    /// This function moves values from a set of registers to another set of registers.
    /// It assumes that:
    /// - every destination needs to be written at most once. Will panic if not.
    /// - destinations are relative addresses, starting from 1 and consecutively incremented by one. Will panic if not
    /// - sources are any relative addresses, and can also be direct address to the global space. TODO: panic if not
    /// - sources and destinations have same length. Will panic if not.
    pub(crate) fn codegen_mov_registers_to_registers(
        &mut self,
        sources: &[MemoryAddress],
        destinations: &[MemoryAddress],
    ) {
        assert_eq!(sources.len(), destinations.len(), "sources and destinations length must match");
        let n = sources.len();
        let mut processed = 0;
        // Compute the number of destinations for each source node that is also a destination node (i.e within 0,..n-1) in the movement graph
        let mut num_destinations = vec![0; n];
        for i in 0..n {
            // Check that destinations are relatives to 0,..,n-1
            assert_eq!(to_index(&destinations[i]), Some(i));
            if let Some(index) = to_index(&sources[i]) {
                if index < n && index != i {
                    num_destinations[index] += 1;
                }
            }
        }

        // Process all sinks in the graph and follow their parents.
        // Keep track of nodes having more than 2 destinations, in case they belong to a loop.
        let mut tail_candidates = Vec::new();
        for i in 0..n {
            // Generate a movement for each sink in the graph
            let mut node = i;
            // A sink has no child
            while num_destinations[node] == 0 {
                if to_index(&sources[node]) == Some(node) {
                    //no-op: mark the node as processed
                    num_destinations[node] = usize::MAX;
                    processed += 1;
                    break;
                }
                // Generates a move instruction
                self.perform_movement(node, sources[node], &mut num_destinations, &mut processed);
                // Follow the parent
                if let Some(index) = to_index(&sources[node]) {
                    if index < n {
                        num_destinations[index] -= 1;
                        if num_destinations[index] > 0 {
                            // The parent node has another child, so we cannot process it yet.
                            tail_candidates.push((sources[node], node));
                            break;
                        }
                        // process the parent node
                        node = index;
                        continue;
                    }
                }
                // End of the path
                break;
            }
            if processed == n {
                return;
            }
        }
        // All sinks and their parents have been processed, remaining nodes are part of a loop
        // Check if a tail_candidate is a branch to a loop
        for (entry, free) in tail_candidates {
            let entry_idx = to_index(&entry).unwrap();
            if entry_idx < n && num_destinations[entry_idx] == 1 {
                // Use the branch as the temporary register for the loop
                self.process_loop(
                    entry_idx,
                    &from_index(free),
                    &mut num_destinations,
                    sources,
                    &mut processed,
                );
            }
        }
        if processed == n {
            return;
        }
        // Now process all the remaining loops with a temporary register.
        // Allocate one temporary per loop to avoid type confusion when reusing registers,
        // since different loops may contain values of different types.
        for i in 0..n {
            if num_destinations[i] == 1 {
                let src = from_index(i);
                // Copy the loop entry to a temporary register.
                // Unfortunately, we cannot use one register for all the loops
                // when the sources do not have the same type
                let temp_register = self.registers_mut().allocate_register();
                self.mov_instruction(temp_register, src);
                self.process_loop(
                    i,
                    &temp_register,
                    &mut num_destinations,
                    sources,
                    &mut processed,
                );
                self.deallocate_register(temp_register);
            } else {
                // Nodes must have been processed, or are part of a loop.
                assert_eq!(num_destinations[i], usize::MAX);
            }
        }
    }

    /// Generates mov opcodes corresponding to a loop, given a node from the loop (entry)
    /// and a register not in the loop that contains its value (free)
    fn process_loop(
        &mut self,
        entry: usize,
        free: &MemoryAddress,
        num_destinations: &mut [usize],
        source: &[MemoryAddress],
        processed: &mut usize,
    ) {
        let mut current = entry;
        while to_index(&source[current]).unwrap() != entry {
            self.perform_movement(current, source[current], num_destinations, processed);
            current = to_index(&source[current]).unwrap();
        }
        self.perform_movement(current, *free, num_destinations, processed);
    }

    /// Generates a move opcode from 'src' to 'dest'.
    fn perform_movement(
        &mut self,
        dest: usize,
        src: MemoryAddress,
        num_destinations: &mut [usize],
        processed: &mut usize,
    ) {
        let destination = from_index(dest);
        self.mov_instruction(destination, src);
        // set the node as 'processed'
        num_destinations[dest] = usize::MAX;
        *processed += 1;
    }
}

/// Map the address so that the first register of the stack will have index 0
fn to_index(adr: &MemoryAddress) -> Option<usize> {
    match adr {
        MemoryAddress::Relative(size) => Some(size - Stack::start()),
        MemoryAddress::Direct(_) => None,
    }
}

/// Construct the register corresponding to the given mapped 'index'
fn from_index(idx: usize) -> MemoryAddress {
    assert!(idx != usize::MAX, "invalid index");
    MemoryAddress::Relative(idx + Stack::start())
}

#[cfg(test)]
mod tests {

    use acvm::{
        FieldElement,
        acir::brillig::{MemoryAddress, Opcode},
    };
    use iter_extended::vecmap;

    use crate::{
        brillig::{
            BrilligOptions,
            brillig_ir::{
                BrilligContext, LayoutConfig,
                artifact::Label,
                registers::{RegisterAllocator, Stack},
            },
        },
        ssa::ir::function::FunctionId,
    };

    // Tests for mov_registers_to_registers

    /// Generate `Opcode::Mov` for a sequence of expected `src -> dst` moves.
    fn generate_opcodes(movements: Vec<(usize, usize)>) -> Vec<Opcode<FieldElement>> {
        movements
            .into_iter()
            .map(|(src, dst)| Opcode::Mov {
                destination: MemoryAddress::relative(dst),
                source: MemoryAddress::relative(src),
            })
            .collect()
    }

    /// Split numeric `src -> dst` movements into separate vectors and convert to `MemoryAddress`
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
        // Show the opcodes if the test fails.
        let options = BrilligOptions {
            enable_debug_trace: true,
            enable_debug_assertions: true,
            enable_array_copy_counter: false,
            show_opcode_advisories: false,
            layout: LayoutConfig::default(),
        };
        let mut context = BrilligContext::new("test", &options);
        context.enter_context(Label::function(FunctionId::test_new(0)));
        context
    }

    /// Test that a series of `src->dst` movements results in a series of `src->dst` move opcodes.
    fn assert_generated_opcodes(
        movements: Vec<(usize, usize)>,
        expected_moves: Vec<(usize, usize)>,
    ) {
        let mut context = create_context();
        for _ in 0..movements.len() {
            context.registers_mut().allocate_register();
        }
        let (sources, destinations) = movements_to_source_and_destinations(movements);
        context.codegen_mov_registers_to_registers(&sources, &destinations);

        let opcodes = context.into_artifact().byte_code;

        assert_eq!(opcodes, generate_opcodes(expected_moves));
    }

    #[test]
    fn test_no_op() {
        let movements = vec![(1, 1), (2, 2), (1, 3), (2, 4)];
        assert_generated_opcodes(movements, vec![(1, 3), (2, 4)]);
    }

    #[test]
    #[should_panic]
    fn test_mov_registers_to_registers_overwrite() {
        let movements = vec![(10, 1), (12, 1), (10, 3)];
        assert_generated_opcodes(movements, vec![]);
    }

    #[test]
    fn test_basic_no_loop() {
        let movements = vec![(2, 1), (3, 2), (4, 3), (5, 4)];
        assert_generated_opcodes(movements, vec![(2, 1), (3, 2), (4, 3), (5, 4)]);
    }

    #[test]
    fn test_basic_loop() {
        let movements = vec![(4, 1), (1, 2), (2, 3), (3, 4)];
        assert_generated_opcodes(movements, vec![(1, 5), (4, 1), (3, 4), (2, 3), (5, 2)]);
    }

    #[test]
    fn test_no_loop() {
        let movements = vec![(6, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
        assert_generated_opcodes(movements, vec![(4, 5), (3, 4), (2, 3), (1, 2), (6, 1)]);
    }

    #[test]
    fn test_loop_with_branch() {
        let movements = vec![(3, 1), (1, 2), (2, 3), (1, 4), (4, 5)];
        assert_generated_opcodes(movements, vec![(4, 5), (1, 4), (3, 1), (2, 3), (4, 2)]);
    }

    #[test]
    fn test_two_loops() {
        let movements = vec![(3, 1), (1, 2), (2, 3), (6, 4), (4, 5), (5, 6)];
        assert_generated_opcodes(
            movements,
            vec![(1, 7), (3, 1), (2, 3), (7, 2), (4, 7), (6, 4), (5, 6), (7, 5)],
        );
    }

    #[test]
    fn test_another_loop_with_branch() {
        let movements = vec![(2, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
        assert_generated_opcodes(movements, vec![(4, 5), (3, 4), (2, 3), (1, 2), (3, 1)]);
    }
    #[test]
    fn test_one_loop() {
        let movements = vec![(2, 1), (4, 2), (5, 3), (3, 4), (1, 5)];
        assert_generated_opcodes(movements, vec![(1, 6), (2, 1), (4, 2), (3, 4), (5, 3), (6, 5)]);
    }

    /// This creates a chain (N+1)->1->2->...->N where N is large enough to overflow the stack
    #[test]
    fn test_deep_chain() {
        // Each movement is i -> i+1, creating a single long chain
        const CHAIN_DEPTH: usize = 10_000;

        let mut movements: Vec<(usize, usize)> = (0..CHAIN_DEPTH).map(|i| (i, i + 1)).collect();
        movements[0] = (CHAIN_DEPTH + 1, 1);
        let (sources, destinations) = movements_to_source_and_destinations(movements);

        let mut context = create_context();

        // This should overflow the stack with recursive implementation
        context.codegen_mov_registers_to_registers(&sources, &destinations);
    }

    #[test]
    fn prop_mov_registers_to_registers() {
        const MEM_SIZE: usize = 10;
        arbtest::arbtest(|u| {
            // Allocate more memory to allow for temporary variables.
            let mut memory: Vec<u32> = vec![0; MEM_SIZE * 2];
            // Fill the memory with some random numbers.
            for slot in memory.iter_mut() {
                *slot = u.arbitrary()?;
            }

            // Pick a random unique subset of the slots as destinations.
            let num_destinations = u.int_in_range(0..=MEM_SIZE)?;

            // All potential memory slots; we can't address before the stack start.
            let all_indexes = (0..MEM_SIZE).map(|i| i + Stack::start()).collect::<Vec<_>>();

            let destinations: Vec<usize> = (1..num_destinations + 1).collect();

            // Pick random sources for each destination (same source can be repeated).
            let mut sources = Vec::with_capacity(num_destinations);
            for _ in 0..num_destinations {
                sources.push(u.choose(&all_indexes).copied()?);
            }

            // Take a snapshot of the source data; this is what we expect the destination to become.
            let source_data = vecmap(&sources, |i| memory[*i]);

            // Generate the opcodes.
            let opcodes = {
                // Convert to MemoryAddress
                let sources = vecmap(&sources, |i| MemoryAddress::relative(*i));
                let destinations = vecmap(&destinations, |i| MemoryAddress::relative(*i));

                let mut context = create_context();

                // Treat the memory we care about as pre-allocated, so temporary variables are created after them.
                let all_registers = vecmap(all_indexes, MemoryAddress::relative);
                context.set_allocated_registers(all_registers);

                context.codegen_mov_registers_to_registers(&sources, &destinations);
                context.into_artifact().byte_code
            };

            // Execute the opcodes.
            for opcode in opcodes {
                let Opcode::Mov { destination, source } = opcode else {
                    unreachable!("only Mov expected");
                };
                memory[destination.to_usize()] = memory[source.to_usize()];
            }

            // Get the final values at the destination slots.
            let destination_data = vecmap(&destinations, |i| memory[*i]);

            // At the end the destination should have the same value as the source had.
            assert_eq!(destination_data, source_data);

            Ok(())
        })
        .run();
    }
}
