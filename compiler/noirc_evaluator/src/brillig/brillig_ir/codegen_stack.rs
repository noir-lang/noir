use acvm::{AcirField, acir::brillig::MemoryAddress};

use crate::brillig::brillig_ir::registers::Stack;

use super::{BrilligContext, debug_show::DebugToString, registers::RegisterAllocator};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Map the address so that the first register of the stack will have index 0
    fn to_index(adr: &MemoryAddress) -> usize {
        match adr {
            MemoryAddress::Relative(size) => size - Stack::start(),
            MemoryAddress::Direct(_) => usize::MAX,
        }
    }

    /// Construct the register corresponding to the given mapped 'index'
    fn from_index(idx: usize) -> MemoryAddress {
        assert!(idx != usize::MAX, "invalid index");
        MemoryAddress::Relative(idx + Stack::start())
    }

    /// This function moves values from a set of registers to another set of registers.
    /// It assumes that:
    /// - every destination needs to be written at most once. Will panic if not.
    /// - destinations are relative addresses, starting from 1 and with 1 increment. Will panic if not
    /// - sources are any relative addresses, and can also be direct address to the global space. TODO: panic if not
    /// - sources and destinations have same length. Will panic if not.
    pub(crate) fn codegen_mov_registers_to_registers(
        &mut self,
        sources: Vec<MemoryAddress>,
        destinations: Vec<MemoryAddress>,
    ) {
        assert_eq!(sources.len(), destinations.len());
        let n = sources.len();
        let start = Stack::start();
        let mut processed = 0;
        // Compute the number of children for each destination node in the movement graph
        let mut children = vec![0; n];
        for i in 0..n {
            // Check that destinations are relatives to 0,..,n-1
            assert_eq!(destinations[i].unwrap_relative() - start, i);
            let index = Self::to_index(&sources[i]);
            if index < n && index != i {
                children[index] += 1;
            }
        }

        // Process all sinks in the graph and follow their parents.
        // Keep track of nodes having more than 2 children, in case they belong to a loop.
        let mut tail_candidates = Vec::new();
        for i in 0..n {
            // Generate a movement for each sink in the graph
            if children[i] == 0 {
                // A sink has no child
                let mut node = i;
                while children[node] == 0 {
                    if Self::to_index(&sources[node]) == node {
                        //no-op: mark the node as processed
                        children[node] = usize::MAX;
                        processed += 1;
                        break;
                    }
                    // Generates a move instruction
                    self.perform_movement(node, sources[node], &mut children, &mut processed);
                    // Follow the parent
                    let index = Self::to_index(&sources[node]);
                    if index < n {
                        children[index] -= 1;
                        if children[index] > 0 {
                            // The parent node has another child, so we cannot process it yet.
                            tail_candidates.push((sources[node], node));
                            break;
                        }
                        // process the parent node
                        node = index;
                    } else {
                        // End of the path
                        break;
                    }
                }
                if processed == n {
                    return;
                }
            }
        }
        // All sinks and their parents have been processed, remaining nodes are part of a loop
        // Check if a tail_candidate is a branch to a loop
        for (entry, free) in tail_candidates {
            let entry_idx = Self::to_index(&entry);
            if entry_idx < n && children[entry_idx] == 1 {
                // Use the branch as the temporary register for the loop
                self.process_loop(
                    entry_idx,
                    &Self::from_index(free),
                    &mut children,
                    &sources,
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
            if children[i] == 1 {
                let src = Self::from_index(i);
                // Copy the loop entry to a temporary register.
                // Unfortunately, we cannot use one register for all the loops
                // when the sources do not have the same type
                let temp_register = self.registers_mut().allocate_register();
                self.mov_instruction(temp_register, src);
                self.process_loop(i, &temp_register, &mut children, &sources, &mut processed);
                self.deallocate_register(temp_register);
            }
        }
    }

    /// Generates mov opcodes corresponding to a loop, given a node from the loop (entry)
    /// and a register not in the loop that contains its value (free)
    fn process_loop(
        &mut self,
        entry: usize,
        free: &MemoryAddress,
        children: &mut [usize],
        source: &[MemoryAddress],
        processed: &mut usize,
    ) {
        let mut current = entry;
        while Self::to_index(&source[current]) != entry {
            self.perform_movement(current, source[current], children, processed);
            current = Self::to_index(&source[current]);
        }
        self.perform_movement(current, *free, children, processed);
        children[current] = usize::MAX;
    }

    /// Generates a move opcode from 'src' to 'dest'.
    fn perform_movement(
        &mut self,
        dest: usize,
        src: MemoryAddress,
        children: &mut [usize],
        processed: &mut usize,
    ) {
        let destination = Self::from_index(dest);
        self.mov_instruction(destination, src);
        // set the node as 'processed'
        children[dest] = usize::MAX;
        *processed += 1;
    }
}

#[cfg(test)]
mod tests {
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
                registers::{RegisterAllocator, Stack},
            },
        },
        ssa::ir::function::FunctionId,
    };

    // Tests for mov_registers_to_registers

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

    #[test]
    fn test_no_op() {
        let movements = vec![(1, 1), (2, 2), (1, 3), (2, 4)];
        let (sources, destinations) = movements_to_source_and_destinations(movements);
        let mut context = create_context();

        context.codegen_mov_registers_to_registers(sources, destinations);
        let opcodes = context.artifact().byte_code;
        assert_eq!(opcodes.len(), 2);
    }

    #[test]
    #[should_panic]
    fn test_mov_registers_to_registers_overwrite() {
        let movements = vec![(10, 1), (12, 1), (10, 3)];
        let (sources, destinations) = movements_to_source_and_destinations(movements);
        let mut context = create_context();

        context.codegen_mov_registers_to_registers(sources, destinations);
    }

    #[test]
    fn test_basic_no_loop() {
        let movements = vec![(2, 1), (3, 2), (4, 3), (5, 4)];
        // let movements_map = generate_movements_map(movements);
        let (sources, destinations) = movements_to_source_and_destinations(movements);
        let mut context = create_context();

        context.codegen_mov_registers_to_registers(sources, destinations);
        let opcodes = context.artifact().byte_code;
        assert_eq!(
            opcodes,
            vec![
                Opcode::Mov {
                    destination: MemoryAddress::relative(1),
                    source: MemoryAddress::relative(2)
                },
                Opcode::Mov {
                    destination: MemoryAddress::relative(2),
                    source: MemoryAddress::relative(3)
                },
                Opcode::Mov {
                    destination: MemoryAddress::relative(3),
                    source: MemoryAddress::relative(4)
                },
                Opcode::Mov {
                    destination: MemoryAddress::relative(4),
                    source: MemoryAddress::relative(5)
                }
            ]
        );
    }

    #[test]
    fn test_basic_loop() {
        let movements = vec![(4, 1), (1, 2), (2, 3), (3, 4)];
        let mut context = create_context();
        for _ in 0..movements.len() {
            context.registers_mut().allocate_register();
        }
        let (sources, destinations) = movements_to_source_and_destinations(movements);

        context.codegen_mov_registers_to_registers(sources, destinations);
        let opcodes = context.artifact().byte_code;
        assert_eq!(opcodes.len(), 5);
    }

    #[test]
    fn test_no_loop() {
        let movements = vec![(6, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
        let mut context = create_context();

        let (sources, destinations) = movements_to_source_and_destinations(movements);

        context.codegen_mov_registers_to_registers(sources, destinations);
        let opcodes = context.artifact().byte_code;
        assert_eq!(opcodes.len(), 5);
    }

    #[test]
    fn test_loop_with_branch() {
        let movements = vec![(3, 1), (1, 2), (2, 3), (1, 4), (4, 5)];
        let mut context = create_context();

        let (sources, destinations) = movements_to_source_and_destinations(movements);

        context.codegen_mov_registers_to_registers(sources, destinations);
        let opcodes = context.artifact().byte_code;
        assert_eq!(opcodes.len(), 5);
    }

    #[test]
    fn test_two_loops() {
        let movements = vec![(3, 1), (1, 2), (2, 3), (6, 4), (4, 5), (5, 6)];

        let mut context = create_context();
        for _ in 0..movements.len() {
            context.registers_mut().allocate_register();
        }
        let (sources, destinations) = movements_to_source_and_destinations(movements);

        context.codegen_mov_registers_to_registers(sources, destinations);
        let opcodes = context.artifact().byte_code;
        assert_eq!(opcodes.len(), 8);
    }

    #[test]
    fn test_another_loop_with_branch() {
        let movements = vec![(2, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
        let mut context = create_context();

        let (sources, destinations) = movements_to_source_and_destinations(movements);

        context.codegen_mov_registers_to_registers(sources, destinations);
        let opcodes = context.artifact().byte_code;
        assert_eq!(
            opcodes,
            vec![
                Opcode::Mov {
                    destination: MemoryAddress::relative(5),
                    source: MemoryAddress::relative(4)
                },
                Opcode::Mov {
                    destination: MemoryAddress::relative(4),
                    source: MemoryAddress::relative(3)
                },
                Opcode::Mov {
                    destination: MemoryAddress::relative(3),
                    source: MemoryAddress::relative(2)
                },
                Opcode::Mov {
                    destination: MemoryAddress::relative(2),
                    source: MemoryAddress::relative(1)
                },
                Opcode::Mov {
                    destination: MemoryAddress::relative(1),
                    source: MemoryAddress::relative(3)
                }
            ]
        );
    }
    #[test]
    fn test_one_loop() {
        let movements = vec![(2, 1), (4, 2), (5, 3), (3, 4), (1, 5)];
        let mut context = create_context();
        for _ in 0..movements.len() {
            context.registers_mut().allocate_register();
        }
        let (sources, destinations) = movements_to_source_and_destinations(movements);

        context.codegen_mov_registers_to_registers(sources, destinations);
        let opcodes = context.artifact().byte_code;
        assert_eq!(
            opcodes,
            vec![
                Opcode::Mov {
                    destination: MemoryAddress::relative(6),
                    source: MemoryAddress::relative(1)
                },
                Opcode::Mov {
                    destination: MemoryAddress::relative(1),
                    source: MemoryAddress::relative(2)
                },
                Opcode::Mov {
                    destination: MemoryAddress::relative(2),
                    source: MemoryAddress::relative(4)
                },
                Opcode::Mov {
                    destination: MemoryAddress::relative(4),
                    source: MemoryAddress::relative(3)
                },
                Opcode::Mov {
                    destination: MemoryAddress::relative(3),
                    source: MemoryAddress::relative(5)
                },
                Opcode::Mov {
                    destination: MemoryAddress::relative(5),
                    source: MemoryAddress::relative(6)
                }
            ]
        );
    }
}
