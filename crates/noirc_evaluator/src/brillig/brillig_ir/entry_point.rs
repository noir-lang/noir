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
            // Make sure we don't overwrite the arguments
            self.allocate_register();
        }

        // Calculate the initial value for the stack pointer register
        let size_arguments_memory: usize = arguments
            .iter()
            .map(|arg| match arg {
                BrilligParameter::Simple => 0,
                _ => BrilligContext::flattened_size(arg),
            })
            .sum();

        // Set the initial value of the stack pointer register
        self.push_opcode(BrilligOpcode::Const {
            destination: ReservedRegisters::stack_pointer(),
            value: size_arguments_memory.into(),
        });
        // Set the initial value of the previous stack pointer register
        self.push_opcode(BrilligOpcode::Const {
            destination: ReservedRegisters::previous_stack_pointer(),
            value: 0_usize.into(),
        });

        for (index, parameter) in arguments.iter().enumerate() {
            if let BrilligParameter::Array(item_type, item_count) = parameter {
                if item_type.iter().any(|param| !matches!(param, BrilligParameter::Simple)) {
                    let pointer_register = ReservedRegisters::user_register_index(index);
                    let deflattened_register =
                        self.deflatten_array(item_type, *item_count, pointer_register);
                    self.mov_instruction(pointer_register, deflattened_register);
                }
            }
        }
    }

    fn flattened_size(param: &BrilligParameter) -> usize {
        match param {
            BrilligParameter::Simple => 1,
            BrilligParameter::Array(item_types, item_count) => {
                let item_size: usize = item_types.iter().map(BrilligContext::flattened_size).sum();
                item_count * item_size
            }
            BrilligParameter::Slice(_) => {
                unreachable!("ICE: Slices cannot be passed as entry point arguments")
            }
        }
    }

    fn deflatten_array(
        &mut self,
        item_type: &[BrilligParameter],
        item_count: usize,
        flattened_array_pointer: RegisterIndex,
    ) -> RegisterIndex {
        let movement_register = self.allocate_register();
        let deflattened_array_pointer = self.allocate_register();

        let target_item_size = item_type.len();
        let source_item_size: usize = item_type.iter().map(BrilligContext::flattened_size).sum();

        self.allocate_fixed_length_array(deflattened_array_pointer, item_count * target_item_size);

        for item_index in 0..item_count {
            let source_item_base_index = item_index * source_item_size;
            let target_item_base_index = item_index * target_item_size;

            let mut source_offset = 0;

            for (subitem_index, subitem) in item_type.iter().enumerate() {
                let source_index =
                    self.make_constant((source_item_base_index + source_offset).into());

                let target_index =
                    self.make_constant((target_item_base_index + subitem_index).into());

                match subitem {
                    BrilligParameter::Simple => {
                        self.array_get(flattened_array_pointer, source_index, movement_register);
                        self.array_set(deflattened_array_pointer, target_index, movement_register);
                        source_offset += 1;
                    }
                    BrilligParameter::Array(subarray_item_type, subarray_item_count) => {
                        let nested_array_pointer = self.allocate_register();
                        self.mov_instruction(nested_array_pointer, flattened_array_pointer);
                        self.memory_op(
                            nested_array_pointer,
                            source_index,
                            nested_array_pointer,
                            acvm::brillig_vm::brillig::BinaryIntOp::Add,
                        );
                        let deflattened_subarray_pointer = self.deflatten_array(
                            subarray_item_type,
                            *subarray_item_count,
                            nested_array_pointer,
                        );
                        self.array_set(
                            deflattened_array_pointer,
                            target_index,
                            deflattened_subarray_pointer,
                        );
                        source_offset += BrilligContext::flattened_size(subitem);
                    }
                    BrilligParameter::Slice(..) => unreachable!("ICE: Cannot deflatten slices"),
                }
            }
        }

        // TODO deallocate registers

        deflattened_array_pointer
    }

    /// Adds the instructions needed to handle return parameters
    fn exit_point_instruction(&mut self, return_parameters: Vec<BrilligParameter>) {
        // Make sure we don't overwrite the return parameters
        return_parameters.iter().for_each(|_| {
            self.allocate_register();
        });

        for (index, ret) in return_parameters.iter().enumerate() {
            if let BrilligParameter::Array(item_type, item_count) = ret {
                if item_type.iter().any(|item| !matches!(item, BrilligParameter::Simple)) {
                    let returned_pointer = ReservedRegisters::user_register_index(index);
                    let flattened_array_pointer = self.allocate_register();

                    self.allocate_fixed_length_array(
                        flattened_array_pointer,
                        BrilligContext::flattened_size(ret),
                    );

                    self.flatten_array(
                        item_type,
                        *item_count,
                        flattened_array_pointer,
                        returned_pointer,
                    );

                    self.mov_instruction(returned_pointer, flattened_array_pointer);
                }
            }
        }
        // We want all functions to follow the calling convention of returning
        // their results in the first `n` registers. So we to move the return values
        // to the first `n` registers once completed.

        // Move the results to registers 0..n
        for i in 0..return_parameters.len() {
            self.push_opcode(BrilligOpcode::Mov {
                destination: i.into(),
                source: ReservedRegisters::user_register_index(i),
            });
        }
        self.push_opcode(BrilligOpcode::Stop);
    }

    fn flatten_array(
        &mut self,
        item_type: &[BrilligParameter],
        item_count: usize,
        flattened_array_pointer: RegisterIndex,
        deflattened_array_pointer: RegisterIndex,
    ) {
        let movement_register = self.allocate_register();

        let source_item_size = item_type.len();
        let target_item_size: usize = item_type.iter().map(BrilligContext::flattened_size).sum();

        for item_index in 0..item_count {
            let source_item_base_index = item_index * source_item_size;
            let target_item_base_index = item_index * target_item_size;

            let mut target_offset = 0;

            for (subitem_index, subitem) in item_type.iter().enumerate() {
                let source_index =
                    self.make_constant((source_item_base_index + subitem_index).into());
                let target_index =
                    self.make_constant((target_item_base_index + target_offset).into());

                match subitem {
                    BrilligParameter::Simple => {
                        self.array_get(deflattened_array_pointer, source_index, movement_register);
                        self.array_set(flattened_array_pointer, target_index, movement_register);
                        target_offset += 1;
                    }
                    BrilligParameter::Array(subarray_item_type, subarray_item_count) => {
                        let nested_array_pointer = self.allocate_register();
                        self.array_get(
                            deflattened_array_pointer,
                            source_index,
                            nested_array_pointer,
                        );

                        let flattened_subarray_pointer = self.allocate_register();

                        self.mov_instruction(flattened_subarray_pointer, flattened_array_pointer);

                        self.memory_op(
                            flattened_subarray_pointer,
                            target_index,
                            flattened_subarray_pointer,
                            acvm::brillig_vm::brillig::BinaryIntOp::Add,
                        );

                        self.flatten_array(
                            subarray_item_type,
                            *subarray_item_count,
                            flattened_subarray_pointer,
                            nested_array_pointer,
                        );

                        target_offset += BrilligContext::flattened_size(subitem);
                    }
                    BrilligParameter::Slice(..) => unreachable!("ICE: Cannot flatten slices"),
                }
            }
        }

        // TODO deallocate registers
    }
}

#[cfg(test)]
mod tests {
    use acvm::brillig_vm::brillig::{RegisterIndex, Value};

    use crate::brillig::brillig_ir::{
        artifact::BrilligParameter,
        tests::{create_and_run_vm, create_context},
    };

    #[test]
    fn entry_point_with_nested_array_parameter() {
        let flattened_array = vec![
            Value::from(1_usize),
            Value::from(2_usize),
            Value::from(3_usize),
            Value::from(4_usize),
            Value::from(5_usize),
            Value::from(6_usize),
        ];
        let arguments = vec![BrilligParameter::Array(
            vec![
                BrilligParameter::Array(vec![BrilligParameter::Simple], 2),
                BrilligParameter::Simple,
            ],
            2,
        )];
        let returns = vec![BrilligParameter::Simple];

        let mut context = create_context();

        // Allocate the parameter
        let array_pointer = context.allocate_register();

        context.return_instruction(&[array_pointer]);

        let vm = create_and_run_vm(
            flattened_array.clone(),
            vec![Value::from(0_usize)],
            context,
            arguments,
            returns,
        );
        let memory = vm.get_memory();

        assert_eq!(vm.get_registers().get(RegisterIndex(0)), Value::from(flattened_array.len()));
        assert_eq!(
            memory,
            &vec![
                // The original flattened values
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(4_usize),
                Value::from(5_usize),
                Value::from(6_usize),
                // The pointer to the subarray of the first item
                Value::from(10_usize),
                Value::from(3_usize),
                // The pointer to the subarray of the second item
                Value::from(12_usize),
                Value::from(6_usize),
                // The subarray of the first item
                Value::from(1_usize),
                Value::from(2_usize),
                // The subarray of the second item
                Value::from(4_usize),
                Value::from(5_usize),
            ]
        );
    }

    #[test]
    fn entry_point_with_nested_array_return() {
        let flattened_array = vec![
            Value::from(1_usize),
            Value::from(2_usize),
            Value::from(3_usize),
            Value::from(4_usize),
            Value::from(5_usize),
            Value::from(6_usize),
        ];

        let array_param = BrilligParameter::Array(
            vec![
                BrilligParameter::Simple,
                BrilligParameter::Array(vec![BrilligParameter::Simple], 2),
            ],
            2,
        );

        let arguments = vec![array_param.clone()];
        let returns = vec![array_param];

        let mut context = create_context();

        // Allocate the parameter
        let array_pointer = context.allocate_register();

        context.return_instruction(&[array_pointer]);

        let vm = create_and_run_vm(
            flattened_array.clone(),
            vec![Value::from(0_usize)],
            context,
            arguments,
            returns,
        );
        let memory = vm.get_memory();

        assert_eq!(
            vm.get_registers().get(RegisterIndex(0)),
            // The returned value will be past the original array and the deflattened array
            Value::from(flattened_array.len() + (flattened_array.len() + 2)),
        );

        assert_eq!(
            memory,
            &vec![
                // The original flattened values
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(4_usize),
                Value::from(5_usize),
                Value::from(6_usize),
                // The pointer to the subarray of the first item
                Value::from(1_usize),
                Value::from(10_usize),
                // The pointer to the subarray of the second item
                Value::from(4_usize),
                Value::from(12_usize),
                // The subarray of the first item
                Value::from(2_usize),
                Value::from(3_usize),
                // The subarray of the second item
                Value::from(5_usize),
                Value::from(6_usize),
                // The values flattened again
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(4_usize),
                Value::from(5_usize),
                Value::from(6_usize),
            ]
        );
    }
}
