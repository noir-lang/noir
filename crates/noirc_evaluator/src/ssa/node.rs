use crate::errors::{RuntimeError, RuntimeErrorKind};
use crate::ssa::{block::BlockId, builtin, conditional, context::SsaContext, mem::ArrayId};
use acvm::{acir::native_types::Witness, FieldElement};
use iter_extended::vecmap;
use noirc_errors::Location;
use noirc_frontend::{
    monomorphization::ast::{Definition, FuncId},
    BinaryOpKind,
};
use num_bigint::BigUint;
use num_traits::{FromPrimitive, One};
use std::ops::{Add, BitAnd, BitOr, BitXor, Mul, Shl, Shr, Sub};

pub(super) trait Node: std::fmt::Display {
    fn get_type(&self) -> ObjectType;
    fn id(&self) -> NodeId;
    fn size_in_bits(&self) -> u32;
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::fmt::Display for NodeObject {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use FunctionKind::*;
        match self {
            NodeObject::Variable(o) => write!(f, "{o}"),
            NodeObject::Instr(i) => write!(f, "{i}"),
            NodeObject::Const(c) => write!(f, "{c}"),
            NodeObject::Function(Normal(id), ..) => write!(f, "f{}", id.0),
            NodeObject::Function(Builtin(opcode), ..) => write!(f, "{opcode}"),
        }
    }
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Node for Variable {
    fn get_type(&self) -> ObjectType {
        self.obj_type
    }

    fn size_in_bits(&self) -> u32 {
        self.get_type().bits()
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

impl Node for NodeObject {
    fn get_type(&self) -> ObjectType {
        match self {
            NodeObject::Variable(o) => o.get_type(),
            NodeObject::Instr(i) => i.res_type,
            NodeObject::Const(o) => o.value_type,
            NodeObject::Function(..) => ObjectType::Function,
        }
    }

    fn size_in_bits(&self) -> u32 {
        match self {
            NodeObject::Variable(o) => o.size_in_bits(),
            NodeObject::Instr(i) => i.res_type.bits(),
            NodeObject::Const(c) => c.size_in_bits(),
            NodeObject::Function(..) => 0,
        }
    }

    fn id(&self) -> NodeId {
        match self {
            NodeObject::Variable(o) => o.id(),
            NodeObject::Instr(i) => i.id,
            NodeObject::Const(c) => c.id(),
            NodeObject::Function(_, id, _) => *id,
        }
    }
}

impl Node for Constant {
    fn get_type(&self) -> ObjectType {
        self.value_type
    }

    fn size_in_bits(&self) -> u32 {
        self.value.bits().try_into().unwrap()
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NodeId(pub(crate) arena::Index);

impl NodeId {
    pub(crate) fn dummy() -> NodeId {
        NodeId(SsaContext::dummy_id())
    }
    pub(crate) fn is_dummy(&self) -> bool {
        self.0 == SsaContext::dummy_id()
    }
}

#[derive(Debug)]
pub(crate) enum NodeObject {
    Variable(Variable),
    Instr(Instruction),
    Const(Constant),
    Function(FunctionKind, NodeId, /*name:*/ String),
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum FunctionKind {
    Normal(FuncId),
    Builtin(builtin::Opcode),
}

#[derive(Debug)]
pub(crate) struct Constant {
    pub(crate) id: NodeId,
    pub(crate) value: BigUint, //TODO use FieldElement instead
    #[allow(dead_code)]
    pub(crate) value_str: String, //TODO ConstStr subtype
    pub(crate) value_type: ObjectType,
}

impl Constant {
    pub(super) fn get_value_field(&self) -> FieldElement {
        FieldElement::from_be_bytes_reduce(&self.value.to_bytes_be())
    }
}

#[derive(Debug)]
pub(crate) struct Variable {
    pub(crate) id: NodeId,
    pub(crate) obj_type: ObjectType,
    pub(crate) name: String,
    pub(crate) root: Option<NodeId>, //when generating SSA, assignment of an object creates a new one which is linked to the original one
    pub(crate) def: Option<Definition>, //AST definition of the variable
    pub(crate) witness: Option<Witness>,
    #[allow(dead_code)]
    pub(crate) parent_block: BlockId,
}

impl Variable {
    pub(crate) fn root(&self) -> NodeId {
        self.root.unwrap_or(self.id)
    }

    pub(crate) fn new(
        obj_type: ObjectType,
        name: String,
        def: Option<Definition>,
        parent_block: BlockId,
    ) -> Variable {
        Variable {
            id: NodeId::dummy(),
            obj_type,
            name,
            root: None,
            def,
            witness: None,
            parent_block,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ObjectType {
    Numeric(NumericType),
    ArrayPointer(ArrayId),
    Function,
    NotAnObject, //not an object
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum NumericType {
    Signed(u32),   // bit size
    Unsigned(u32), // bit size
    NativeField,
}

impl From<ObjectType> for NumericType {
    fn from(object_type: ObjectType) -> NumericType {
        match object_type {
            ObjectType::Numeric(numeric_type) => numeric_type,
            _ => unreachable!("failed to convert an object type into a numeric type"),
        }
    }
}

impl ObjectType {
    pub(crate) fn bits(&self) -> u32 {
        match self {
            ObjectType::NotAnObject => 0,
            ObjectType::ArrayPointer(_) => 0,
            ObjectType::Function => 0,
            ObjectType::Numeric(numeric_type) => match numeric_type {
                NumericType::Signed(c) | NumericType::Unsigned(c) => *c,
                NumericType::NativeField => FieldElement::max_num_bits(),
            },
        }
    }

    /// Returns the type that represents
    /// a field element.
    /// The most basic/fundamental type in the language
    pub(crate) fn native_field() -> ObjectType {
        ObjectType::Numeric(NumericType::NativeField)
    }
    /// Returns a type that represents an unsigned integer
    pub(crate) fn unsigned_integer(bit_size: u32) -> ObjectType {
        ObjectType::Numeric(NumericType::Unsigned(bit_size))
    }
    /// Returns a type that represents an boolean
    /// Booleans are just seen as an unsigned integer
    /// with a bit size of 1.
    pub(crate) fn boolean() -> ObjectType {
        ObjectType::unsigned_integer(1)
    }
    /// Returns a type that represents an signed integer
    pub(crate) fn signed_integer(bit_size: u32) -> ObjectType {
        ObjectType::Numeric(NumericType::Signed(bit_size))
    }

    /// Returns true, if the `ObjectType`
    /// represents a field element
    pub(crate) fn is_native_field(&self) -> bool {
        matches!(self, ObjectType::Numeric(NumericType::NativeField))
    }
    /// Returns true, if the `ObjectType`
    /// represents an unsigned integer
    pub(crate) fn is_unsigned_integer(&self) -> bool {
        matches!(self, ObjectType::Numeric(NumericType::Unsigned(_)))
    }

    //maximum size of the representation (e.g. signed(8).max_size() return 255, not 128.)
    pub(crate) fn max_size(&self) -> BigUint {
        match self {
            ObjectType::Numeric(NumericType::NativeField) => {
                BigUint::from_bytes_be(&FieldElement::from(-1_i128).to_be_bytes())
            }
            _ => (BigUint::one() << self.bits()) - BigUint::one(),
        }
    }

    // TODO: the name of this function is misleading
    // TODO since the type is not being returned
    pub(crate) fn field_to_type(&self, f: FieldElement) -> FieldElement {
        match self {
            // TODO: document why this is unreachable
            ObjectType::NotAnObject | ObjectType::ArrayPointer(_) => {
                unreachable!()
            }
            ObjectType::Numeric(NumericType::NativeField) => f,
            // TODO: document why this is a TODO and create an issue
            ObjectType::Numeric(NumericType::Signed(_)) => todo!(),
            ObjectType::Function | ObjectType::Numeric(NumericType::Unsigned(_)) => {
                // TODO: document where this 128 comes from
                assert!(self.bits() < 128);
                FieldElement::from(f.to_u128() % (1_u128 << self.bits()))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Instruction {
    pub(crate) id: NodeId,
    pub(crate) operation: Operation,
    pub(crate) res_type: ObjectType, //result type
    pub(crate) parent_block: BlockId,
    pub(crate) res_name: String,
    pub(crate) mark: Mark,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Mark {
    None,
    Deleted,
    ReplaceWith(NodeId),
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.res_name.is_empty() {
            write!(f, "({:?})", self.id.0.into_raw_parts().0)
        } else {
            write!(f, "{}", self.res_name.clone())
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(super) enum NodeEval {
    Const(FieldElement, ObjectType),
    VarOrInstruction(NodeId),
    Function(FunctionKind, NodeId),
}

impl NodeEval {
    pub(super) fn into_const_value(self) -> Option<FieldElement> {
        match self {
            NodeEval::Const(c, _) => Some(c),
            _ => None,
        }
    }

    pub(super) fn into_node_id(self) -> Option<NodeId> {
        match self {
            NodeEval::VarOrInstruction(i) => Some(i),
            NodeEval::Const(_, _) => None,
            NodeEval::Function(_, id) => Some(id),
        }
    }

    //returns the NodeObject index of a NodeEval object
    //if NodeEval is a constant, it may creates a new NodeObject corresponding to the constant value
    pub(super) fn to_index(self, ctx: &mut SsaContext) -> NodeId {
        match self {
            NodeEval::Const(c, t) => ctx.get_or_create_const(c, t),
            NodeEval::VarOrInstruction(i) => i,
            NodeEval::Function(_, id) => id,
        }
    }

    pub(super) fn from_id(ctx: &SsaContext, id: NodeId) -> NodeEval {
        match &ctx[id] {
            NodeObject::Const(c) => {
                let value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be());
                NodeEval::Const(value, c.get_type())
            }
            NodeObject::Function(f, id, _name) => NodeEval::Function(*f, *id),
            NodeObject::Variable(_) | NodeObject::Instr(_) => NodeEval::VarOrInstruction(id),
        }
    }

    fn from_u128(value: u128, typ: ObjectType) -> NodeEval {
        NodeEval::Const(value.into(), typ)
    }
}

impl Instruction {
    pub(super) fn new(
        op_code: Operation,
        r_type: ObjectType,
        parent_block: Option<BlockId>,
    ) -> Instruction {
        let id = NodeId::dummy();
        let p_block = parent_block.unwrap_or_else(BlockId::dummy);

        Instruction {
            id,
            operation: op_code,
            res_type: r_type,
            res_name: String::new(),
            parent_block: p_block,
            mark: Mark::None,
        }
    }

    /// Indicates whether the left and/or right operand of the instruction is required to be truncated to its bit-width
    pub(super) fn truncate_required(&self, ctx: &SsaContext) -> bool {
        match &self.operation {
            Operation::Binary(binary) => binary.truncate_required(),
            Operation::Not(..) => true,
            Operation::Constrain(..) => true,
            Operation::Cast(value_id) => {
                let obj = ctx.try_get_node(*value_id);
                let bits = obj.map_or(0, |obj| obj.size_in_bits());
                self.res_type.bits() > bits
            }
            Operation::Truncate { .. } | Operation::Phi { .. } => false,
            Operation::Nop
            | Operation::Jne(..)
            | Operation::Jeq(..)
            | Operation::Jmp(..)
            | Operation::Cond { .. } => false,
            Operation::Load { .. } => false,
            Operation::Store { .. } => true,
            Operation::Intrinsic(_, _) => true,
            Operation::Call { .. } => true, //return values are in the return statement
            Operation::Return(_) => true,
            Operation::Result { .. } => false,
        }
    }

    pub(super) fn evaluate(&self, ctx: &SsaContext) -> Result<NodeEval, RuntimeError> {
        self.evaluate_with(ctx, |ctx, id| Ok(NodeEval::from_id(ctx, id)))
    }

    //Evaluate the instruction value when its operands are constant (constant folding)
    pub(super) fn evaluate_with<F>(
        &self,
        ctx: &SsaContext,
        mut eval_fn: F,
    ) -> Result<NodeEval, RuntimeError>
    where
        F: FnMut(&SsaContext, NodeId) -> Result<NodeEval, RuntimeError>,
    {
        match &self.operation {
            Operation::Binary(binary) => {
                return binary.evaluate(ctx, self.id, self.res_type, eval_fn)
            }
            Operation::Cast(value) => {
                if let Some(l_const) = eval_fn(ctx, *value)?.into_const_value() {
                    if self.res_type.is_native_field() {
                        return Ok(NodeEval::Const(l_const, self.res_type));
                    } else if let Some(l_const) = l_const.try_into_u128() {
                        return Ok(NodeEval::Const(
                            FieldElement::from(l_const % (1_u128 << self.res_type.bits())),
                            self.res_type,
                        ));
                    }
                }
            }
            Operation::Not(value) => {
                if let Some(l_const) = eval_fn(ctx, *value)?.into_const_value() {
                    let l = self.res_type.field_to_type(l_const).to_u128();
                    let max = (1_u128 << self.res_type.bits()) - 1;
                    return Ok(NodeEval::Const(FieldElement::from((!l) & max), self.res_type));
                }
            }
            Operation::Constrain(value, location) => {
                if let Some(obj) = eval_fn(ctx, *value)?.into_const_value() {
                    if obj.is_one() {
                        // Delete the constrain, it is always true
                        return Ok(NodeEval::VarOrInstruction(NodeId::dummy()));
                    } else if obj.is_zero() {
                        if let Some(location) = *location {
                            return Err(RuntimeError::new(
                                RuntimeErrorKind::UnstructuredError {
                                    message: "Constraint is always false".into(),
                                },
                                Some(location),
                            ));
                        } else {
                            return Err(RuntimeErrorKind::Spanless(
                                "Constraint is always false".into(),
                            )
                            .into());
                        }
                    }
                }
            }
            Operation::Cond { condition, val_true, val_false } => {
                if let Some(cond) = eval_fn(ctx, *condition)?.into_const_value() {
                    if cond.is_zero() {
                        return Ok(NodeEval::VarOrInstruction(*val_false));
                    } else {
                        return Ok(NodeEval::VarOrInstruction(*val_true));
                    }
                }
                if *val_true == *val_false {
                    return Ok(NodeEval::VarOrInstruction(*val_false));
                }
            }
            Operation::Phi { .. } => (), //Phi are simplified by simply_phi() later on; they must not be simplified here
            _ => (),
        }
        Ok(NodeEval::VarOrInstruction(self.id))
    }

    // Simplifies trivial Phi instructions by returning:
    // None, if the instruction is unreachable or in the root block and can be safely deleted
    // Some(id), if the instruction can be replaced by the node id
    // Some(ins_id), if the instruction is not trivial
    pub(super) fn simplify_phi(
        ins_id: NodeId,
        phi_arguments: &[(NodeId, BlockId)],
    ) -> Option<NodeId> {
        let mut same = None;
        for op in phi_arguments {
            if Some(op.0) == same || op.0 == ins_id {
                continue;
            }
            if same.is_some() {
                //no simplification
                return Some(ins_id);
            }

            same = Some(op.0);
        }
        //if same.is_none()  => unreachable phi or in root block, can be replaced by ins.lhs (i.e the root) then.
        same
    }

    pub(super) fn is_deleted(&self) -> bool {
        !matches!(self.mark, Mark::None)
    }

    pub(super) fn standard_form(&mut self) {
        if let Operation::Binary(binary) = &mut self.operation {
            if binary.operator.is_commutative() && binary.rhs < binary.lhs {
                std::mem::swap(&mut binary.rhs, &mut binary.lhs);
            }
        }
    }

    pub(crate) fn get_location(&self) -> Option<Location> {
        match &self.operation {
            Operation::Binary(bin) => match bin.operator {
                BinaryOp::Udiv(location)
                | BinaryOp::Sdiv(location)
                | BinaryOp::Urem(location)
                | BinaryOp::Srem(location)
                | BinaryOp::Div(location)
                | BinaryOp::Shr(location) => Some(location),
                _ => None,
            },
            Operation::Call { location, .. } => Some(*location),
            Operation::Load { location, .. }
            | Operation::Store { location, .. }
            | Operation::Constrain(_, location) => *location,
            Operation::Cast(_)
            | Operation::Truncate { .. }
            | Operation::Not(_)
            | Operation::Jne(_, _)
            | Operation::Jeq(_, _)
            | Operation::Jmp(_)
            | Operation::Phi { .. }
            | Operation::Return(_)
            | Operation::Result { .. }
            | Operation::Cond { .. }
            | Operation::Intrinsic(_, _)
            | Operation::Nop => None,
        }
    }
}

//adapted from LLVM IR
#[allow(dead_code)] //Some enums are not used yet, allow dead_code should be removed once they are all implemented.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum Operation {
    Binary(Binary),
    Cast(NodeId), //convert type
    Truncate {
        value: NodeId,
        bit_size: u32,
        max_bit_size: u32,
    }, //truncate

    Not(NodeId), //(!) Bitwise Not
    Constrain(NodeId, Option<Location>),

    //control flow
    Jne(NodeId, BlockId), //jump on not equal
    Jeq(NodeId, BlockId), //jump on equal
    Jmp(BlockId),         //unconditional jump
    Phi {
        root: NodeId,
        block_args: Vec<(NodeId, BlockId)>,
    },
    Call {
        func: NodeId,
        arguments: Vec<NodeId>,
        returned_arrays: Vec<(super::mem::ArrayId, u32)>,
        predicate: conditional::AssumptionId,
        location: Location,
    },
    Return(Vec<NodeId>), //Return value(s) from a function block
    Result {
        call_instruction: NodeId,
        index: u32,
    }, //Get result index n from a function call
    Cond {
        condition: NodeId,
        val_true: NodeId,
        val_false: NodeId,
    },

    Load {
        array_id: ArrayId,
        index: NodeId,
        location: Option<Location>,
    },
    Store {
        array_id: ArrayId,
        index: NodeId,
        value: NodeId,
        predicate: Option<NodeId>,
        location: Option<Location>,
    },

    Intrinsic(builtin::Opcode, Vec<NodeId>), //Custom implementation of useful primitives which are more performant with Aztec backend

    Nop, // no op
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub(super) enum Opcode {
    Add,
    SafeAdd,
    Sub,
    SafeSub,
    Mul,
    SafeMul,
    Udiv,
    Sdiv,
    Urem,
    Srem,
    Div,
    Eq,
    Ne,
    Ult,
    Ule,
    Slt,
    Sle,
    Lt,
    Lte,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Assign,
    Cond,
    Constrain,
    Cast,     //convert type
    Truncate, //truncate
    Not,      //(!) Bitwise Not

    //control flow
    Jne, //jump on not equal
    Jeq, //jump on equal
    Jmp, //unconditional jump
    Phi,

    Call(NodeId), //Call a function
    Return,       //Return value(s) from a function block
    Results,      //Get result(s) from a function call

    //memory
    Load(ArrayId),
    Store(ArrayId),
    Intrinsic(builtin::Opcode), //Custom implementation of useful primitives
    Nop,                        // no op
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) struct Binary {
    pub(crate) lhs: NodeId,
    pub(crate) rhs: NodeId,
    pub(crate) operator: BinaryOp,
    pub(crate) predicate: Option<NodeId>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum BinaryOp {
    Add, //(+)
    #[allow(dead_code)]
    SafeAdd, //(+) safe addition
    Sub {
        max_rhs_value: BigUint,
    }, //(-)
    #[allow(dead_code)]
    SafeSub {
        max_rhs_value: BigUint,
    }, //(-) safe subtraction
    Mul, //(*)
    #[allow(dead_code)]
    SafeMul, //(*) safe multiplication
    Udiv(Location), //(/) unsigned division
    Sdiv(Location), //(/) signed division
    Urem(Location), //(%) modulo; remainder of unsigned division
    Srem(Location), //(%) remainder of signed division
    Div(Location), //(/) field division
    Eq,  //(==) equal
    Ne,  //(!=) not equal
    Ult, //(<) unsigned less than
    Ule, //(<=) unsigned less or equal
    Slt, //(<) signed less than
    Sle, //(<=) signed less or equal
    Lt,  //(<) field less
    Lte, //(<=) field less or equal
    And, //(&) Bitwise And
    Or,  //(|) Bitwise Or
    Xor, //(^) Bitwise Xor
    Shl, //(<<) Shift left
    Shr(Location), //(>>) Shift right

    Assign,
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op = match &self {
            BinaryOp::Add => "add",
            BinaryOp::SafeAdd => "safe_add",
            BinaryOp::Sub { .. } => "sub",
            BinaryOp::SafeSub { .. } => "safe_sub",
            BinaryOp::Mul => "mul",
            BinaryOp::SafeMul => "safe_mul",
            BinaryOp::Udiv(_) => "udiv",
            BinaryOp::Sdiv(_) => "sdiv",
            BinaryOp::Urem(_) => "urem",
            BinaryOp::Srem(_) => "srem",
            BinaryOp::Div(_) => "div",
            BinaryOp::Eq => "eq",
            BinaryOp::Ne => "ne",
            BinaryOp::Ult => "ult",
            BinaryOp::Ule => "ule",
            BinaryOp::Slt => "slt",
            BinaryOp::Sle => "sle",
            BinaryOp::Lt => "lt",
            BinaryOp::Lte => "lte",
            BinaryOp::And => "and",
            BinaryOp::Or => "or",
            BinaryOp::Xor => "xor",
            BinaryOp::Assign => "assign",
            BinaryOp::Shl => "shl",
            BinaryOp::Shr(_) => "shr",
        };
        write!(f, "{op}")
    }
}

impl Binary {
    fn new(operator: BinaryOp, lhs: NodeId, rhs: NodeId) -> Binary {
        Binary { operator, lhs, rhs, predicate: None }
    }

    pub(super) fn from_ast(
        op_kind: BinaryOpKind,
        op_type: ObjectType,
        lhs: NodeId,
        rhs: NodeId,
        location: Location,
    ) -> Binary {
        let operator = match op_kind {
            BinaryOpKind::Add => BinaryOp::Add,
            BinaryOpKind::Subtract => BinaryOp::Sub { max_rhs_value: BigUint::from_u8(0).unwrap() },
            BinaryOpKind::Multiply => BinaryOp::Mul,
            BinaryOpKind::Equal => BinaryOp::Eq,
            BinaryOpKind::NotEqual => BinaryOp::Ne,
            BinaryOpKind::And => BinaryOp::And,
            BinaryOpKind::Or => BinaryOp::Or,
            BinaryOpKind::Xor => BinaryOp::Xor,
            BinaryOpKind::Divide => {
                let num_type: NumericType = op_type.into();
                match num_type {
                    NumericType::Signed(_) => BinaryOp::Sdiv(location),
                    NumericType::Unsigned(_) => BinaryOp::Udiv(location),
                    NumericType::NativeField => BinaryOp::Div(location),
                }
            }
            BinaryOpKind::Less => {
                let num_type: NumericType = op_type.into();
                match num_type {
                    NumericType::Signed(_) => BinaryOp::Slt,
                    NumericType::Unsigned(_) => BinaryOp::Ult,
                    NumericType::NativeField => BinaryOp::Lt,
                }
            }
            BinaryOpKind::LessEqual => {
                let num_type: NumericType = op_type.into();
                match num_type {
                    NumericType::Signed(_) => BinaryOp::Sle,
                    NumericType::Unsigned(_) => BinaryOp::Ule,
                    NumericType::NativeField => BinaryOp::Lte,
                }
            }
            BinaryOpKind::Greater => {
                let num_type: NumericType = op_type.into();
                match num_type {
                    NumericType::Signed(_) => return Binary::new(BinaryOp::Slt, rhs, lhs),
                    NumericType::Unsigned(_) => return Binary::new(BinaryOp::Ult, rhs, lhs),
                    NumericType::NativeField => return Binary::new(BinaryOp::Lt, rhs, lhs),
                }
            }
            BinaryOpKind::GreaterEqual => {
                let num_type: NumericType = op_type.into();
                match num_type {
                    NumericType::Signed(_) => return Binary::new(BinaryOp::Sle, rhs, lhs),
                    NumericType::Unsigned(_) => return Binary::new(BinaryOp::Ule, rhs, lhs),
                    NumericType::NativeField => return Binary::new(BinaryOp::Lte, rhs, lhs),
                }
            }
            BinaryOpKind::ShiftLeft => BinaryOp::Shl,
            BinaryOpKind::ShiftRight => BinaryOp::Shr(location),
            BinaryOpKind::Modulo => {
                let num_type: NumericType = op_type.into();
                match num_type {
                    NumericType::Signed(_) => {
                        return Binary::new(BinaryOp::Srem(location), lhs, rhs)
                    }
                    NumericType::Unsigned(_) => {
                        return Binary::new(BinaryOp::Urem(location), lhs, rhs)
                    }
                    NumericType::NativeField => {
                        unimplemented!("Modulo operation with Field elements is not supported")
                    }
                }
            }
        };

        Binary::new(operator, lhs, rhs)
    }

    fn zero_div_error(&self, location: &Location) -> Result<(), RuntimeError> {
        if self.predicate.is_none() {
            Err(RuntimeError {
                location: Some(*location),
                kind: RuntimeErrorKind::UnstructuredError {
                    message: "Panic - division by zero".to_string(),
                },
            })
        } else {
            Ok(())
        }
    }

    fn evaluate<F>(
        &self,
        ctx: &SsaContext,
        id: NodeId,
        res_type: ObjectType,
        mut eval_fn: F,
    ) -> Result<NodeEval, RuntimeError>
    where
        F: FnMut(&SsaContext, NodeId) -> Result<NodeEval, RuntimeError>,
    {
        let l_eval = eval_fn(ctx, self.lhs)?;
        let r_eval = eval_fn(ctx, self.rhs)?;
        let l_type = ctx.object_type(self.lhs);
        let r_type = ctx.object_type(self.rhs);

        let lhs = l_eval.into_const_value();
        let rhs = r_eval.into_const_value();

        let l_is_zero = lhs.map_or(false, |x| x.is_zero());
        let r_is_zero = rhs.map_or(false, |x| x.is_zero());

        match &self.operator {
            BinaryOp::Add | BinaryOp::SafeAdd => {
                if l_is_zero {
                    return Ok(r_eval);
                } else if r_is_zero {
                    return Ok(l_eval);
                }
                assert_eq!(l_type, r_type);
                if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return Ok(wrapping(lhs, rhs, l_type, u128::add, Add::add));
                }
                //if only one is const, we could try to do constant propagation but this will be handled by the arithmetization step anyways
                //so it is probably not worth it.
                //same for x+x vs 2*x
            }
            BinaryOp::Sub { .. } | BinaryOp::SafeSub { .. } => {
                if r_is_zero {
                    return Ok(l_eval);
                }
                if self.lhs == self.rhs {
                    return Ok(NodeEval::from_u128(0, res_type));
                }
                //constant folding
                if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return Ok(wrapping(lhs, rhs, res_type, u128::wrapping_sub, Sub::sub));
                }
            }
            BinaryOp::Mul | BinaryOp::SafeMul => {
                let l_is_one = lhs.map_or(false, |x| x.is_one());
                let r_is_one = rhs.map_or(false, |x| x.is_one());
                assert_eq!(l_type, r_type);
                if l_is_zero || r_is_one {
                    return Ok(l_eval);
                } else if r_is_zero || l_is_one {
                    return Ok(r_eval);
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return Ok(wrapping(lhs, rhs, res_type, u128::mul, Mul::mul));
                }
                //if only one is const, we could try to do constant propagation but this will be handled by the arithmetization step anyways
                //so it is probably not worth it.
            }

            BinaryOp::Udiv(loc) => {
                if r_is_zero {
                    self.zero_div_error(loc)?;
                } else if l_is_zero {
                    return Ok(l_eval); //TODO should we ensure rhs != 0 ???
                }
                //constant folding
                else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    let lhs = res_type.field_to_type(lhs).to_u128();
                    let rhs = res_type.field_to_type(rhs).to_u128();
                    return Ok(NodeEval::Const(FieldElement::from(lhs / rhs), res_type));
                }
            }
            BinaryOp::Div(loc) => {
                if r_is_zero {
                    self.zero_div_error(loc)?;
                } else if l_is_zero {
                    return Ok(l_eval); //TODO should we ensure rhs != 0 ???
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return Ok(NodeEval::Const(lhs / rhs, res_type));
                }
            }
            BinaryOp::Sdiv(loc) => {
                if r_is_zero {
                    self.zero_div_error(loc)?;
                } else if l_is_zero {
                    return Ok(l_eval); //TODO should we ensure rhs != 0 ???
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    let a = field_to_signed(lhs, res_type.bits());
                    let b = field_to_signed(rhs, res_type.bits());
                    return Ok(NodeEval::Const(signed_to_field(a / b, res_type.bits())?, res_type));
                }
            }
            BinaryOp::Urem(loc) => {
                if r_is_zero {
                    self.zero_div_error(loc)?;
                } else if l_is_zero {
                    return Ok(l_eval); //TODO what is the correct result?
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    let lhs = res_type.field_to_type(lhs).to_u128();
                    let rhs = res_type.field_to_type(rhs).to_u128();
                    let result = lhs - rhs * (lhs / rhs);
                    return Ok(NodeEval::Const(FieldElement::from(result), res_type));
                }
            }
            BinaryOp::Srem(loc) => {
                if r_is_zero {
                    self.zero_div_error(loc)?;
                } else if l_is_zero {
                    return Ok(l_eval); //TODO what is the correct result?
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    let a = field_to_signed(lhs, res_type.bits());
                    let b = field_to_signed(rhs, res_type.bits());
                    return Ok(NodeEval::Const(
                        signed_to_field(a - b + (a / b), res_type.bits())?,
                        res_type,
                    ));
                }
            }
            BinaryOp::Ult => {
                if r_is_zero {
                    return Ok(NodeEval::Const(FieldElement::zero(), ObjectType::boolean()));
                    //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    assert!(
                        !res_type.is_native_field(),
                        "ICE: comparisons are not implemented for field elements"
                    );
                    return Ok(NodeEval::Const(
                        FieldElement::from(lhs < rhs),
                        ObjectType::boolean(),
                    ));
                }
            }
            BinaryOp::Ule => {
                if l_is_zero {
                    return Ok(NodeEval::Const(FieldElement::one(), ObjectType::boolean()));
                    //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    assert!(
                        !res_type.is_native_field(),
                        "ICE: comparisons are not implemented for field elements"
                    );
                    return Ok(NodeEval::Const(
                        FieldElement::from(lhs <= rhs),
                        ObjectType::boolean(),
                    ));
                }
            }
            BinaryOp::Slt => (),
            BinaryOp::Sle => (),
            BinaryOp::Lt => {
                if r_is_zero {
                    return Ok(NodeEval::Const(FieldElement::zero(), ObjectType::boolean()));
                    //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return Ok(NodeEval::Const(
                        FieldElement::from(lhs < rhs),
                        ObjectType::boolean(),
                    ));
                }
            }
            BinaryOp::Lte => {
                if l_is_zero {
                    return Ok(NodeEval::Const(FieldElement::one(), ObjectType::boolean()));
                    //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return Ok(NodeEval::Const(
                        FieldElement::from(lhs <= rhs),
                        ObjectType::boolean(),
                    ));
                }
            }
            BinaryOp::Eq => {
                if self.lhs == self.rhs {
                    return Ok(NodeEval::Const(FieldElement::one(), ObjectType::boolean()));
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return Ok(NodeEval::Const(
                        FieldElement::from(lhs == rhs),
                        ObjectType::boolean(),
                    ));
                }
            }
            BinaryOp::Ne => {
                if self.lhs == self.rhs {
                    return Ok(NodeEval::Const(FieldElement::zero(), ObjectType::boolean()));
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return Ok(NodeEval::Const(
                        FieldElement::from(lhs != rhs),
                        ObjectType::boolean(),
                    ));
                }
            }
            BinaryOp::And => {
                //Bitwise AND
                if l_is_zero || self.lhs == self.rhs {
                    return Ok(l_eval);
                } else if r_is_zero {
                    return Ok(r_eval);
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return Ok(wrapping(lhs, rhs, res_type, u128::bitand, field_op_not_allowed));
                } else {
                    let n = res_type.bits();
                    let max = FieldElement::from(2_u128.pow(n) - 1);
                    if lhs == Some(max) {
                        return Ok(r_eval);
                    } else if rhs == Some(max) {
                        return Ok(l_eval);
                    }
                }
            }
            BinaryOp::Or => {
                //Bitwise OR
                if l_is_zero || self.lhs == self.rhs {
                    return Ok(r_eval);
                } else if r_is_zero {
                    return Ok(l_eval);
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return Ok(wrapping(lhs, rhs, res_type, u128::bitor, field_op_not_allowed));
                } else {
                    let n = res_type.bits();
                    let max = FieldElement::from(2_u128.pow(n) - 1);
                    if lhs == Some(max) || rhs == Some(max) {
                        return Ok(NodeEval::Const(max, res_type));
                    }
                }
            }
            BinaryOp::Xor => {
                if self.lhs == self.rhs {
                    return Ok(NodeEval::Const(FieldElement::zero(), res_type));
                } else if l_is_zero {
                    return Ok(r_eval);
                } else if r_is_zero {
                    return Ok(l_eval);
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return Ok(wrapping(lhs, rhs, res_type, u128::bitxor, field_op_not_allowed));
                }
            }
            BinaryOp::Shl => {
                if l_is_zero {
                    return Ok(l_eval);
                }
                if r_is_zero {
                    return Ok(l_eval);
                }
                if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return Ok(wrapping(lhs, rhs, res_type, u128::shl, field_op_not_allowed));
                }
            }
            BinaryOp::Shr(_) => {
                if l_is_zero {
                    return Ok(l_eval);
                }
                if r_is_zero {
                    return Ok(l_eval);
                }
                if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return Ok(wrapping(lhs, rhs, res_type, u128::shr, field_op_not_allowed));
                }
            }
            BinaryOp::Assign => (),
        }
        Ok(NodeEval::VarOrInstruction(id))
    }

    fn truncate_required(&self) -> bool {
        match &self.operator {
            BinaryOp::Add => false,
            BinaryOp::SafeAdd => false,
            BinaryOp::Sub { .. } => false,
            BinaryOp::SafeSub { .. } => false,
            BinaryOp::Mul => false,
            BinaryOp::SafeMul => false,
            BinaryOp::Udiv(_) => true,
            BinaryOp::Sdiv(_) => true,
            BinaryOp::Urem(_) => true,
            BinaryOp::Srem(_) => true,
            BinaryOp::Div(_) => false,
            BinaryOp::Eq => true,
            BinaryOp::Ne => true,
            BinaryOp::Ult => true,
            BinaryOp::Ule => true,
            BinaryOp::Slt => true,
            BinaryOp::Sle => true,
            BinaryOp::Lt => true,
            BinaryOp::Lte => true,
            BinaryOp::And => true,
            BinaryOp::Or => true,
            BinaryOp::Xor => true,
            BinaryOp::Assign => false,
            BinaryOp::Shl => true,
            BinaryOp::Shr(_) => true,
        }
    }

    pub(super) fn opcode(&self) -> Opcode {
        match &self.operator {
            BinaryOp::Add => Opcode::Add,
            BinaryOp::SafeAdd => Opcode::SafeAdd,
            BinaryOp::Sub { .. } => Opcode::Sub,
            BinaryOp::SafeSub { .. } => Opcode::SafeSub,
            BinaryOp::Mul => Opcode::Mul,
            BinaryOp::SafeMul => Opcode::SafeMul,
            BinaryOp::Udiv(_) => Opcode::Udiv,
            BinaryOp::Sdiv(_) => Opcode::Sdiv,
            BinaryOp::Urem(_) => Opcode::Urem,
            BinaryOp::Srem(_) => Opcode::Srem,
            BinaryOp::Div(_) => Opcode::Div,
            BinaryOp::Eq => Opcode::Eq,
            BinaryOp::Ne => Opcode::Ne,
            BinaryOp::Ult => Opcode::Ult,
            BinaryOp::Ule => Opcode::Ule,
            BinaryOp::Slt => Opcode::Slt,
            BinaryOp::Sle => Opcode::Sle,
            BinaryOp::Lt => Opcode::Lt,
            BinaryOp::Lte => Opcode::Lte,
            BinaryOp::And => Opcode::And,
            BinaryOp::Or => Opcode::Or,
            BinaryOp::Xor => Opcode::Xor,
            BinaryOp::Shl => Opcode::Shl,
            BinaryOp::Shr(_) => Opcode::Shr,
            BinaryOp::Assign => Opcode::Assign,
        }
    }
}

/// Perform the given numeric operation and modulo the result by the max value for the given bit count
/// if the res_type is not a NativeField.
fn wrapping(
    lhs: FieldElement,
    rhs: FieldElement,
    res_type: ObjectType,
    u128_op: impl FnOnce(u128, u128) -> u128,
    field_op: impl FnOnce(FieldElement, FieldElement) -> FieldElement,
) -> NodeEval {
    if !res_type.is_native_field() {
        let type_modulo = 1_u128 << res_type.bits();
        let lhs = lhs.to_u128() % type_modulo;
        let rhs = rhs.to_u128() % type_modulo;
        let mut x = u128_op(lhs, rhs);
        x %= type_modulo;
        NodeEval::from_u128(x, res_type)
    } else {
        NodeEval::Const(field_op(lhs, rhs), res_type)
    }
}

fn field_op_not_allowed(_lhs: FieldElement, _rhs: FieldElement) -> FieldElement {
    unreachable!("operation not allowed for FieldElement");
}

impl Operation {
    pub(super) fn binary(op: BinaryOp, lhs: NodeId, rhs: NodeId) -> Self {
        Operation::Binary(Binary::new(op, lhs, rhs))
    }

    pub(super) fn is_dummy_store(&self) -> bool {
        match self {
            Operation::Store { index, value, .. } => {
                *index == NodeId::dummy() && *value == NodeId::dummy()
            }
            _ => false,
        }
    }

    pub(super) fn map_id(&self, mut f: impl FnMut(NodeId) -> NodeId) -> Operation {
        use Operation::*;
        match self {
            Binary(self::Binary { lhs, rhs, operator, predicate }) => Binary(self::Binary {
                lhs: f(*lhs),
                rhs: f(*rhs),
                operator: operator.clone(),
                predicate: predicate.as_ref().map(|pred| f(*pred)),
            }),
            Cast(value) => Cast(f(*value)),
            Truncate { value, bit_size, max_bit_size } => {
                Truncate { value: f(*value), bit_size: *bit_size, max_bit_size: *max_bit_size }
            }
            Not(id) => Not(f(*id)),
            Constrain(id, loc) => Constrain(f(*id), *loc),
            Jne(id, block) => Jne(f(*id), *block),
            Jeq(id, block) => Jeq(f(*id), *block),
            Jmp(block) => Jmp(*block),
            Phi { root, block_args } => Phi {
                root: f(*root),
                block_args: vecmap(block_args, |(id, block)| (f(*id), *block)),
            },
            Cond { condition, val_true: lhs, val_false: rhs } => {
                Cond { condition: f(*condition), val_true: f(*lhs), val_false: f(*rhs) }
            }
            Load { array_id: array, index, location } => {
                Load { array_id: *array, index: f(*index), location: *location }
            }
            Store { array_id: array, index, value, predicate, location } => Store {
                array_id: *array,
                index: f(*index),
                value: f(*value),
                predicate: predicate.as_ref().map(|pred| f(*pred)),
                location: *location,
            },
            Intrinsic(i, args) => Intrinsic(*i, vecmap(args.iter().copied(), f)),
            Nop => Nop,
            Call { func: func_id, arguments, returned_arrays, predicate, location } => Call {
                func: f(*func_id),
                arguments: vecmap(arguments.iter().copied(), f),
                returned_arrays: returned_arrays.clone(),
                predicate: *predicate,
                location: *location,
            },
            Return(values) => Return(vecmap(values.iter().copied(), f)),
            Result { call_instruction, index } => {
                Result { call_instruction: f(*call_instruction), index: *index }
            }
        }
    }

    /// Mutate each contained NodeId in place using the given function f
    pub(super) fn map_id_mut(&mut self, mut f: impl FnMut(NodeId) -> NodeId) {
        use Operation::*;
        match self {
            Binary(self::Binary { lhs, rhs, predicate, .. }) => {
                *lhs = f(*lhs);
                *rhs = f(*rhs);
                *predicate = predicate.as_mut().map(|pred| f(*pred));
            }
            Cast(value) => *value = f(*value),
            Truncate { value, .. } => *value = f(*value),
            Not(id) => *id = f(*id),
            Constrain(id, ..) => *id = f(*id),
            Jne(id, _) => *id = f(*id),
            Jeq(id, _) => *id = f(*id),
            Jmp(_) => (),
            Phi { root, block_args } => {
                f(*root);
                for (id, _block) in block_args {
                    *id = f(*id);
                }
            }
            Cond { condition, val_true: lhs, val_false: rhs } => {
                *condition = f(*condition);
                *lhs = f(*lhs);
                *rhs = f(*rhs);
            }
            Load { index, .. } => *index = f(*index),
            Store { index, value, predicate, .. } => {
                *index = f(*index);
                *value = f(*value);
                *predicate = predicate.as_mut().map(|pred| f(*pred));
            }
            Intrinsic(_, args) => {
                for arg in args {
                    *arg = f(*arg);
                }
            }
            Nop => (),
            Call { func, arguments, .. } => {
                *func = f(*func);
                for arg in arguments {
                    *arg = f(*arg);
                }
            }
            Return(values) => {
                for value in values {
                    *value = f(*value);
                }
            }
            Result { call_instruction, index: _ } => {
                *call_instruction = f(*call_instruction);
            }
        }
    }

    /// This is the same as map_id but doesn't return a new Operation
    pub(super) fn for_each_id(&self, mut f: impl FnMut(NodeId)) {
        use Operation::*;
        match self {
            Binary(self::Binary { lhs, rhs, .. }) => {
                f(*lhs);
                f(*rhs);
            }
            Cast(value) => f(*value),
            Truncate { value, .. } => f(*value),
            Not(id) => f(*id),
            Constrain(id, ..) => f(*id),
            Jne(id, _) => f(*id),
            Jeq(id, _) => f(*id),
            Jmp(_) => (),
            Phi { root, block_args } => {
                f(*root);
                for (id, _block) in block_args {
                    f(*id);
                }
            }
            Cond { condition, val_true: lhs, val_false: rhs } => {
                f(*condition);
                f(*lhs);
                f(*rhs);
            }
            Load { index, .. } => f(*index),
            Store { index, value, .. } => {
                f(*index);
                f(*value);
            }
            Intrinsic(_, args) => args.iter().copied().for_each(f),
            Nop => (),
            Call { func, arguments, .. } => {
                f(*func);
                arguments.iter().copied().for_each(f);
            }
            Return(values) => values.iter().copied().for_each(f),
            Result { call_instruction, .. } => {
                f(*call_instruction);
            }
        }
    }

    pub(super) fn opcode(&self) -> Opcode {
        match self {
            Operation::Binary(binary) => binary.opcode(),
            Operation::Cast(_) => Opcode::Cast,
            Operation::Truncate { .. } => Opcode::Truncate,
            Operation::Not(_) => Opcode::Not,
            Operation::Constrain(..) => Opcode::Constrain,
            Operation::Jne(_, _) => Opcode::Jne,
            Operation::Jeq(_, _) => Opcode::Jeq,
            Operation::Jmp(_) => Opcode::Jmp,
            Operation::Phi { .. } => Opcode::Phi,
            Operation::Cond { .. } => Opcode::Cond,
            Operation::Call { func, .. } => Opcode::Call(*func),
            Operation::Return(_) => Opcode::Return,
            Operation::Result { .. } => Opcode::Results,
            Operation::Load { array_id, .. } => Opcode::Load(*array_id),
            Operation::Store { array_id, .. } => Opcode::Store(*array_id),
            Operation::Intrinsic(opcode, _) => Opcode::Intrinsic(*opcode),
            Operation::Nop => Opcode::Nop,
        }
    }
}

impl BinaryOp {
    fn is_commutative(&self) -> bool {
        matches!(
            self,
            BinaryOp::Add
                | BinaryOp::SafeAdd
                | BinaryOp::Mul
                | BinaryOp::SafeMul
                | BinaryOp::And
                | BinaryOp::Or
                | BinaryOp::Xor
        )
    }
}
// TODO: We should create a constant and explain where the 127 and 126 constants
// TODO are from
fn field_to_signed(f: FieldElement, n: u32) -> i128 {
    assert!(n < 127);
    let a = f.to_u128();
    let pow_2 = 2_u128.pow(n);
    if a < pow_2 {
        a as i128
    } else {
        (a - 2 * pow_2) as i128
    }
}

fn signed_to_field(a: i128, n: u32) -> Result<FieldElement, RuntimeError> {
    if n >= 126 {
        return Err(RuntimeErrorKind::UnstructuredError {
            message: "ICE: cannot convert signed {n} bit size into field".to_string(),
        })?;
    }
    if a >= 0 {
        Ok(FieldElement::from(a))
    } else {
        let b = (a + 2_i128.pow(n + 1)) as u128;
        Ok(FieldElement::from(b))
    }
}
