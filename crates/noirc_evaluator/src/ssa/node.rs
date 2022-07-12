use std::convert::TryInto;

use crate::object::Object;
use acvm::acir::native_types::Witness;
use acvm::acir::OPCODE;
use acvm::FieldElement;
use arena;
use noirc_errors::Span;
use noirc_frontend::hir::def_map::ModuleId;
use noirc_frontend::hir_def::expr::HirBinaryOpKind;
use noirc_frontend::node_interner::DefinitionId;
use noirc_frontend::util::vecmap;
use noirc_frontend::{Signedness, Type};
use num_bigint::BigUint;
use num_traits::{FromPrimitive, One};
use std::ops::{Add, Mul, Sub};
use std::ops::{BitAnd, BitOr, BitXor, Shl, Shr};

use super::block::BlockId;
use super::context::SsaContext;
use super::mem::ArrayId;

pub trait Node: std::fmt::Display {
    fn get_type(&self) -> ObjectType;
    fn get_id(&self) -> NodeId;
    fn size_in_bits(&self) -> u32;
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::fmt::Display for NodeObj {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NodeObj::Obj(o) => write!(f, "{}", o),
            NodeObj::Instr(i) => write!(f, "{}", i),
            NodeObj::Const(c) => write!(f, "{}", c),
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

    fn get_id(&self) -> NodeId {
        self.id
    }
}

impl Node for NodeObj {
    fn get_type(&self) -> ObjectType {
        match self {
            NodeObj::Obj(o) => o.get_type(),
            NodeObj::Instr(i) => i.res_type,
            NodeObj::Const(o) => o.value_type,
        }
    }

    fn size_in_bits(&self) -> u32 {
        match self {
            NodeObj::Obj(o) => o.size_in_bits(),
            NodeObj::Instr(i) => i.res_type.bits(),
            NodeObj::Const(c) => c.size_in_bits(),
        }
    }

    fn get_id(&self) -> NodeId {
        match self {
            NodeObj::Obj(o) => o.get_id(),
            NodeObj::Instr(i) => i.id,
            NodeObj::Const(c) => c.get_id(),
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

    fn get_id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub arena::Index);

impl NodeId {
    pub fn dummy() -> NodeId {
        NodeId(SsaContext::dummy_id())
    }
}

#[derive(Debug)]
pub enum NodeObj {
    Obj(Variable),
    Instr(Instruction),
    Const(Constant),
}

impl NodeObj {
    pub fn set_id_and_parent(&mut self, new_id: NodeId, block: BlockId) {
        match self {
            NodeObj::Instr(inst) => {
                inst.id = new_id;
                inst.parent_block = block;
            }
            _ => unreachable!("Expected instruction, got {}", self),
        }
    }
}

#[derive(Debug)]
pub struct Constant {
    pub id: NodeId,
    pub value: BigUint,    //TODO use FieldElement instead
    pub value_str: String, //TODO ConstStr subtype
    pub value_type: ObjectType,
}

impl Constant {
    pub fn get_value_field(&self) -> FieldElement {
        FieldElement::from_be_bytes_reduce(&self.value.to_bytes_be())
    }
}

#[derive(Debug)]
pub struct Variable {
    pub id: NodeId,
    pub obj_type: ObjectType,
    pub name: String,
    //pub cur_value: arena::Index, //for generating the SSA form, current value of the object during parsing of the AST
    pub root: Option<NodeId>, //when generating SSA, assignment of an object creates a new one which is linked to the original one
    pub def: Option<DefinitionId>, //TODO redundant with root - should it be an option?
    //TODO clarify where cur_value and root is stored, and also this:
    //  pub max_bits: u32,                  //max possible bit size of the expression
    //  pub max_value: Option<BigUInt>,     //maximum possible value of the expression, if less than max_bits
    pub witness: Option<Witness>,
    pub parent_block: BlockId,
}

impl Variable {
    pub fn get_root(&self) -> NodeId {
        self.root.unwrap_or(self.id)
    }

    pub fn new(
        obj_type: ObjectType,
        name: String,
        def: Option<DefinitionId>,
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ObjectType {
    //Numeric(NumericType),
    NativeField,
    // custom_field(BigUint), //TODO requires copy trait for BigUint
    Boolean,
    Unsigned(u32), //bit size
    Signed(u32),   //bit size
    Pointer(ArrayId),
    //custom(u32),   //user-defined struct, u32 refers to the id of the type in...?todo
    //TODO big_int
    //TODO floats
    NotAnObject, //not an object
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NumericType {
    Signed(u32),
    Unsigned(u32),
    NativeField,
}

impl From<ObjectType> for NumericType {
    fn from(object_type: ObjectType) -> NumericType {
        match object_type {
            ObjectType::Signed(x) => NumericType::Signed(x),
            ObjectType::Unsigned(x) => NumericType::Unsigned(x),
            ObjectType::NativeField => NumericType::NativeField,
            _ => unreachable!("failed to convert an object type into a numeric type"),
        }
    }
}

impl From<&Type> for ObjectType {
    fn from(t: &noirc_frontend::Type) -> ObjectType {
        match t {
            Type::Bool => ObjectType::Boolean,
            Type::FieldElement(..) => ObjectType::NativeField,
            Type::Integer(_, _ftype, sign, bit_size) => {
                assert!(
                    *bit_size < super::integer::short_integer_max_bit_size(),
                    "long integers are not yet supported"
                );
                match sign {
                    Signedness::Signed => ObjectType::Signed(*bit_size),
                    Signedness::Unsigned => ObjectType::Unsigned(*bit_size),
                }
            }
            Type::PolymorphicInteger(_, binding) => match &*binding.borrow() {
                noirc_frontend::TypeBinding::Bound(typ) => typ.into(),
                noirc_frontend::TypeBinding::Unbound(_) => Type::default_int_type(None).into(),
            },
            // TODO: We should probably not convert an array type into the element type
            noirc_frontend::Type::Array(_, _, t) => ObjectType::from(t.as_ref()),
            x => unimplemented!("Conversion to ObjectType is unimplemented for type {}", x),
        }
    }
}

impl From<Type> for ObjectType {
    fn from(t: noirc_frontend::Type) -> ObjectType {
        ObjectType::from(&t)
    }
}

impl ObjectType {
    pub fn get_type_from_object(obj: &Object) -> ObjectType {
        match obj {
            Object::Arithmetic(_) => {
                todo!();
                //ObjectType::native_field
            }
            Object::Array(_) => {
                todo!(); //TODO we should match an array in mem: ObjectType::Pointer(0)
            }
            Object::Constants(_) => ObjectType::NativeField, //TODO
            Object::Integer(i) => {
                assert!(
                    i.num_bits < super::integer::short_integer_max_bit_size(),
                    "long integers are not yet supported"
                );
                ObjectType::Unsigned(i.num_bits)
            } //TODO signed or unsigned?
            Object::Linear(_) => {
                ObjectType::NativeField //TODO check with Kev!
            }
            Object::Null => ObjectType::NotAnObject,
        }
    }

    pub fn bits(&self) -> u32 {
        match self {
            ObjectType::Boolean => 1,
            ObjectType::NativeField => FieldElement::max_num_bits(),
            ObjectType::NotAnObject => 0,
            ObjectType::Signed(c) => *c,
            ObjectType::Unsigned(c) => *c,
            ObjectType::Pointer(_) => 0,
        }
    }

    //maximum size of the representation (e.g. signed(8).max_size() return 255, not 128.)
    pub fn max_size(&self) -> BigUint {
        match self {
            &ObjectType::NativeField => {
                BigUint::from_bytes_be(&FieldElement::from(-1_i128).to_bytes())
            }
            _ => (BigUint::one() << self.bits()) - BigUint::one(),
        }
    }

    pub fn deref(&self, ctx: &SsaContext) -> ObjectType {
        match self {
            ObjectType::Pointer(a) => ctx.mem[*a].element_type,
            _ => *self,
        }
    }

    pub fn type_to_pointer(&self) -> ArrayId {
        match self {
            ObjectType::Pointer(a) => *a,
            _ => unreachable!("Type is not a pointer",),
        }
    }

    pub fn field_to_type(&self, f: FieldElement) -> FieldElement {
        match self {
            ObjectType::NotAnObject | ObjectType::Pointer(_) => {
                unreachable!()
            }
            ObjectType::NativeField => f,
            ObjectType::Signed(_) => todo!(),
            _ => {
                assert!(self.bits() < 128);
                FieldElement::from(f.to_u128() % (1_u128 << self.bits()))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Instruction {
    pub id: NodeId,
    pub operation: Operation,
    pub res_type: ObjectType, //result type
    pub parent_block: BlockId,
    pub res_name: String,
    pub mark: Mark,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mark {
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

#[derive(Debug, Clone, Copy)]
pub enum NodeEval {
    Const(FieldElement, ObjectType),
    VarOrInstruction(NodeId),
}

impl NodeEval {
    pub fn into_const_value(self) -> Option<FieldElement> {
        match self {
            NodeEval::Const(c, _) => Some(c),
            _ => None,
        }
    }

    pub fn into_node_id(self) -> Option<NodeId> {
        match self {
            NodeEval::VarOrInstruction(i) => Some(i),
            NodeEval::Const(_, _) => None,
        }
    }

    //returns the NodeObj index of a NodeEval object
    //if NodeEval is a constant, it may creates a new NodeObj corresponding to the constant value
    pub fn to_index(self, ctx: &mut SsaContext) -> NodeId {
        match self {
            NodeEval::Const(c, t) => ctx.get_or_create_const(c, t),
            NodeEval::VarOrInstruction(i) => i,
        }
    }

    pub fn from_id(ctx: &SsaContext, id: NodeId) -> NodeEval {
        match &ctx[id] {
            NodeObj::Const(c) => {
                let value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be());
                NodeEval::Const(value, c.get_type())
            }
            _ => NodeEval::VarOrInstruction(id),
        }
    }

    fn from_u128(value: u128, typ: ObjectType) -> NodeEval {
        NodeEval::Const(value.into(), typ)
    }
}

impl Instruction {
    pub fn new(
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
    pub fn truncate_required(&self, ctx: &SsaContext) -> bool {
        match &self.operation {
            Operation::Binary(binary) => binary.truncate_required(),
            Operation::Not(..) => true,
            Operation::Cast(value_id) => {
                let obj = ctx.try_get_node(*value_id);
                let bits = obj.map_or(0, |obj| obj.size_in_bits());
                self.res_type.bits() > bits
            }
            Operation::Truncate { .. } | Operation::Phi { .. } => false,
            Operation::Nop | Operation::Jne(..) | Operation::Jeq(..) | Operation::Jmp(..) => false,
            Operation::Load { .. } => false,
            Operation::Store { .. } => true,
            Operation::Intrinsic(_, _) => true, //TODO to check
            Operation::Call(_, _, _) => false, //return values are in the return statment, should we truncate function arguments? probably but not lhs and rhs anyways.
            Operation::Return(_) => true,
            Operation::Result { .. } => false,
        }
    }

    pub fn evaluate(&self, ctx: &SsaContext) -> NodeEval {
        self.evaluate_with(ctx, NodeEval::from_id)
    }

    //Evaluate the instruction value when its operands are constant (constant folding)
    pub fn evaluate_with<F>(&self, ctx: &SsaContext, mut eval_fn: F) -> NodeEval
    where
        F: FnMut(&SsaContext, NodeId) -> NodeEval,
    {
        match &self.operation {
            Operation::Binary(binary) => {
                return binary.evaluate(ctx, self.id, self.res_type, eval_fn)
            }
            Operation::Cast(value) => {
                if let Some(l_const) = eval_fn(ctx, *value).into_const_value() {
                    if self.res_type == ObjectType::NativeField {
                        return NodeEval::Const(l_const, self.res_type);
                    } else if let Some(l_const) = l_const.try_into_u128() {
                        return NodeEval::Const(
                            FieldElement::from(l_const % (1_u128 << self.res_type.bits())),
                            self.res_type,
                        );
                    }
                }
            }
            Operation::Not(value) => {
                if let Some(l_const) = eval_fn(ctx, *value).into_const_value() {
                    let l = self.res_type.field_to_type(l_const).to_u128();
                    let max = (1_u128 << self.res_type.bits()) - 1;
                    return NodeEval::Const(FieldElement::from((!l) & max), self.res_type);
                }
            }
            Operation::Phi { .. } => (), //Phi are simplified by simply_phi() later on; they must not be simplified here
            _ => (),
        }
        NodeEval::VarOrInstruction(self.id)
    }

    // Simplifies trivial Phi instructions by returning:
    // None, if the instruction is unreachable or in the root block and can be safely deleted
    // Some(id), if the instruction can be replaced by the node id
    // Some(ins_id), if the instruction is not trivial
    pub fn simplify_phi(ins_id: NodeId, phi_arguments: &[(NodeId, BlockId)]) -> Option<NodeId> {
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

    pub fn is_deleted(&self) -> bool {
        !matches!(self.mark, Mark::None)
    }

    pub fn standard_form(&mut self) {
        if let Operation::Binary(binary) = &mut self.operation {
            if let BinaryOp::Constrain(op, ..) = &binary.operator {
                match op {
                    ConstrainOp::Eq => {
                        if binary.lhs == binary.rhs {
                            self.operation = Operation::Nop;
                            return;
                        }
                    }
                    ConstrainOp::Neq => assert_ne!(binary.lhs, binary.rhs),
                }
            }

            if binary.operator.is_commutative() && binary.rhs < binary.lhs {
                std::mem::swap(&mut binary.rhs, &mut binary.lhs);
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ConstrainOp {
    Eq,
    Neq,
    //Cmp...
}

//adapted from LLVM IR
#[allow(dead_code)] //Some enums are not used yet, allow dead_code should be removed once they are all implemented.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Operation {
    Binary(Binary),
    Cast(NodeId),                                                 //convert type
    Truncate { value: NodeId, bit_size: u32, max_bit_size: u32 }, //truncate

    Not(NodeId), //(!) Bitwise Not

    //control flow
    Jne(NodeId, BlockId), //jump on not equal
    Jeq(NodeId, BlockId), //jump on equal
    Jmp(BlockId),         //unconditional jump
    Phi { root: NodeId, block_args: Vec<(NodeId, BlockId)> },
    Call(noirc_frontend::node_interner::FuncId, Vec<NodeId>, Vec<ArrayId>), //Call a function
    Return(Vec<NodeId>), //Return value(s) from a function block
    Result { call_instruction: NodeId, index: u32 }, //Get result index n from a function call

    Load { array_id: ArrayId, index: NodeId },
    Store { array_id: ArrayId, index: NodeId, value: NodeId },

    Intrinsic(OPCODE, Vec<NodeId>), //Custom implementation of usefull primitives which are more performant with Aztec backend

    Nop, // no op
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum Opcode {
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
    Constrain(ConstrainOp),

    Cast,     //convert type
    Truncate, //truncate
    Not,      //(!) Bitwise Not

    //control flow
    Jne, //jump on not equal
    Jeq, //jump on equal
    Jmp, //unconditional jump
    Phi,

    Call(noirc_frontend::node_interner::FuncId), //Call a function
    Return,                                      //Return value(s) from a function block
    Results,                                     //Get result(s) from a function call

    //memory
    Load(ArrayId),
    Store(ArrayId),
    Intrinsic(OPCODE), //Custom implementation of usefull primitives which are more performant with Aztec backend
    Nop,               // no op
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Binary {
    pub lhs: NodeId,
    pub rhs: NodeId,
    pub operator: BinaryOp,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum BinaryOp {
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
    Udiv, //(/) unsigned division
    Sdiv, //(/) signed division
    #[allow(dead_code)]
    Urem, //(%) modulo; remainder of unsigned division
    #[allow(dead_code)]
    Srem, //(%) remainder of signed division
    Div, //(/) field division
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
    Shr, //(>>) Shift right

    Assign,
    Constrain(ConstrainOp, Span, ModuleId), //write gates enforcing the ContrainOp to be true
}

impl Binary {
    fn new(operator: BinaryOp, lhs: NodeId, rhs: NodeId) -> Binary {
        Binary { operator, lhs, rhs }
    }

    pub fn from_hir(
        op_kind: HirBinaryOpKind,
        op_type: ObjectType,
        lhs: NodeId,
        rhs: NodeId,
    ) -> Binary {
        let operator = match op_kind {
            HirBinaryOpKind::Add => BinaryOp::Add,
            HirBinaryOpKind::Subtract => {
                BinaryOp::Sub { max_rhs_value: BigUint::from_u8(0).unwrap() }
            }
            HirBinaryOpKind::Multiply => BinaryOp::Mul,
            HirBinaryOpKind::Equal => BinaryOp::Eq,
            HirBinaryOpKind::NotEqual => BinaryOp::Ne,
            HirBinaryOpKind::And => BinaryOp::And,
            HirBinaryOpKind::Or => BinaryOp::Or,
            HirBinaryOpKind::Xor => BinaryOp::Xor,
            HirBinaryOpKind::Divide => {
                let num_type: NumericType = op_type.into();
                match num_type {
                    NumericType::Signed(_) => BinaryOp::Sdiv,
                    NumericType::Unsigned(_) => BinaryOp::Udiv,
                    NumericType::NativeField => BinaryOp::Div,
                }
            }
            HirBinaryOpKind::Less => {
                let num_type: NumericType = op_type.into();
                match num_type {
                    NumericType::Signed(_) => BinaryOp::Slt,
                    NumericType::Unsigned(_) => BinaryOp::Ult,
                    NumericType::NativeField => BinaryOp::Lt,
                }
            }
            HirBinaryOpKind::LessEqual => {
                let num_type: NumericType = op_type.into();
                match num_type {
                    NumericType::Signed(_) => BinaryOp::Sle,
                    NumericType::Unsigned(_) => BinaryOp::Ule,
                    NumericType::NativeField => BinaryOp::Lte,
                }
            }
            HirBinaryOpKind::Greater => {
                let num_type: NumericType = op_type.into();
                match num_type {
                    NumericType::Signed(_) => return Binary::new(BinaryOp::Slt, rhs, lhs),
                    NumericType::Unsigned(_) => return Binary::new(BinaryOp::Ult, rhs, lhs),
                    NumericType::NativeField => return Binary::new(BinaryOp::Lt, rhs, lhs),
                }
            }
            HirBinaryOpKind::GreaterEqual => {
                let num_type: NumericType = op_type.into();
                match num_type {
                    NumericType::Signed(_) => return Binary::new(BinaryOp::Sle, rhs, lhs),
                    NumericType::Unsigned(_) => return Binary::new(BinaryOp::Ule, rhs, lhs),
                    NumericType::NativeField => return Binary::new(BinaryOp::Lte, rhs, lhs),
                }
            }
            HirBinaryOpKind::Assign => BinaryOp::Assign,
            HirBinaryOpKind::Shl => BinaryOp::Shl,
            HirBinaryOpKind::Shr => BinaryOp::Shr,
        };

        Binary::new(operator, lhs, rhs)
    }

    fn evaluate<F>(
        &self,
        ctx: &SsaContext,
        id: NodeId,
        res_type: ObjectType,
        mut eval_fn: F,
    ) -> NodeEval
    where
        F: FnMut(&SsaContext, NodeId) -> NodeEval,
    {
        let l_eval = eval_fn(ctx, self.lhs);
        let r_eval = eval_fn(ctx, self.rhs);
        let l_type = ctx.get_object_type(self.lhs);
        let r_type = ctx.get_object_type(self.rhs);

        let lhs = l_eval.into_const_value();
        let rhs = r_eval.into_const_value();

        let l_is_zero = lhs.map_or(false, |x| x.is_zero());
        let r_is_zero = rhs.map_or(false, |x| x.is_zero());

        match &self.operator {
            BinaryOp::Add | BinaryOp::SafeAdd => {
                if l_is_zero {
                    return r_eval;
                } else if r_is_zero {
                    return l_eval;
                }
                assert_eq!(l_type, r_type);
                if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return wrapping(lhs, rhs, l_type, u128::add, Add::add);
                }
                //if only one is const, we could try to do constant propagation but this will be handled by the arithmetization step anyways
                //so it is probably not worth it.
                //same for x+x vs 2*x
            }
            BinaryOp::Sub { .. } | BinaryOp::SafeSub { .. } => {
                if r_is_zero {
                    return l_eval;
                }
                if self.lhs == self.rhs {
                    return NodeEval::from_u128(0, res_type);
                }
                //constant folding
                if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return wrapping(lhs, rhs, res_type, u128::wrapping_sub, Sub::sub);
                }
            }
            BinaryOp::Mul | BinaryOp::SafeMul => {
                let l_is_one = lhs.map_or(false, |x| x.is_one());
                let r_is_one = rhs.map_or(false, |x| x.is_one());
                assert_eq!(l_type, r_type);
                if l_is_zero || r_is_one {
                    return l_eval;
                } else if r_is_zero || l_is_one {
                    return r_eval;
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return wrapping(lhs, rhs, res_type, u128::mul, Mul::mul);
                }
                //if only one is const, we could try to do constant propagation but this will be handled by the arithmetization step anyways
                //so it is probably not worth it.
            }

            BinaryOp::Udiv => {
                if r_is_zero {
                    todo!("Panic - division by zero");
                } else if l_is_zero {
                    return l_eval; //TODO should we ensure rhs != 0 ???
                }
                //constant folding
                else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    let lhs = res_type.field_to_type(lhs).to_u128();
                    let rhs = res_type.field_to_type(rhs).to_u128();
                    return NodeEval::Const(FieldElement::from(lhs / rhs), res_type);
                }
            }
            BinaryOp::Div => {
                if r_is_zero {
                    todo!("Panic - division by zero");
                } else if l_is_zero {
                    return l_eval; //TODO should we ensure rhs != 0 ???
                }
                //constant folding - TODO
                else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return NodeEval::Const(lhs / rhs, res_type);
                }
            }
            BinaryOp::Sdiv => {
                if r_is_zero {
                    todo!("Panic - division by zero");
                } else if l_is_zero {
                    return l_eval; //TODO should we ensure rhs != 0 ???
                }
                //constant folding...TODO
                else if lhs.is_some() && rhs.is_some() {
                    todo!("Constant folding for division");
                }
            }
            BinaryOp::Urem | BinaryOp::Srem => {
                if r_is_zero {
                    todo!("Panic - division by zero");
                } else if l_is_zero {
                    return l_eval; //TODO what is the correct result?
                }
                //constant folding - TODO
                else if lhs.is_some() && rhs.is_some() {
                    todo!("divide lhs/rhs but take sign into account");
                }
            }
            BinaryOp::Ult => {
                if r_is_zero {
                    return NodeEval::Const(FieldElement::zero(), ObjectType::Boolean);
                    //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    assert_ne!(res_type, ObjectType::NativeField); //comparisons are not implemented for field elements
                    let res = if lhs < rhs { FieldElement::one() } else { FieldElement::zero() };
                    return NodeEval::Const(res, ObjectType::Boolean);
                }
            }
            BinaryOp::Ule => {
                if l_is_zero {
                    return NodeEval::Const(FieldElement::one(), ObjectType::Boolean);
                    //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    assert_ne!(res_type, ObjectType::NativeField); //comparisons are not implemented for field elements
                    let res = if lhs <= rhs { FieldElement::one() } else { FieldElement::zero() };
                    return NodeEval::Const(res, ObjectType::Boolean);
                }
            }
            BinaryOp::Slt => (),
            BinaryOp::Sle => (),
            BinaryOp::Lt => {
                if r_is_zero {
                    return NodeEval::Const(FieldElement::zero(), ObjectType::Boolean);
                    //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    let res = if lhs < rhs { FieldElement::one() } else { FieldElement::zero() };
                    return NodeEval::Const(res, ObjectType::Boolean);
                }
            }
            BinaryOp::Lte => {
                if l_is_zero {
                    return NodeEval::Const(FieldElement::one(), ObjectType::Boolean);
                    //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    let res = if lhs <= rhs { FieldElement::one() } else { FieldElement::zero() };
                    return NodeEval::Const(res, ObjectType::Boolean);
                }
            }
            BinaryOp::Eq => {
                if self.lhs == self.rhs {
                    return NodeEval::Const(FieldElement::one(), ObjectType::Boolean);
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    if lhs == rhs {
                        return NodeEval::Const(FieldElement::one(), ObjectType::Boolean);
                    } else {
                        return NodeEval::Const(FieldElement::zero(), ObjectType::Boolean);
                    }
                }
            }
            BinaryOp::Ne => {
                if self.lhs == self.rhs {
                    return NodeEval::Const(FieldElement::zero(), ObjectType::Boolean);
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    if lhs != rhs {
                        return NodeEval::Const(FieldElement::one(), ObjectType::Boolean);
                    } else {
                        return NodeEval::Const(FieldElement::zero(), ObjectType::Boolean);
                    }
                }
            }
            BinaryOp::And => {
                //Bitwise AND
                if l_is_zero || self.lhs == self.rhs {
                    return l_eval;
                } else if r_is_zero {
                    return r_eval;
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return wrapping(lhs, rhs, res_type, u128::bitand, field_op_not_allowed);
                }
                //TODO if boolean and not zero, also checks this is correct for field elements
            }
            BinaryOp::Or => {
                //Bitwise OR
                if l_is_zero || self.lhs == self.rhs {
                    return r_eval;
                } else if r_is_zero {
                    return l_eval;
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return wrapping(lhs, rhs, res_type, u128::bitor, field_op_not_allowed);
                }
                //TODO if boolean and not zero, also checks this is correct for field elements
            }
            BinaryOp::Xor => {
                if self.lhs == self.rhs {
                    return NodeEval::Const(FieldElement::zero(), res_type);
                } else if l_is_zero {
                    return r_eval;
                } else if r_is_zero {
                    return l_eval;
                } else if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return wrapping(lhs, rhs, res_type, u128::bitxor, field_op_not_allowed);
                }
                //TODO handle case when lhs is one (or rhs is one) by generating 'not rhs' instruction (or 'not lhs' instruction)
            }
            BinaryOp::Shl => {
                if l_is_zero {
                    return l_eval;
                }
                if r_is_zero {
                    return l_eval;
                }
                if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return wrapping(lhs, rhs, res_type, u128::shl, field_op_not_allowed);
                }
            }
            BinaryOp::Shr => {
                if l_is_zero {
                    return l_eval;
                }
                if r_is_zero {
                    return l_eval;
                }
                if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    return wrapping(lhs, rhs, res_type, u128::shr, field_op_not_allowed);
                }
            }
            BinaryOp::Constrain(op, ..) => {
                if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    let lhs = l_type.field_to_type(lhs);
                    let rhs = r_type.field_to_type(rhs);
                    match op {
                        ConstrainOp::Eq => assert_eq!(lhs, rhs),
                        ConstrainOp::Neq => assert_ne!(lhs, rhs),
                    }
                    //we can delete the instruction
                    return NodeEval::VarOrInstruction(NodeId::dummy());
                }
            }
            BinaryOp::Assign => (),
        }
        NodeEval::VarOrInstruction(id)
    }

    fn truncate_required(&self) -> bool {
        match &self.operator {
            BinaryOp::Add => false,
            BinaryOp::SafeAdd => false,
            BinaryOp::Sub { .. } => false,
            BinaryOp::SafeSub { .. } => false,
            BinaryOp::Mul => false,
            BinaryOp::SafeMul => false,
            BinaryOp::Udiv => true,
            BinaryOp::Sdiv => true,
            BinaryOp::Urem => true,
            BinaryOp::Srem => true,
            BinaryOp::Div => false,
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
            BinaryOp::Constrain(..) => true,
            BinaryOp::Assign => false,
            BinaryOp::Shl => true,
            BinaryOp::Shr => true,
        }
    }

    pub fn opcode(&self) -> Opcode {
        match &self.operator {
            BinaryOp::Add => Opcode::Add,
            BinaryOp::SafeAdd => Opcode::SafeAdd,
            BinaryOp::Sub { .. } => Opcode::Sub,
            BinaryOp::SafeSub { .. } => Opcode::SafeSub,
            BinaryOp::Mul => Opcode::Mul,
            BinaryOp::SafeMul => Opcode::SafeMul,
            BinaryOp::Udiv => Opcode::Udiv,
            BinaryOp::Sdiv => Opcode::Sdiv,
            BinaryOp::Urem => Opcode::Urem,
            BinaryOp::Srem => Opcode::Srem,
            BinaryOp::Div => Opcode::Div,
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
            BinaryOp::Shr => Opcode::Shr,
            BinaryOp::Assign => Opcode::Assign,
            BinaryOp::Constrain(op, span, module) => Opcode::Constrain(*op),
        }
    }
}

/// Perform the given numeric operation and modulo the result by the max value for the given bitcount
/// if the res_type is not a NativeField.
fn wrapping(
    lhs: FieldElement,
    rhs: FieldElement,
    res_type: ObjectType,
    u128_op: impl FnOnce(u128, u128) -> u128,
    field_op: impl FnOnce(FieldElement, FieldElement) -> FieldElement,
) -> NodeEval {
    if res_type != ObjectType::NativeField {
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
    pub fn binary(op: BinaryOp, lhs: NodeId, rhs: NodeId) -> Self {
        Operation::Binary(Binary::new(op, lhs, rhs))
    }

    pub fn is_dummy_store(&self) -> bool {
        match self {
            Operation::Store { index, value, .. } => {
                *index == NodeId::dummy() && *value == NodeId::dummy()
            }
            _ => false,
        }
    }

    pub fn map_id(&self, mut f: impl FnMut(NodeId) -> NodeId) -> Operation {
        use Operation::*;
        match self {
            Binary(self::Binary { lhs, rhs, operator }) => {
                Binary(self::Binary { lhs: f(*lhs), rhs: f(*rhs), operator: operator.clone() })
            }
            Cast(value) => Cast(f(*value)),
            Truncate { value, bit_size, max_bit_size } => {
                Truncate { value: f(*value), bit_size: *bit_size, max_bit_size: *max_bit_size }
            }
            Not(id) => Not(f(*id)),
            Jne(id, block) => Jne(f(*id), *block),
            Jeq(id, block) => Jeq(f(*id), *block),
            Jmp(block) => Jmp(*block),
            Phi { root, block_args } => Phi {
                root: f(*root),
                block_args: vecmap(block_args, |(id, block)| (f(*id), *block)),
            },
            Load { array_id: array, index } => Load { array_id: *array, index: f(*index) },
            Store { array_id: array, index, value } => {
                Store { array_id: *array, index: f(*index), value: f(*value) }
            }
            Intrinsic(i, args) => Intrinsic(*i, vecmap(args.iter().copied(), f)),
            Nop => Nop,
            Call(func_id, args, returned_array) => {
                Call(*func_id, vecmap(args.iter().copied(), f), returned_array.clone())
            }
            Return(values) => Return(vecmap(values.iter().copied(), f)),
            Result { call_instruction, index } => {
                Result { call_instruction: f(*call_instruction), index: *index }
            }
        }
    }

    /// Mutate each contained NodeId in place using the given function f
    pub fn map_id_mut(&mut self, mut f: impl FnMut(NodeId) -> NodeId) {
        use Operation::*;
        match self {
            Binary(self::Binary { lhs, rhs, .. }) => {
                *lhs = f(*lhs);
                *rhs = f(*rhs);
            }
            Cast(value) => *value = f(*value),
            Truncate { value, .. } => *value = f(*value),
            Not(id) => *id = f(*id),
            Jne(id, _) => *id = f(*id),
            Jeq(id, _) => *id = f(*id),
            Jmp(_) => (),
            Phi { root, block_args } => {
                f(*root);
                for (id, _block) in block_args {
                    *id = f(*id);
                }
            }
            Load { index, .. } => *index = f(*index),
            Store { index, value, .. } => {
                *index = f(*index);
                *value = f(*value);
            }
            Intrinsic(_, args) => {
                for arg in args {
                    *arg = f(*arg);
                }
            }
            Nop => (),
            Call(_, args, _) => {
                for arg in args {
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
    pub fn for_each_id(&self, mut f: impl FnMut(NodeId)) {
        use Operation::*;
        match self {
            Binary(self::Binary { lhs, rhs, .. }) => {
                f(*lhs);
                f(*rhs);
            }
            Cast(value) => f(*value),
            Truncate { value, .. } => f(*value),
            Not(id) => f(*id),
            Jne(id, _) => f(*id),
            Jeq(id, _) => f(*id),
            Jmp(_) => (),
            Phi { root, block_args } => {
                f(*root);
                for (id, _block) in block_args {
                    f(*id);
                }
            }
            Load { index, .. } => f(*index),
            Store { index, value, .. } => {
                f(*index);
                f(*value);
            }
            Intrinsic(_, args) => args.iter().copied().for_each(f),
            Nop => (),
            Call(_, args, _) => args.iter().copied().for_each(f),
            Return(values) => values.iter().copied().for_each(f),
            Result { call_instruction, .. } => {
                f(*call_instruction);
            }
        }
    }

    pub fn opcode(&self) -> Opcode {
        match self {
            Operation::Binary(binary) => binary.opcode(),
            Operation::Cast(_) => Opcode::Cast,
            Operation::Truncate { .. } => Opcode::Truncate,
            Operation::Not(_) => Opcode::Not,
            Operation::Jne(_, _) => Opcode::Jne,
            Operation::Jeq(_, _) => Opcode::Jeq,
            Operation::Jmp(_) => Opcode::Jmp,
            Operation::Phi { .. } => Opcode::Phi,
            Operation::Call(id, _, _) => Opcode::Call(*id),
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
                // This isn't a match-all pattern in case more ops are ever added
                // that aren't commutative
                | BinaryOp::Constrain(ConstrainOp::Eq | ConstrainOp::Neq, ..)
        )
    }
}

pub fn get_witness_from_object(obj: &Object) -> Option<Witness> {
    match obj {
        Object::Integer(i) => Some(i.witness),
        Object::Array(_) => unreachable!("Array has multiple witnesses"),
        _ => obj.witness(),
    }
}
