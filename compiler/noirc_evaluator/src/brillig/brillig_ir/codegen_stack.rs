use std::collections::HashMap;

use acvm::{AcirField, acir::brillig::MemoryAddress};

use super::{BrilligContext, debug_show::DebugToString, registers::RegisterAllocator};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Map sources to potentially multiple destinations.
    /// This function moves values from a set of registers to another set of registers.
    /// The movement is treated as a set of simultaneous assignments: the moves are ordered
    /// (and cycles broken with temporaries) so that every destination ends up holding the
    /// value its source held *before* any move was performed.
    ///
    /// When `condition` is `Some(_)`, every destination is written with a `conditional_move`
    /// guarded by that register, so the whole batch is a no-op when the condition is false;
    /// with `None` the destinations are written unconditionally.
    ///
    /// It assumes that:
    /// - every destination needs to be written at most once. Will panic if not.
    /// - sources and destinations are any addresses (relative registers or direct globals);
    ///   a source that is not also a destination is read as-is.
    /// - sources and destinations have same length. Will panic if not.
    pub(crate) fn codegen_mov_registers_to_registers(
        &mut self,
        sources: &[MemoryAddress],
        destinations: &[MemoryAddress],
        condition: Option<MemoryAddress>,
    ) {
        assert_eq!(sources.len(), destinations.len(), "sources and destinations length must match");
        let n = sources.len();

        // Map each destination address to its node index. Destinations must be unique,
        // since every destination is written at most once.
        let mut dest_index: HashMap<MemoryAddress, usize> = HashMap::with_capacity(n);
        for (i, destination) in destinations.iter().enumerate() {
            let previous = dest_index.insert(*destination, i);
            assert!(previous.is_none(), "destination {destination:?} is written more than once");
        }

        let mut processed = 0;
        // Count, for each node, how many *other* nodes read the value it writes (i.e. use its
        // destination address as their source). A node with zero such readers is a sink and can
        // be written immediately.
        let mut num_destinations = vec![0; n];
        for (i, source) in sources.iter().enumerate() {
            if let Some(index) = to_index(&dest_index, source)
                && index != i
            {
                num_destinations[index] += 1;
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
                if to_index(&dest_index, &sources[node]) == Some(node) {
                    //no-op: mark the node as processed
                    num_destinations[node] = usize::MAX;
                    processed += 1;
                    break;
                }
                // Generates a move instruction
                self.perform_movement(
                    node,
                    sources[node],
                    destinations,
                    condition,
                    &mut num_destinations,
                    &mut processed,
                );
                // Follow the parent
                if let Some(index) = to_index(&dest_index, &sources[node]) {
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
                // End of the path
                break;
            }
            if processed == n {
                return;
            }
        }
        // All sinks and their parents have been processed, remaining nodes are part of a loop.
        // Check if a tail_candidate is a branch to a loop.
        //
        // This reuses an already-written destination (`free`) as the loop's scratch register.
        // That is only sound when the writes are unconditional: under a false `condition` the
        // move into `free` never happened, so it would hold stale data. When conditional, we
        // skip this and let these loops fall through to the fresh-temporary path below.
        if condition.is_none() {
            for (entry, free) in tail_candidates {
                let entry_idx = to_index(&dest_index, &entry).unwrap();
                if num_destinations[entry_idx] == 1 {
                    // Use the branch as the temporary register for the loop
                    let free_register = from_index(destinations, free);
                    self.process_loop(
                        entry_idx,
                        &free_register,
                        destinations,
                        &dest_index,
                        condition,
                        &mut num_destinations,
                        sources,
                        &mut processed,
                    );
                }
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
                let src = from_index(destinations, i);
                // Copy the loop entry to a temporary register.
                // Unfortunately, we cannot use one register for all the loops
                // when the sources do not have the same type
                let temp_register = self.registers_mut().allocate_register();
                // Prime the temporary with the loop entry unconditionally: it is fresh scratch,
                // so writing it even when the condition is false is harmless.
                self.mov_instruction(temp_register, src);
                self.process_loop(
                    i,
                    &temp_register,
                    destinations,
                    &dest_index,
                    condition,
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
    #[allow(clippy::too_many_arguments)]
    fn process_loop(
        &mut self,
        entry: usize,
        free: &MemoryAddress,
        destinations: &[MemoryAddress],
        dest_index: &HashMap<MemoryAddress, usize>,
        condition: Option<MemoryAddress>,
        num_destinations: &mut [usize],
        source: &[MemoryAddress],
        processed: &mut usize,
    ) {
        let mut current = entry;
        while to_index(dest_index, &source[current]).unwrap() != entry {
            self.perform_movement(
                current,
                source[current],
                destinations,
                condition,
                num_destinations,
                processed,
            );
            current = to_index(dest_index, &source[current]).unwrap();
        }
        self.perform_movement(current, *free, destinations, condition, num_destinations, processed);
    }

    /// Generates a move opcode from 'src' to 'dest'. When `condition` is `Some(_)`, the write
    /// is a `conditional_move` guarded by it (leaving 'dest' unchanged when false); otherwise
    /// it is an unconditional `mov`.
    fn perform_movement(
        &mut self,
        dest: usize,
        src: MemoryAddress,
        destinations: &[MemoryAddress],
        condition: Option<MemoryAddress>,
        num_destinations: &mut [usize],
        processed: &mut usize,
    ) {
        let destination = from_index(destinations, dest);
        match condition {
            None => self.mov_instruction(destination, src),
            Some(condition) => {
                self.conditional_move_instruction(condition, src, destination, destination);
            }
        }
        // set the node as 'processed'
        num_destinations[dest] = usize::MAX;
        *processed += 1;
    }
}

/// Look up the node index of an address that is one of the destinations, if any.
/// Sources that are not destinations (e.g. globals or registers outside the destination set)
/// return `None` and are treated as leaves in the movement graph.
fn to_index(dest_index: &HashMap<MemoryAddress, usize>, adr: &MemoryAddress) -> Option<usize> {
    dest_index.get(adr).copied()
}

/// Recover the destination address of the given node index.
fn from_index(destinations: &[MemoryAddress], idx: usize) -> MemoryAddress {
    assert!(idx != usize::MAX, "invalid index");
    destinations[idx]
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
            BrilligOptions, assert_u32, assert_usize,
            brillig_ir::{
                BrilligContext, LayoutConfig, Stack, artifact::Label, registers::RegisterAllocator,
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
                destination: MemoryAddress::relative(assert_u32(dst)),
                source: MemoryAddress::relative(assert_u32(src)),
            })
            .collect()
    }

    /// Split numeric `src -> dst` movements into separate vectors and convert to `MemoryAddress`
    fn movements_to_source_and_destinations(
        movements: Vec<(usize, usize)>,
    ) -> (Vec<MemoryAddress>, Vec<MemoryAddress>) {
        let sources = movements
            .iter()
            .map(|(source, _)| MemoryAddress::relative(assert_u32(*source)))
            .collect();
        let destinations = movements
            .iter()
            .map(|(_, destination)| MemoryAddress::relative(assert_u32(*destination)))
            .collect();
        (sources, destinations)
    }

    pub(crate) fn create_context() -> BrilligContext<FieldElement, Stack> {
        // Show the opcodes if the test fails.
        let options = BrilligOptions {
            enable_debug_trace: true,
            enable_debug_assertions: true,
            show_opcode_advisories: false,
            layout: LayoutConfig::default(),
            copy_site_registry: None,
            use_linear_scan_allocator: false,
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
        context.codegen_mov_registers_to_registers(&sources, &destinations, None);

        let opcodes = context.into_artifact().byte_code;

        assert_eq!(opcodes, generate_opcodes(expected_moves));
    }

    /// Stack offset base. All stacks now start at offset 2 (see `Stack::new`).
    const S: usize = 2;

    #[test]
    fn test_no_op() {
        let movements = vec![(S, S), (S + 1, S + 1), (S, S + 2), (S + 1, S + 3)];
        assert_generated_opcodes(movements, vec![(S, S + 2), (S + 1, S + 3)]);
    }

    #[test]
    #[should_panic]
    fn test_mov_registers_to_registers_overwrite() {
        let movements = vec![(S + 9, S), (S + 11, S), (S + 9, S + 2)];
        assert_generated_opcodes(movements, vec![]);
    }

    #[test]
    fn test_basic_no_loop() {
        let movements = vec![(S + 1, S), (S + 2, S + 1), (S + 3, S + 2), (S + 4, S + 3)];
        assert_generated_opcodes(
            movements,
            vec![(S + 1, S), (S + 2, S + 1), (S + 3, S + 2), (S + 4, S + 3)],
        );
    }

    #[test]
    fn test_basic_loop() {
        let movements = vec![(S + 3, S), (S, S + 1), (S + 1, S + 2), (S + 2, S + 3)];
        assert_generated_opcodes(
            movements,
            vec![(S, S + 4), (S + 3, S), (S + 2, S + 3), (S + 1, S + 2), (S + 4, S + 1)],
        );
    }

    #[test]
    fn test_no_loop() {
        let movements =
            vec![(S + 5, S), (S, S + 1), (S + 1, S + 2), (S + 2, S + 3), (S + 3, S + 4)];
        assert_generated_opcodes(
            movements,
            vec![(S + 3, S + 4), (S + 2, S + 3), (S + 1, S + 2), (S, S + 1), (S + 5, S)],
        );
    }

    #[test]
    fn test_loop_with_branch() {
        let movements = vec![(S + 2, S), (S, S + 1), (S + 1, S + 2), (S, S + 3), (S + 3, S + 4)];
        assert_generated_opcodes(
            movements,
            vec![(S + 3, S + 4), (S, S + 3), (S + 2, S), (S + 1, S + 2), (S + 3, S + 1)],
        );
    }

    #[test]
    fn test_two_loops() {
        let movements = vec![
            (S + 2, S),
            (S, S + 1),
            (S + 1, S + 2),
            (S + 5, S + 3),
            (S + 3, S + 4),
            (S + 4, S + 5),
        ];
        assert_generated_opcodes(
            movements,
            vec![
                (S, S + 6),
                (S + 2, S),
                (S + 1, S + 2),
                (S + 6, S + 1),
                (S + 3, S + 6),
                (S + 5, S + 3),
                (S + 4, S + 5),
                (S + 6, S + 4),
            ],
        );
    }

    #[test]
    fn test_another_loop_with_branch() {
        let movements =
            vec![(S + 1, S), (S, S + 1), (S + 1, S + 2), (S + 2, S + 3), (S + 3, S + 4)];
        assert_generated_opcodes(
            movements,
            vec![(S + 3, S + 4), (S + 2, S + 3), (S + 1, S + 2), (S, S + 1), (S + 2, S)],
        );
    }

    #[test]
    fn test_one_loop() {
        let movements =
            vec![(S + 1, S), (S + 3, S + 1), (S + 4, S + 2), (S + 2, S + 3), (S, S + 4)];
        assert_generated_opcodes(
            movements,
            vec![
                (S, S + 5),
                (S + 1, S),
                (S + 3, S + 1),
                (S + 2, S + 3),
                (S + 4, S + 2),
                (S + 5, S + 4),
            ],
        );
    }

    /// This creates a chain (S+N)->S->(S+1)->...->S+(N-1) where N is large enough to overflow the stack
    #[test]
    fn test_deep_chain() {
        const CHAIN_DEPTH: usize = 10_000;

        // destinations[i] = S+i, sources form a chain: S+N, S, S+1, ..., S+N-2
        let movements: Vec<(usize, usize)> = (0..CHAIN_DEPTH)
            .map(|i| if i == 0 { (S + CHAIN_DEPTH, S) } else { (S + i - 1, S + i) })
            .collect();
        let (sources, destinations) = movements_to_source_and_destinations(movements);

        let mut context = create_context();

        // This should overflow the stack with recursive implementation
        context.codegen_mov_registers_to_registers(&sources, &destinations, None);
    }

    #[test]
    fn prop_mov_registers_to_registers() {
        const MEM_SIZE: usize = 10;
        arbtest::arbtest(|u| {
            // Room for the working slots, the condition register, and any temporaries.
            let mut initial_memory: Vec<u32> = vec![0; MEM_SIZE * 3];
            // Fill the memory with some random numbers.
            for slot in &mut initial_memory {
                *slot = u.arbitrary()?;
            }

            // Pick a random unique subset of the slots as destinations.
            let num_destinations = u.int_in_range(0..=MEM_SIZE)?;

            // All potential source/destination slots; we can't address before the stack start.
            let all_indexes = (0..MEM_SIZE).map(|i| i + S).collect::<Vec<_>>();

            // Choose the destinations as a random unique subset of the slots (in an
            // arbitrary order, not necessarily consecutive) to exercise the general
            // any-source-to-any-destination mover.
            let mut pool = all_indexes.clone();
            let mut destinations = Vec::with_capacity(num_destinations);
            for _ in 0..num_destinations {
                let idx = u.int_in_range(0..=pool.len() - 1)?;
                destinations.push(pool.swap_remove(idx));
            }

            // Pick random sources for each destination (same source can be repeated).
            let mut sources = Vec::with_capacity(num_destinations);
            for _ in 0..num_destinations {
                sources.push(u.choose(&all_indexes).copied()?);
            }

            // A dedicated condition register just past the working slots, so it is never a
            // source or destination and the mover's temporaries are allocated above it.
            let condition_slot = S + MEM_SIZE;

            // Exercise the same move set unconditionally, and conditionally with both a true
            // and a false guard. A true (or absent) guard performs the moves; a false guard
            // must leave every destination untouched.
            for condition_value in [None, Some(true), Some(false)] {
                let mut memory = initial_memory.clone();
                if let Some(value) = condition_value {
                    memory[condition_slot] = u32::from(value);
                }

                // Snapshots taken before the moves: what the destinations should become when
                // the moves run, and what they must stay when a false guard suppresses them.
                let source_data = vecmap(&sources, |i| memory[*i]);
                let original_destination_data = vecmap(&destinations, |i| memory[*i]);

                let condition_address =
                    condition_value.map(|_| MemoryAddress::relative(assert_u32(condition_slot)));

                // Generate the opcodes.
                let opcodes = {
                    // Convert to MemoryAddress
                    let sources = vecmap(&sources, |i| MemoryAddress::relative(assert_u32(*i)));
                    let destinations =
                        vecmap(&destinations, |i| MemoryAddress::relative(assert_u32(*i)));

                    let mut context = create_context();

                    // Pre-allocate the working slots and the condition register, so temporary
                    // variables are created above them.
                    let mut allocated =
                        vecmap(&all_indexes, |i| MemoryAddress::relative(assert_u32(*i)));
                    allocated.push(MemoryAddress::relative(assert_u32(condition_slot)));
                    context.set_allocated_registers(allocated);

                    context.codegen_mov_registers_to_registers(
                        &sources,
                        &destinations,
                        condition_address,
                    );
                    context.into_artifact().byte_code
                };

                // Execute the opcodes.
                for opcode in opcodes {
                    match opcode {
                        Opcode::Mov { destination, source } => {
                            memory[assert_usize(destination.to_u32())] =
                                memory[assert_usize(source.to_u32())];
                        }
                        Opcode::ConditionalMov { destination, source_a, source_b, condition } => {
                            let taken = memory[assert_usize(condition.to_u32())] != 0;
                            let source = if taken { source_a } else { source_b };
                            memory[assert_usize(destination.to_u32())] =
                                memory[assert_usize(source.to_u32())];
                        }
                        other => unreachable!("only Mov/ConditionalMov expected, got {other:?}"),
                    }
                }

                // Get the final values at the destination slots.
                let destination_data = vecmap(&destinations, |i| memory[*i]);

                match condition_value {
                    None | Some(true) => assert_eq!(destination_data, source_data),
                    Some(false) => assert_eq!(destination_data, original_destination_data),
                }
            }

            Ok(())
        })
        .run();
    }
}
