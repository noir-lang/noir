use acvm::{AcirField, FieldElement};
use iter_extended::vecmap;
use noirc_errors::Location;

use crate::{
    Type, TypeBindings,
    ast::IntegerBitSize,
    monomorphization::{
        Monomorphizer,
        ast::{self, Definition, FuncId, Function, InlineType},
        errors::MonomorphizationError,
    },
    node_interner::{self, ExprId},
    shared::{Signedness, Visibility},
    signed_field::SignedField,
    token::FmtStrFragment,
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
    /// Try to evaluate certain builtin functions (just the function itself) given their type.
    /// All builtins are function types, so the evaluated result will always be a new function or None.
    ///
    /// Prerequisite: `typ = typ.follow_bindings()`
    ///          and: `turbofish_generics = vecmap(turbofish_generics, Type::follow_bindings)`
    pub(super) fn try_evaluate_builtin(
        &mut self,
        opcode_string: &str,
        typ: Type,
        turbofish_generics: Vec<Type>,
        is_unconstrained: bool,
        id: node_interner::FuncId,
        location: Location,
    ) -> Result<Option<FuncId>, MonomorphizationError> {
        let Some(opcode) = HandledOpcode::parse(opcode_string) else { return Ok(None) };

        let (parameter_types, return_type) = match &typ {
            Type::Function(parameters, ret, _, _) => (parameters, ret),
            other => unreachable!("Expected built-in to be a function, found {other:?}"),
        };

        let converted_return_type = Self::convert_type(return_type, location)?;

        let mut parameters = Vec::new();
        let body = match opcode {
            HandledOpcode::CheckedTransmute => {
                assert_eq!(parameter_types.len(), 1);
                let parameter_id = self.next_local_id();
                let parameter_type = Self::convert_type(&parameter_types[0], location)?;
                parameters = vec![(
                    parameter_id,
                    false,
                    "x".to_string(),
                    parameter_type.clone(),
                    Visibility::Private,
                )];

                self.check_transmute(&parameter_types[0], return_type, location)?;

                ast::Expression::Ident(ast::Ident {
                    location: Some(location),
                    definition: Definition::Local(parameter_id),
                    mutable: false,
                    name: "x".to_string(),
                    typ: parameter_type,
                    id: self.next_ident_id(),
                })
            }
            HandledOpcode::ModulusBeBits => self.modulus_be_bits(location),
            HandledOpcode::ModulusBeBytes => self.modulus_be_bytes(location),
            HandledOpcode::ModulusLeBits => self.modulus_le_bits(location),
            HandledOpcode::ModulusLeBytes => self.modulus_le_bytes(location),
            HandledOpcode::ModulusNumBits => Self::modulus_num_bits(location),
            HandledOpcode::Zeroed => self.zeroed_value_of_type(&converted_return_type, location),
        };

        let new_function_id = self.next_function_id();
        self.push_function(
            new_function_id,
            Function {
                id: new_function_id,
                name: opcode_string.to_string(),
                parameters,
                body,
                return_type: converted_return_type,
                return_visibility: Visibility::Private,
                unconstrained: is_unconstrained,
                inline_type: InlineType::InlineAlways,
                is_entry_point: false,
            },
        );
        self.define_function(id, typ, turbofish_generics, is_unconstrained, new_function_id);
        Ok(Some(new_function_id))
    }

    /// Check the given type conversion that the types are equal, or issue an error if not.
    fn check_transmute(
        &mut self,
        actual: &Type,
        expected: &Type,
        location: Location,
    ) -> Result<(), MonomorphizationError> {
        if actual.try_unify(expected, &mut TypeBindings::default()).is_ok() {
            Ok(())
        } else {
            let actual = actual.to_string();
            let expected = expected.to_string();
            Err(MonomorphizationError::CheckedTransmuteFailed { actual, expected, location })
        }
    }

    fn modulus_vector_literal(
        &self,
        bytes: Vec<u8>,
        arr_elem_bits: IntegerBitSize,
        location: Location,
    ) -> ast::Expression {
        use ast::*;

        let int_type = Type::Integer(Signedness::Unsigned, arr_elem_bits);

        let bytes_as_expr = vecmap(bytes, |byte| {
            let value = SignedField::positive(u32::from(byte));
            Expression::Literal(Literal::Integer(value, int_type.clone(), location))
        });

        let typ = Type::Vector(Box::new(int_type));
        let arr_literal = ArrayLiteral { typ, contents: bytes_as_expr };
        Expression::Literal(Literal::Vector(arr_literal))
    }

    /// Implements `std::mem::zeroed` by returning an appropriate zeroed
    /// ast literal or collection node for the given type. Note that for functions
    /// there is no obvious zeroed value so this should be considered unsafe to use.
    pub(super) fn zeroed_value_of_type(
        &mut self,
        typ: &ast::Type,
        location: Location,
    ) -> ast::Expression {
        match typ {
            ast::Type::Field | ast::Type::Integer(..) => {
                let typ = typ.clone();
                let zero = SignedField::positive(0u32);
                ast::Expression::Literal(ast::Literal::Integer(zero, typ, location))
            }
            ast::Type::Bool => ast::Expression::Literal(ast::Literal::Bool(false)),
            ast::Type::Unit => ast::Expression::Literal(ast::Literal::Unit),
            ast::Type::Array(length, element_type) => {
                let element = self.zeroed_value_of_type(element_type.as_ref(), location);
                ast::Expression::Literal(ast::Literal::Array(ast::ArrayLiteral {
                    contents: vec![element; *length as usize],
                    typ: ast::Type::Array(*length, element_type.clone()),
                }))
            }
            ast::Type::String(length) => {
                ast::Expression::Literal(ast::Literal::Str("\0".repeat(*length as usize)))
            }
            ast::Type::FmtString(length, fields) => {
                let zeroed_tuple = self.zeroed_value_of_type(fields, location);
                let fields_len = match &zeroed_tuple {
                    ast::Expression::Tuple(fields) => fields.len() as u64,
                    _ => unreachable!(
                        "ICE: format string fields should be structured in a tuple, but got a {zeroed_tuple}"
                    ),
                };
                ast::Expression::Literal(ast::Literal::FmtStr(
                    vec![FmtStrFragment::String("\0".repeat(*length as usize))],
                    fields_len,
                    Box::new(zeroed_tuple),
                ))
            }
            ast::Type::Tuple(fields) => ast::Expression::Tuple(vecmap(fields, |field| {
                self.zeroed_value_of_type(field, location)
            })),
            ast::Type::Function(parameter_types, ret_type, env, unconstrained) => self
                .create_zeroed_function(parameter_types, ret_type, env, *unconstrained, location),
            ast::Type::Vector(element_type) => {
                ast::Expression::Literal(ast::Literal::Vector(ast::ArrayLiteral {
                    contents: vec![],
                    typ: ast::Type::Vector(element_type.clone()),
                }))
            }
            ast::Type::Reference(element, mutable) => {
                let rhs = Box::new(self.zeroed_value_of_type(element, location));
                let result_type = typ.clone();
                ast::Expression::Unary(ast::Unary {
                    rhs,
                    result_type,
                    operator: super::UnaryOp::Reference { mutable: *mutable },
                    location,
                    skip: false,
                })
            }
        }
    }

    // Creating a zeroed function value is almost always an error if it is used later,
    // Hence why std::unsafe_func::zeroed is unsafe.
    //
    // To avoid confusing later passes, we arbitrarily choose to construct a function
    // that satisfies the input type by discarding all its parameters and returning a
    // zeroed value of the result type.
    fn create_zeroed_function(
        &mut self,
        parameter_types: &[ast::Type],
        ret_type: &ast::Type,
        env_type: &ast::Type,
        unconstrained: bool,
        location: Location,
    ) -> ast::Expression {
        let lambda_name = "zeroed_lambda";

        let parameters = vecmap(parameter_types, |parameter_type| {
            (self.next_local_id(), false, "_".into(), parameter_type.clone(), Visibility::Private)
        });

        let body = self.zeroed_value_of_type(ret_type, location);

        let id = self.next_function_id();
        let return_type = ret_type.clone();
        let name = lambda_name.to_owned();

        let function = Function {
            id,
            name,
            parameters,
            body,
            return_type,
            return_visibility: Visibility::Private,
            unconstrained,
            inline_type: InlineType::default(),
            is_entry_point: false,
        };
        self.push_function(id, function);

        ast::Expression::Ident(ast::Ident {
            definition: Definition::Function(id),
            mutable: false,
            location: None,
            name: lambda_name.to_owned(),
            typ: ast::Type::Function(
                parameter_types.to_owned(),
                Box::new(ret_type.clone()),
                Box::new(env_type.clone()),
                unconstrained,
            ),
            id: self.next_ident_id(),
        })
    }

    /// Try to call certain builtin functions with the given arguments, returning the result as an
    /// expression.
    pub(super) fn try_evaluate_builtin_call(
        &mut self,
        func: &ast::Expression,
        expr_id: &ExprId,
        arguments: &[ExprId],
        argument_values: &[ast::Expression],
        result_type: &ast::Type,
    ) -> Result<Option<ast::Expression>, MonomorphizationError> {
        if let ast::Expression::Ident(ident) = func {
            if let Definition::Builtin(opcode) = &ident.definition {
                let location = self.interner.expr_location(expr_id);

                return Ok(Some(match HandledOpcode::parse(opcode) {
                    Some(HandledOpcode::CheckedTransmute) => {
                        assert_eq!(arguments.len(), 1);
                        let parameter_type = self.interner.id_type(arguments[0]).follow_bindings();
                        let result_type = self.interner.id_type(expr_id).follow_bindings();
                        self.check_transmute(&parameter_type, &result_type, location)?;
                        argument_values[0].clone()
                    }
                    Some(HandledOpcode::ModulusBeBits) => self.modulus_be_bits(location),
                    Some(HandledOpcode::ModulusBeBytes) => self.modulus_be_bytes(location),
                    Some(HandledOpcode::ModulusLeBits) => self.modulus_le_bits(location),
                    Some(HandledOpcode::ModulusLeBytes) => self.modulus_le_bytes(location),
                    Some(HandledOpcode::ModulusNumBits) => Self::modulus_num_bits(location),
                    Some(HandledOpcode::Zeroed) => self.zeroed_value_of_type(result_type, location),
                    None => return Ok(None),
                }));
            }
        }
        Ok(None)
    }

    fn modulus_be_bits(&self, location: Location) -> ast::Expression {
        let bits = FieldElement::modulus().to_radix_be(2);
        self.modulus_vector_literal(bits, IntegerBitSize::One, location)
    }

    fn modulus_be_bytes(&self, location: Location) -> ast::Expression {
        let bytes = FieldElement::modulus().to_bytes_be();
        self.modulus_vector_literal(bytes, IntegerBitSize::Eight, location)
    }

    fn modulus_le_bits(&self, location: Location) -> ast::Expression {
        let bits = FieldElement::modulus().to_radix_le(2);
        self.modulus_vector_literal(bits, IntegerBitSize::One, location)
    }

    fn modulus_le_bytes(&self, location: Location) -> ast::Expression {
        let bytes = FieldElement::modulus().to_bytes_le();
        self.modulus_vector_literal(bytes, IntegerBitSize::Eight, location)
    }

    fn modulus_num_bits(location: Location) -> ast::Expression {
        let bits = FieldElement::max_num_bits();
        let typ = ast::Type::Integer(Signedness::Unsigned, IntegerBitSize::SixtyFour);
        let bits = SignedField::positive(bits);
        ast::Expression::Literal(ast::Literal::Integer(bits, typ, location))
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
