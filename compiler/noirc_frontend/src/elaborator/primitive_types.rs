//! Primitive type definitions

use iter_extended::vecmap;
use noirc_errors::Location;

use crate::{
    QuotedType, Type,
    ast::{GenericTypeArgs, IntegerBitSize},
    elaborator::{Elaborator, PathResolutionMode, Turbofish, types::WildcardAllowed},
    hir::{
        def_collector::dc_crate::CompilationError,
        type_check::{
            TypeCheckError,
            generics::{FmtstrPrimitiveType, Generic as _, StrPrimitiveType},
        },
    },
    shared::Signedness,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::EnumIter)]
pub enum PrimitiveType {
    Bool,
    CtString,
    Expr,
    Field,
    Fmtstr,
    FunctionDefinition,
    I8,
    I16,
    I32,
    I64,
    U1,
    U8,
    U16,
    U32,
    U64,
    U128,
    Module,
    Quoted,
    Str,
    StructDefinition,
    TraitConstraint,
    TraitDefinition,
    TraitImpl,
    TypeDefinition,
    TypedExpr,
    Type,
    UnresolvedType,
}

impl PrimitiveType {
    pub fn lookup_by_name(name: &str) -> Option<Self> {
        match name {
            "bool" => Some(Self::Bool),
            "CtString" => Some(Self::CtString),
            "Expr" => Some(Self::Expr),
            "fmtstr" => Some(Self::Fmtstr),
            "Field" => Some(Self::Field),
            "FunctionDefinition" => Some(Self::FunctionDefinition),
            "i8" => Some(Self::I8),
            "i16" => Some(Self::I16),
            "i32" => Some(Self::I32),
            "i64" => Some(Self::I64),
            "u1" => Some(Self::U1),
            "u8" => Some(Self::U8),
            "u16" => Some(Self::U16),
            "u32" => Some(Self::U32),
            "u64" => Some(Self::U64),
            "u128" => Some(Self::U128),
            "Module" => Some(Self::Module),
            "Quoted" => Some(Self::Quoted),
            "str" => Some(Self::Str),
            "StructDefinition" => Some(Self::StructDefinition),
            "TraitConstraint" => Some(Self::TraitConstraint),
            "TraitDefinition" => Some(Self::TraitDefinition),
            "TraitImpl" => Some(Self::TraitImpl),
            "TypeDefinition" => Some(Self::TypeDefinition),
            "TypedExpr" => Some(Self::TypedExpr),
            "Type" => Some(Self::Type),
            "UnresolvedType" => Some(Self::UnresolvedType),
            _ => None,
        }
    }

    pub fn to_type(self) -> Type {
        match self {
            Self::Bool => Type::Bool,
            Self::CtString => Type::Quoted(QuotedType::CtString),
            Self::Expr => Type::Quoted(QuotedType::Expr),
            Self::Fmtstr => Type::FmtString(Box::new(Type::Error), Box::new(Type::Error)),
            Self::Field => Type::FieldElement,
            Self::FunctionDefinition => Type::Quoted(QuotedType::FunctionDefinition),
            Self::I8 => Type::Integer(Signedness::Signed, IntegerBitSize::Eight),
            Self::I16 => Type::Integer(Signedness::Signed, IntegerBitSize::Sixteen),
            Self::I32 => Type::Integer(Signedness::Signed, IntegerBitSize::ThirtyTwo),
            Self::I64 => Type::Integer(Signedness::Signed, IntegerBitSize::SixtyFour),
            Self::U1 => Type::Integer(Signedness::Unsigned, IntegerBitSize::One),
            Self::U8 => Type::Integer(Signedness::Unsigned, IntegerBitSize::Eight),
            Self::U16 => Type::Integer(Signedness::Unsigned, IntegerBitSize::Sixteen),
            Self::U32 => Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo),
            Self::U64 => Type::Integer(Signedness::Unsigned, IntegerBitSize::SixtyFour),
            Self::U128 => Type::Integer(Signedness::Unsigned, IntegerBitSize::HundredTwentyEight),
            Self::Module => Type::Quoted(QuotedType::Module),
            Self::Quoted => Type::Quoted(QuotedType::Quoted),
            Self::Str => Type::String(Box::new(Type::Error)),
            Self::TraitConstraint => Type::Quoted(QuotedType::TraitConstraint),
            Self::TraitDefinition => Type::Quoted(QuotedType::TraitDefinition),
            Self::TraitImpl => Type::Quoted(QuotedType::TraitImpl),
            Self::StructDefinition | Self::TypeDefinition => {
                Type::Quoted(QuotedType::TypeDefinition)
            }
            Self::TypedExpr => Type::Quoted(QuotedType::TypedExpr),
            Self::Type => Type::Quoted(QuotedType::Type),
            Self::UnresolvedType => Type::Quoted(QuotedType::UnresolvedType),
        }
    }

    pub fn to_integer_or_field(self) -> Option<Type> {
        match self {
            Self::I8 => Some(Type::Integer(Signedness::Signed, IntegerBitSize::Eight)),
            Self::I16 => Some(Type::Integer(Signedness::Signed, IntegerBitSize::Sixteen)),
            Self::I32 => Some(Type::Integer(Signedness::Signed, IntegerBitSize::ThirtyTwo)),
            Self::I64 => Some(Type::Integer(Signedness::Signed, IntegerBitSize::SixtyFour)),
            Self::U1 => Some(Type::Integer(Signedness::Unsigned, IntegerBitSize::One)),
            Self::U8 => Some(Type::Integer(Signedness::Unsigned, IntegerBitSize::Eight)),
            Self::U16 => Some(Type::Integer(Signedness::Unsigned, IntegerBitSize::Sixteen)),
            Self::U32 => Some(Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo)),
            Self::U64 => Some(Type::Integer(Signedness::Unsigned, IntegerBitSize::SixtyFour)),
            Self::U128 => {
                Some(Type::Integer(Signedness::Unsigned, IntegerBitSize::HundredTwentyEight))
            }
            Self::Field => Some(Type::FieldElement),
            Self::Bool
            | Self::CtString
            | Self::Expr
            | Self::Fmtstr
            | Self::FunctionDefinition
            | Self::Module
            | Self::Quoted
            | Self::Str
            | Self::StructDefinition
            | Self::TraitConstraint
            | Self::TraitDefinition
            | Self::TraitImpl
            | Self::TypeDefinition
            | Self::TypedExpr
            | Self::Type
            | Self::UnresolvedType => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::CtString => "CtString",
            Self::Expr => "Expr",
            Self::Field => "Field",
            Self::Fmtstr => "fmtstr",
            Self::FunctionDefinition => "FunctionDefinition",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::U1 => "u1",
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::U128 => "u128",
            Self::Module => "Module",
            Self::Quoted => "Quoted",
            Self::Str => "str",
            Self::StructDefinition => "StructDefinition",
            Self::TraitConstraint => "TraitConstraint",
            Self::TraitDefinition => "TraitDefinition",
            Self::TraitImpl => "TraitImpl",
            Self::TypeDefinition => "TypeDefinition",
            Self::TypedExpr => "TypedExpr",
            Self::Type => "Type",
            Self::UnresolvedType => "UnresolvedType",
        }
    }
}

impl Elaborator<'_> {
    pub(crate) fn instantiate_primitive_type(
        &mut self,
        primitive_type: PrimitiveType,
        args: GenericTypeArgs,
        location: Location,
        wildcard_allowed: WildcardAllowed,
    ) -> Type {
        match primitive_type {
            PrimitiveType::Bool
            | PrimitiveType::CtString
            | PrimitiveType::Expr
            | PrimitiveType::Field
            | PrimitiveType::FunctionDefinition
            | PrimitiveType::I8
            | PrimitiveType::I16
            | PrimitiveType::I32
            | PrimitiveType::I64
            | PrimitiveType::U1
            | PrimitiveType::U8
            | PrimitiveType::U16
            | PrimitiveType::U32
            | PrimitiveType::U64
            | PrimitiveType::U128
            | PrimitiveType::Module
            | PrimitiveType::Quoted
            | PrimitiveType::StructDefinition
            | PrimitiveType::TraitConstraint
            | PrimitiveType::TraitDefinition
            | PrimitiveType::TraitImpl
            | PrimitiveType::TypeDefinition
            | PrimitiveType::TypedExpr
            | PrimitiveType::Type
            | PrimitiveType::UnresolvedType => {
                if !args.is_empty() {
                    let found = args.ordered_args.len() + args.named_args.len();
                    self.push_err(CompilationError::TypeError(
                        TypeCheckError::GenericCountMismatch {
                            item: primitive_type.name().to_string(),
                            expected: 0,
                            found,
                            location,
                        },
                    ));
                }
            }
            PrimitiveType::Str => {
                let item = StrPrimitiveType;
                let (mut args, _) = self.resolve_type_args_inner(
                    args,
                    item,
                    location,
                    PathResolutionMode::MarkAsReferenced,
                    wildcard_allowed,
                );
                assert_eq!(args.len(), 1, "str generics should be: [length]");
                let length = args.pop().unwrap();
                return Type::String(Box::new(length));
            }
            PrimitiveType::Fmtstr => {
                let item = FmtstrPrimitiveType;
                let (mut args, _) = self.resolve_type_args_inner(
                    args,
                    item,
                    location,
                    PathResolutionMode::MarkAsReferenced,
                    wildcard_allowed,
                );
                assert_eq!(args.len(), 2, "fmtstr generics should be: [length, element]");
                let element = args.pop().unwrap();
                let length = args.pop().unwrap();
                return Type::FmtString(Box::new(length), Box::new(element));
            }
        }

        primitive_type.to_type()
    }

    /// Instantiates a primitive type with turbofish generics.
    ///
    /// # Returns
    /// A tuple of:
    /// - The instantiated [Type]
    /// - A boolean indicating whether this primitive type has generics
    pub(crate) fn instantiate_primitive_type_with_turbofish(
        &mut self,
        primitive_type: PrimitiveType,
        turbofish: Option<Turbofish>,
    ) -> (Type, bool) {
        match primitive_type {
            PrimitiveType::Bool
            | PrimitiveType::CtString
            | PrimitiveType::Expr
            | PrimitiveType::Field
            | PrimitiveType::FunctionDefinition
            | PrimitiveType::I8
            | PrimitiveType::I16
            | PrimitiveType::I32
            | PrimitiveType::I64
            | PrimitiveType::U1
            | PrimitiveType::U8
            | PrimitiveType::U16
            | PrimitiveType::U32
            | PrimitiveType::U64
            | PrimitiveType::U128
            | PrimitiveType::Module
            | PrimitiveType::Quoted
            | PrimitiveType::StructDefinition
            | PrimitiveType::TraitConstraint
            | PrimitiveType::TraitDefinition
            | PrimitiveType::TraitImpl
            | PrimitiveType::TypeDefinition
            | PrimitiveType::TypedExpr
            | PrimitiveType::Type
            | PrimitiveType::UnresolvedType => {
                if let Some(turbofish) = turbofish {
                    self.push_err(CompilationError::TypeError(
                        TypeCheckError::GenericCountMismatch {
                            item: primitive_type.name().to_string(),
                            expected: 0,
                            found: turbofish.generics.len(),
                            location: turbofish.location,
                        },
                    ));
                }
                (primitive_type.to_type(), false)
            }
            PrimitiveType::Str => {
                let item = StrPrimitiveType;
                let item_generic_kinds = item.generic_kinds(self.interner);
                let generics = vecmap(&item_generic_kinds, |kind| {
                    self.interner.next_type_variable_with_kind(kind.clone())
                });
                let mut args = if let Some(turbofish) = turbofish {
                    self.resolve_item_turbofish_generics(
                        item.item_kind(),
                        &item.item_name(self.interner),
                        item_generic_kinds,
                        generics,
                        Some(turbofish.generics),
                        turbofish.location,
                    )
                } else {
                    generics
                };
                assert_eq!(args.len(), 1, "str generics should be: [length]");
                let length = args.pop().unwrap();
                (Type::String(Box::new(length)), true)
            }
            PrimitiveType::Fmtstr => {
                let item = FmtstrPrimitiveType;
                let item_generic_kinds = item.generic_kinds(self.interner);
                let generics = vecmap(&item_generic_kinds, |kind| {
                    self.interner.next_type_variable_with_kind(kind.clone())
                });
                let mut args = if let Some(turbofish) = turbofish {
                    self.resolve_item_turbofish_generics(
                        FmtstrPrimitiveType.item_kind(),
                        &item.item_name(self.interner),
                        item_generic_kinds,
                        generics,
                        Some(turbofish.generics),
                        turbofish.location,
                    )
                } else {
                    generics
                };
                assert_eq!(args.len(), 2, "fmtstr generics should be: [length, element]");
                let element = args.pop().unwrap();
                let length = args.pop().unwrap();
                (Type::FmtString(Box::new(length), Box::new(element)), true)
            }
        }
    }
}
