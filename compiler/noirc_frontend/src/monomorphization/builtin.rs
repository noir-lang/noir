use acvm::{AcirField, FieldElement};
use noirc_errors::Location;

use crate::{
    Type,
    ast::IntegerBitSize,
    monomorphization::{
        Monomorphizer,
        ast::{self, FuncId, Function},
        errors::MonomorphizationError,
    },
    shared::{Signedness, Visibility},
    signed_field::SignedField,
};

/// These are the opcodes which the monomorphizer can replace with functions.
/// Any opcode not included is forwarded to SSA as a built-in function.
enum HandledOpcode {
    CheckedTransmute,
    ModulusBeBits,
    ModulusBeBytes,
    ModulusLeBits,
    ModulusLeBytes,
    ModulusNumBits,
    Zeroed,
}

impl Monomorphizer<'_> {
    /// Try to evaluate certain builtin functions given their type. All builtins are function
    /// types, so the evaluated result will always be a new function or None.
    pub(super) fn try_evaluate_builtin(
        &mut self,
        opcode_string: &str,
        typ: &Type,
        is_unconstrained: bool,
        location: Location,
    ) -> Result<Option<FuncId>, MonomorphizationError> {
        let Some(opcode) = HandledOpcode::parse(opcode_string) else { return Ok(None) };

        // Monomorphized function types are paires of (constrained, unconstrained) individual function types.
        let (parameter_types, return_type) = match Self::convert_type(typ, location)? {
            ast::Type::Tuple(mut fields) if fields.len() == 2 => match fields.pop().unwrap() {
                ast::Type::Function(parameters, ret, _, _) => (parameters, *ret),
                other => unreachable!("Expected built-in to be a function, found {other:?}"),
            },
            other => unreachable!("Expected built-in to be a function, found {other:?}"),
        };

        let (parameters, body) = match opcode {
            HandledOpcode::CheckedTransmute => {
                assert_eq!(parameter_types.len(), 1);
                let parameter_id = self.next_local_id();
                let parameters = vec![(
                    parameter_id,
                    false,
                    "x".to_string(),
                    parameter_types[0].clone(),
                    Visibility::Private,
                )];

                let body = self.checked_transmute(
                    parameter_id,
                    &parameter_types[0],
                    &return_type,
                    location,
                )?;
                (parameters, body)
            }
            HandledOpcode::ModulusBeBits => {
                let bits = FieldElement::modulus().to_radix_be(2);
                (Vec::new(), self.modulus_vector_literal(bits, IntegerBitSize::One, location))
            }
            HandledOpcode::ModulusBeBytes => {
                let bytes = FieldElement::modulus().to_bytes_be();
                (Vec::new(), self.modulus_vector_literal(bytes, IntegerBitSize::Eight, location))
            }
            HandledOpcode::ModulusLeBits => {
                let bits = FieldElement::modulus().to_radix_le(2);
                (Vec::new(), self.modulus_vector_literal(bits, IntegerBitSize::One, location))
            }
            HandledOpcode::ModulusLeBytes => {
                let bytes = FieldElement::modulus().to_bytes_le();
                (Vec::new(), self.modulus_vector_literal(bytes, IntegerBitSize::Eight, location))
            }
            HandledOpcode::ModulusNumBits => {
                let bits = FieldElement::max_num_bits();
                let typ = ast::Type::Integer(Signedness::Unsigned, IntegerBitSize::SixtyFour);
                let bits = SignedField::positive(bits);
                (Vec::new(), ast::Expression::Literal(ast::Literal::Integer(bits, typ, location)))
            }
            HandledOpcode::Zeroed => {
                (Vec::new(), self.zeroed_value_of_type(&return_type, location))
            }
        };

        let new_function_id = self.next_function_id();
        self.push_function(
            new_function_id,
            Function {
                id: new_function_id,
                name: opcode_string.to_string(),
                parameters,
                body,
                return_type,
                return_visibility: Visibility::Private,
                unconstrained: is_unconstrained,
                inline_type: ast::InlineType::InlineAlways,
                func_sig: Default::default(),
            },
        );
        Ok(Some(new_function_id))
    }
}

impl HandledOpcode {
    fn parse(opcode: &str) -> Option<Self> {
        match opcode {
            "checked_transmute" => Some(Self::CheckedTransmute),
            "modulus_be_bits" => Some(Self::ModulusBeBits),
            "modulus_be_bytes" => Some(Self::ModulusBeBytes),
            "modulus_le_bits" => Some(Self::ModulusLeBits),
            "modulus_le_bytes" => Some(Self::ModulusLeBytes),
            "modulus_num_bits" => Some(Self::ModulusNumBits),
            "zeroed" => Some(Self::Zeroed),
            _ => None,
        }
    }
}
