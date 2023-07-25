use crate::brillig::brillig_ir::ReservedRegisters;

use super::{
    artifact::{BrilligArtifact, BrilligParameter},
    debug_show::DebugShow,
    registers::BrilligRegistersContext,
    BrilligContext,
};
use acvm::acir::brillig::{Opcode as BrilligOpcode, RegisterIndex};

impl BrilligContext {
    /// Creates an entry point artifact that will jump to the function label provided.
    pub(crate) fn new_entry_point_artifact<T: ToString>(
        arguments: Vec<BrilligParameter>,
        return_parameters: Vec<BrilligParameter>,
        target_function: T,
    ) -> BrilligArtifact {
        let mut context = BrilligContext {
            obj: BrilligArtifact::default(),
            registers: BrilligRegistersContext::new(),
            context_label: String::default(),
            section_label: 0,
            debug_show: DebugShow::new(false),
        };

        context.entry_point_instruction(arguments);

        context.add_external_call_instruction(target_function);

        context.exit_point_instruction(return_parameters);
        context.artifact()
    }

    /// Adds the instructions needed to handle entry point parameters
    ///
    /// And sets the starting value of the reserved registers
    fn entry_point_instruction(&mut self, arguments: Vec<BrilligParameter>) {
        // Translate the inputs by the reserved registers offset
        for i in (0..arguments.len()).rev() {
            self.push_opcode(BrilligOpcode::Mov {
                destination: ReservedRegisters::user_register_index(i),
                source: RegisterIndex::from(i),
            });
        }

        fn memory_usage(param: &BrilligParameter, is_array_item: bool) -> usize {
            match param {
                // Only occupies memory if it's an array item
                BrilligParameter::Simple => usize::from(is_array_item),
                BrilligParameter::Array(item_types, item_count) => {
                    let item_size: usize =
                        item_types.iter().map(|item_type| memory_usage(item_type, true)).sum();
                    item_count * item_size
                }
                BrilligParameter::Slice(_) => {
                    unreachable!("ICE: Heap vectors cannot be passed as entry point arguments")
                }
            }
        }

        // Calculate the initial value for the stack pointer register
        let size_arguments_memory: usize =
            arguments.iter().map(|arg| memory_usage(arg, false)).sum();
        println!("Initial stack pointer value: {}", size_arguments_memory);
        // Set the initial value of the stack pointer register
        self.push_opcode(BrilligOpcode::Const {
            destination: ReservedRegisters::stack_pointer(),
            value: size_arguments_memory.into(),
        });
    }

    /// Adds the instructions needed to handle return parameters
    fn exit_point_instruction(&mut self, return_parameters: Vec<BrilligParameter>) {
        // We want all functions to follow the calling convention of returning
        // their results in the first `n` registers. So we modify the bytecode of the
        // function to move the return values to the first `n` registers once completed.

        // Move the results to registers 0..n
        for i in 0..return_parameters.len() {
            self.push_opcode(BrilligOpcode::Mov {
                destination: i.into(),
                source: ReservedRegisters::user_register_index(i),
            });
        }
        self.push_opcode(BrilligOpcode::Stop);
    }
}
