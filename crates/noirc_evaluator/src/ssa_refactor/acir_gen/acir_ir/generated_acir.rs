//! `GeneratedAcir` is constructed as part of the `acir_gen` pass to accumulate all of the ACIR
//! program as it is being converted from SSA form.
use super::errors::AcirGenError;
use acvm::acir::{
    circuit::opcodes::{BlackBoxFuncCall, FunctionInput, Opcode as AcirOpcode},
    native_types::Witness,
};
use acvm::{
    acir::{circuit::directives::Directive, native_types::Expression},
    FieldElement,
};

#[derive(Debug, Default)]
pub(crate) struct OpcodeRegionLabel {
    /// Range of the opcodes included in the region
    start_range: usize,
    /// The end range is `None` while its on the region_stack
    /// and the caller has not finalized the region.
    end_range_inclusive: Option<usize>,

    /// Label to apply to this region of code
    label: String,
}

#[derive(Debug, Default)]
/// The output of the Acir-gen pass
pub(crate) struct GeneratedAcir {
    /// The next witness index that may be declared.
    ///
    /// Equivalent to acvm::acir::circuit::Circuit's field of the same name.
    pub(crate) current_witness_index: u32,

    /// The opcodes of which the compiled ACIR will comprise.
    pub(crate) opcodes: Vec<AcirOpcode>,

    /// All witness indices that comprise the final return value of the program
    ///
    /// Note: This may contain repeated indices, which is necessary for later mapping into the
    /// abi's return type.
    pub(crate) return_witnesses: Vec<Witness>,

    /// For debugging purposes, one can label blocks of the opcode.
    finalized_regions: Vec<OpcodeRegionLabel>,
    region_stack: Vec<OpcodeRegionLabel>,
}

impl GeneratedAcir {
    pub(crate) fn start_region_label(&mut self, region_name: String) {
        self.region_stack.push(OpcodeRegionLabel {
            start_range: self.opcodes.len(),
            end_range_inclusive: None,
            label: region_name,
        })
    }
    pub(crate) fn end_label(&mut self) {
        let mut region_label = self.region_stack.pop().expect("tried to pop a region label from the stack without first pushing a region onto the stack");
        region_label.end_range_inclusive = Some(self.opcodes.len());
        self.finalized_regions.push(region_label)
    }

    pub(crate) fn print_acir(self) -> Self {
        fn check_if_region_starting(
            index: usize,
            regions: &[OpcodeRegionLabel],
        ) -> Vec<&OpcodeRegionLabel> {
            regions.into_iter().filter(|region| region.start_range == index).collect()
        }
        fn check_if_region_ending(
            index: usize,
            regions: &[OpcodeRegionLabel],
        ) -> Vec<&OpcodeRegionLabel> {
            regions
                .into_iter()
                .filter(|region| {
                    region.end_range_inclusive.expect("region has not been finalized") == index
                })
                .collect()
        }

        for (index, opcode) in self.opcodes.iter().enumerate() {
            let regions_starting = check_if_region_starting(index, &self.finalized_regions);
            let regions_ending = check_if_region_ending(index, &self.finalized_regions);

            for region in regions_starting {
                println!("region start: {}", region.label)
            }

            println!("OPCODE : {}", opcode);

            for region in regions_ending {
                println!("region end: {}", region.label)
            }
        }

        self
    }

    /// Returns the current witness index.
    pub(crate) fn current_witness_index(&self) -> Witness {
        Witness(self.current_witness_index)
    }

    /// Adds a new opcode into ACIR.
    fn push_opcode(&mut self, opcode: AcirOpcode) {
        self.opcodes.push(opcode);
    }

    /// Updates the witness index counter and returns
    /// the next witness index.
    pub(crate) fn next_witness_index(&mut self) -> Witness {
        self.current_witness_index += 1;
        Witness(self.current_witness_index)
    }

    /// Converts an expression into a Witness.
    ///
    /// This is done by creating a new Witness and creating an opcode which
    /// sets the Witness to be equal to the expression.
    ///
    /// The reason we do this is because _constraints_ in ACIR have a degree limit
    /// This means you cannot multiply an infinite amount of Expressions together.
    /// Once the expression goes over degree-2, then it needs to be reduced to a Witness
    /// which has degree-1 in order to be able to continue the multiplication chain.
    pub(crate) fn expression_to_witness(&mut self, expression: &Expression) -> Witness {
        let fresh_witness = self.next_witness_index();

        // Create a constraint that sets them to be equal to each other
        // Then return the witness as this can now be used in places
        // where we would have used the Witness.
        let constraint = expression - fresh_witness;
        // This assertion means that verification of this
        // program will fail if expression != witness.
        //
        // This is because we have:
        //  => constraint == 0
        //  => expression - fresh_witness == 0
        //  => expression == fresh_witness
        self.assert_is_zero(constraint);

        fresh_witness
    }

    /// Adds a witness index to the program's return witnesses.
    pub(crate) fn push_return_witness(&mut self, witness: Witness) {
        self.return_witnesses.push(witness);
    }
}

impl GeneratedAcir {
    /// Adds an inversion directive.
    ///
    /// This directive will invert `expr` without applying constraints
    /// and return a `Witness` which may or may not be the result of
    /// inverting `expr`.
    ///
    /// Safety: It is the callers responsibility to ensure that the
    /// resulting `Witness` is constrained to be the inverse.
    pub(crate) fn directive_inverse(&mut self, expr: &Expression) -> Witness {
        // The inversion directive requires that
        // the inputs be Witness, so we need this potential extra
        // reduction constraint.
        // Note: changing this in ACIR would allow us to remove it
        let witness = match expr.to_witness() {
            Some(witness) => witness,
            None => self.expression_to_witness(expr),
        };

        // Create the witness for the result
        let inverted_witness = self.next_witness_index();

        self.push_opcode(AcirOpcode::Directive(Directive::Invert {
            x: witness,
            result: inverted_witness,
        }));

        inverted_witness
    }

    /// Asserts `expr` to be zero.
    ///
    /// If `expr` is not zero, then the constraint system will
    /// fail upon verification.
    pub(crate) fn assert_is_zero(&mut self, expr: Expression) {
        self.push_opcode(AcirOpcode::Arithmetic(expr));
    }

    /// Adds a constraint which ensure thats `witness` is an
    /// integer within the range [0, 2^{num_bits} - 1]
    pub(crate) fn range_constraint(
        &mut self,
        witness: Witness,
        num_bits: u32,
    ) -> Result<(), AcirGenError> {
        if num_bits == FieldElement::max_num_bits() {
            return Err(AcirGenError::InvalidRangeConstraint {
                num_bits: FieldElement::max_num_bits(),
            });
        };

        let constraint = AcirOpcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
            input: FunctionInput { witness, num_bits },
        });
        self.push_opcode(constraint);

        Ok(())
    }
}
