use std::convert::TryInto;
use std::ops::Add;

use acvm::acir::native_types::Witness;
use acvm::acir::OPCODE;
use acvm::FieldElement;
use arena;
use noirc_frontend::hir_def::expr::HirBinaryOpKind;
use noirc_frontend::node_interner::DefinitionId;
use noirc_frontend::util::vecmap;
use noirc_frontend::{Signedness, Type};
use num_bigint::BigUint;
use num_traits::{FromPrimitive, One};

use crate::object::Object;
use std::ops::Mul;

use super::block::BlockId;
use super::context::SsaContext;

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
    pub fn set_id(&mut self, new_id: NodeId) {
        match self {
            NodeObj::Obj(obj) => obj.id = new_id,
            NodeObj::Instr(inst) => inst.id = new_id,
            NodeObj::Const(c) => c.id = new_id,
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
    Pointer(u32),  //array index
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
            Type::FieldElement(_) => ObjectType::NativeField,
            Type::Integer(_ftype, sign, bit_size) => {
                assert!(
                    *bit_size < super::integer::short_integer_max_bit_size(),
                    "long integers are not yet supported"
                );
                match sign {
                    Signedness::Signed => ObjectType::Signed(*bit_size),
                    Signedness::Unsigned => ObjectType::Unsigned(*bit_size),
                }
            }
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
}

#[derive(Clone, Debug)]
pub struct Instruction {
    pub id: NodeId,
    pub operator: Operation,
    pub res_type: ObjectType, //result type
    pub parent_block: BlockId,
    pub res_name: String,

    /// Set to Some(id) if this instruction is to be replaced by another
    pub replacement: Option<NodeId>,
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

    /// Returns is_zero, the field value if known, and the bitcount
    pub fn evaluate(&self) -> (bool, Option<u128>, u32) {
        match self {
            &NodeEval::Const(c, t) => {
                let (value, bitcount) = field_to_u128(c, t);
                (c.is_zero(), Some(value), bitcount)
            }
            _ => (false, None, 0),
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

//Returns the field element as i128 and the bit size of the constant node
pub fn field_to_u128(c: FieldElement, ctype: ObjectType) -> (u128, u32) {
    match ctype {
        ObjectType::Boolean => (if c.is_zero() { 0 } else { 1 }, 1),
        ObjectType::NativeField => {
            (c.to_u128(), 256) //TODO: handle elements that do not fit in 128 bits
        }
        ObjectType::Signed(b) | ObjectType::Unsigned(b) => {
            assert!(b < 128); //we do not support integers bigger than 128 bits for now.
            (c.to_u128(), b)
        } //TODO check how to handle signed integers
        _ => todo!(),
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
            operator: op_code,
            res_type: r_type,
            res_name: String::new(),
            parent_block: p_block,
            replacement: None,
        }
    }

    /// Indicates whether the left and/or right operand of the instruction is required to be truncated to its bit-width
    pub fn truncate_required(&self, cast_operation_lhs_bits: u32) -> bool {
        match &self.operator {
            Operation::Binary(binary) => binary.truncate_required(),
            Operation::Not(..) => true,
            Operation::Cast(..) => {
                self.res_type.bits() > cast_operation_lhs_bits
            }
            Operation::Truncate { .. } | Operation::Phi { .. } => false,
            Operation::Nop | Operation::Jne(..) | Operation::Jeq(..) | Operation::Jmp(..) => {
                false
            }
            Operation::Load { .. } | Operation::Store { .. } => false,
            Operation::Intrinsic(_, _) => true, //TODO to check
            Operation::Call(_, _) => false, //return values are in the return statment, should we truncate function arguments? probably but not lhs and rhs anyways.
            Operation::Return(_) => true,
            Operation::Results { .. } => false,
        }
    }

    pub fn evaluate(&self, ctx: &SsaContext) -> NodeEval {
        self.evaluate_with(ctx, NodeEval::from_id)
    }

    //Evaluate the instruction value when its operands are constant (constant folding)
    pub fn evaluate_with<F>(&self, ctx: &SsaContext, eval_fn: F) -> NodeEval
    where
        F: FnMut(&SsaContext, NodeId) -> NodeEval,
    {
        match &self.operator {
            Operation::Binary(binary) => {
                return binary.evaluate(ctx, self.id, self.res_type, eval_fn)
            }
            Operation::Cast(value) => {
                if let Some(l_const) = ctx.get_const_value(*value) {
                    if self.res_type == ObjectType::NativeField {
                        return NodeEval::Const(FieldElement::from(l_const), self.res_type);
                    }
                    return NodeEval::Const(
                        FieldElement::from(l_const % (1_u128 << self.res_type.bits())),
                        self.res_type,
                    );
                }
            }
            Operation::Not(value) => {
                if let Some(l_const) = ctx.get_const_value(*value) {
                    return NodeEval::Const(FieldElement::from(!l_const), self.res_type);
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

    /// Delete this instruciton by mutating it into a Operation::Nop
    pub fn delete(&mut self) {
        self.operator = Operation::Nop;
    }

    pub fn is_deleted(&self) -> bool {
        self.operator == Operation::Nop || self.replacement.is_some()
    }

    pub fn standard_form(&mut self) {
        match &mut self.operator {
            Operation::Binary(binary) => {
                if let BinaryOp::Constrain(op) = &binary.operator {
                    match op {
                        ConstrainOp::Eq => {
                            if binary.rhs == binary.rhs {
                                self.delete();
                                return;
                            }
                        }
                        // TODO: Why are we asserting here?
                        ConstrainOp::Neq => assert_ne!(binary.lhs, binary.rhs),
                    }
                }

                if binary.operator.is_commutative() && binary.rhs < binary.lhs {
                    std::mem::swap(&mut binary.rhs, &mut binary.lhs);
                }
            }
            _ => (),
        }
    }
}

impl<'c> SsaContext<'c> {
    fn get_const_value(&self, id: NodeId) -> Option<u128> {
        match &self[id] {
            NodeObj::Const(c) => c.value.clone().try_into().ok(),
            _ => None,
        }
    }

    fn get_value_and_bitsize(&self, id: NodeId) -> Option<(FieldElement, u32)> {
        match &self[id] {
            NodeObj::Const(c) => {
                let value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be());
                Some((value, c.value_type.bits()))
            }
            _ => None,
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
    Cast(NodeId), //convert type
    Truncate {
        value: NodeId,
        bit_size: u32,
        max_bit_size: u32,
    }, //truncate

    Not(NodeId), //(!) Bitwise Not

    //control flow
    Jne(NodeId, BlockId), //jump on not equal
    Jeq(NodeId, BlockId), //jump on equal
    Jmp(BlockId),         //unconditional jump
    Phi {
        root: NodeId,
        block_args: Vec<(NodeId, BlockId)>,
    },

    Call(noirc_frontend::node_interner::FuncId, Vec<NodeId>), //Call a function
    Return(Vec<NodeId>), //Return value(s) from a function block
    Results {
        call_instruction: NodeId,
        results: Vec<NodeId>,
    }, //Get result(s) from a function call

    //memory
    Load {
        array: u32,
        index: NodeId,
    },
    Store {
        array: u32,
        index: NodeId,
        value: NodeId,
    },

    Intrinsic(OPCODE, Vec<NodeId>), //Custom implementation of usefull primitives which are more performant with Aztec backend

    Nop, // no op
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Binary {
    pub lhs: NodeId,
    pub rhs: NodeId,
    pub operator: BinaryOp,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum BinaryOp {
    Add,                                //(+)
    SafeAdd,                            //(+) safe addition
    Sub { max_rhs_value: BigUint },     //(-)
    SafeSub { max_rhs_value: BigUint }, //(-) safe subtraction
    Mul,                                //(*)
    SafeMul,                            //(*) safe multiplication
    Udiv,                               //(/) unsigned division
    Sdiv,                               //(/) signed division
    Urem,                               //(%) modulo; remainder of unsigned division
    Srem,                               //(%) remainder of signed division
    Div,                                //(/) field division
    Eq,                                 //(==) equal
    Ne,                                 //(!=) not equal
    Ult,                                //(<) unsigned less than
    Ule,                                //(<=) unsigned less or equal
    Slt,                                //(<) signed less than
    Sle,                                //(<=) signed less or equal
    Lt,                                 //(<) field less
    Lte,                                //(<=) field less or equal
    And,                                //(&) Bitwise And
    Or,                                 //(|) Bitwise Or
    Xor,                                //(^) Bitwise Xor

    Assign,
    Constrain(ConstrainOp), //write gates enforcing the ContrainOp to be true
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
            HirBinaryOpKind::Subtract => BinaryOp::Sub {
                max_rhs_value: BigUint::from_u8(0).unwrap(),
            },
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
                    NumericType::Unsigned(_) => BinaryOp::Ult,
                    NumericType::NativeField => BinaryOp::Lte,
                }
            }
            HirBinaryOpKind::Greater => {
                let num_type: NumericType = op_type.into();
                match num_type {
                    NumericType::Signed(_) => return Binary::new(BinaryOp::Sle, rhs, lhs),
                    NumericType::Unsigned(_) => return Binary::new(BinaryOp::Ule, rhs, lhs),
                    NumericType::NativeField => return Binary::new(BinaryOp::Lte, rhs, lhs),
                }
            }
            HirBinaryOpKind::GreaterEqual => {
                let num_type: NumericType = op_type.into();
                match num_type {
                    NumericType::Signed(_) => return Binary::new(BinaryOp::Slt, rhs, lhs),
                    NumericType::Unsigned(_) => return Binary::new(BinaryOp::Ult, rhs, lhs),
                    NumericType::NativeField => return Binary::new(BinaryOp::Lt, rhs, lhs),
                }
            }
            HirBinaryOpKind::Assign => BinaryOp::Assign,
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
        let lhs = ctx.get_value_and_bitsize(self.lhs);
        let rhs = ctx.get_value_and_bitsize(self.rhs);

        let l_is_zero = lhs.map_or(false, |x| x.0.is_zero());
        let r_is_zero = rhs.map_or(false, |x| x.0.is_zero());

        let l_eval = eval_fn(ctx, self.lhs);
        let r_eval = eval_fn(ctx, self.rhs);

        match &self.operator {
            BinaryOp::Add | BinaryOp::SafeAdd => {
                if l_is_zero {
                    return r_eval;
                } else if r_is_zero {
                    return l_eval;
                }

                if let (Some((lhs, l_bits)), Some((rhs, r_bits))) = (lhs, rhs) {
                    //constant folding
                    if l_bits == 256 {
                        //NO modulo for field elements - May be we should have a different opcode?
                        return NodeEval::Const(lhs + rhs, res_type);
                    }
                    assert_eq!(l_bits, r_bits);
                    return wrapping(lhs, rhs, l_bits, res_type, u128::add);
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
                if let (Some((lhs, l_bits)), Some((rhs, r_bits))) = (lhs, rhs) {
                    if l_bits == 256 {
                        //NO modulo for field elements - May be we should have a different opcode?
                        return NodeEval::Const(lhs - rhs, res_type);
                    }
                    assert_eq!(l_bits, r_bits);
                    return wrapping(lhs, rhs, l_bits, res_type, u128::wrapping_sub);
                }
            }
            BinaryOp::Mul | BinaryOp::SafeMul => {
                let l_is_one = lhs.map_or(false, |x| x.0.is_one());
                let r_is_one = rhs.map_or(false, |x| x.0.is_one());

                if l_is_zero || r_is_one {
                    return l_eval;
                } else if r_is_zero || l_is_one {
                    return r_eval;
                } else if let (Some((lhs, l_bits)), Some((rhs, r_bits))) = (lhs, rhs) {
                    //constant folding
                    if l_bits == 256 {
                        //NO modulo for field elements - May be we should have a different opcode?
                        return NodeEval::Const(lhs * rhs, res_type);
                    }

                    assert_eq!(l_bits, r_bits);
                    return wrapping(lhs, rhs, l_bits, res_type, u128::mul);
                }
                //if only one is const, we could try to do constant propagation but this will be handled by the arithmetization step anyways
                //so it is probably not worth it.
            }
            BinaryOp::Udiv | BinaryOp::Sdiv | BinaryOp::Div => {
                if r_is_zero {
                    todo!("Panic - division by zero");
                } else if l_is_zero {
                    return l_eval;
                }
                //constant folding - TODO
                else if lhs.is_some() && rhs.is_some() {
                    todo!();
                } else if rhs.is_some() {
                    //same as lhs*1/r
                    todo!();
                    //return (Some(self.lhs), None, None);
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
                } else if let (Some((lhs, l_bits)), Some((rhs, _))) = (lhs, rhs) {
                    assert!(l_bits < 256); //comparisons are not implemented for field elements
                    let res = if lhs < rhs {
                        FieldElement::one()
                    } else {
                        FieldElement::zero()
                    };
                    return NodeEval::Const(res, ObjectType::Boolean);
                }
            }
            BinaryOp::Ule => {
                if l_is_zero {
                    return NodeEval::Const(FieldElement::one(), ObjectType::Boolean);
                    //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this
                } else if let (Some((lhs, l_bits)), Some((rhs, _))) = (lhs, rhs) {
                    assert!(l_bits < 256); //comparisons are not implemented for field elements
                    let res = if lhs <= rhs {
                        FieldElement::one()
                    } else {
                        FieldElement::zero()
                    };
                    return NodeEval::Const(res, ObjectType::Boolean);
                }
            }
            BinaryOp::Eq => {
                if self.lhs == self.rhs {
                    return NodeEval::Const(FieldElement::one(), ObjectType::Boolean);
                } else if let (Some((lhs, _)), Some((rhs, _))) = (lhs, rhs) {
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
                } else if let (Some((lhs, _)), Some((rhs, _))) = (lhs, rhs) {
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
                } else if let (Some((lhs, _)), Some((rhs, _))) = (lhs, rhs) {
                    return NodeEval::from_u128(lhs.to_u128() & rhs.to_u128(), res_type);
                }
                //TODO if boolean and not zero, also checks this is correct for field elements
            }
            BinaryOp::Or => {
                //Bitwise OR
                if l_is_zero || self.lhs == self.rhs {
                    return r_eval;
                } else if r_is_zero {
                    return l_eval;
                } else if let (Some((lhs, _)), Some((rhs, _))) = (lhs, rhs) {
                    return NodeEval::from_u128(lhs.to_u128() | rhs.to_u128(), res_type);
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
                } else if let (Some((lhs, _)), Some((rhs, _))) = (lhs, rhs) {
                    return NodeEval::from_u128(lhs.to_u128() ^ rhs.to_u128(), res_type);
                }
                //TODO handle case when lhs is one (or rhs is one) by generating 'not rhs' instruction (or 'not lhs' instruction)
            }
            BinaryOp::Constrain(op) => {
                if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    match op {
                        ConstrainOp::Eq => assert_eq!(lhs, rhs),
                        ConstrainOp::Neq => assert_ne!(lhs, rhs),
                    }
                    //we can delete the instruction
                    return NodeEval::VarOrInstruction(NodeId::dummy());
                }
            }
            BinaryOp::Slt => (),
            BinaryOp::Sle => (),
            BinaryOp::Lt => (),
            BinaryOp::Lte => (),
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
        }
    }
}

/// Perform f(lhs, rhs) and modulo the result by the max value for the given bitcount.
fn wrapping<F>(
    lhs: FieldElement,
    rhs: FieldElement,
    bitcount: u32,
    res_type: ObjectType,
    f: F,
) -> NodeEval
where
    F: FnOnce(u128, u128) -> u128,
{
    let mut x = f(lhs.to_u128(), rhs.to_u128());
    if bitcount != 256 {
        x = x % (1_u128 << bitcount);
    }
    NodeEval::from_u128(x, res_type)
}

impl Operation {
    pub fn is_binary(&self) -> bool {
        matches!(self, Operation::Binary(..))
    }

    pub fn binary(op: BinaryOp, lhs: NodeId, rhs: NodeId) -> Self {
        Operation::Binary(Binary::new(op, lhs, rhs))
    }

    pub fn map_id(&self, mut f: impl FnMut(NodeId) -> NodeId) -> Operation {
        use Operation::*;
        match self {
            Binary(self::Binary { lhs, rhs, operator }) => Binary(self::Binary {
                lhs: f(*lhs),
                rhs: f(*rhs),
                operator: operator.clone(),
            }),
            Cast(value) => Cast(f(*value)),
            Truncate {
                value,
                bit_size,
                max_bit_size,
            } => Truncate {
                value: f(*value),
                bit_size: *bit_size,
                max_bit_size: *max_bit_size,
            },
            Not(id) => Not(f(*id)),
            Jne(id, block) => Jne(f(*id), *block),
            Jeq(id, block) => Jeq(f(*id), *block),
            Jmp(block) => Jmp(*block),
            Phi { root, block_args } => Phi {
                root: f(*root),
                block_args: vecmap(block_args, |(id, block)| (f(*id), *block)),
            },
            Load { array, index } => Load {
                array: *array,
                index: f(*index),
            },
            Store {
                array,
                index,
                value,
            } => Store {
                array: *array,
                index: f(*index),
                value: f(*value),
            },
            Intrinsic(i, args) => Intrinsic(*i, vecmap(args.iter().copied(), f)),
            Nop => Nop,
            Call(func_id, args) => Call(*func_id, vecmap(args.iter().copied(), f)),
            Return(values) => Return(vecmap(values.iter().copied(), f)),
            Results {
                call_instruction,
                results,
            } => Results {
                call_instruction: f(*call_instruction),
                results: vecmap(results.iter().copied(), f),
            },
        }
    }

    /// Mutate each contained NodeId in place using the given function f
    pub fn map_id_mut(&mut self, mut f: impl FnMut(NodeId) -> NodeId) {
        use Operation::*;
        match self {
            Binary(self::Binary { lhs, rhs, .. }) => {
                *lhs = f(*lhs);
                *rhs = f(*rhs);
            },
            Cast(value) => *value = f(*value),
            Truncate {
                value,
                ..
            } => *value = f(*value),
            Not(id) => *id = f(*id),
            Jne(id, _) => *id = f(*id),
            Jeq(id, _) => *id = f(*id),
            Jmp(_) => (),
            Phi { root, block_args } => {
                f(*root);
                for (id, _block) in block_args {
                    *id = f(*id);
                }
            },
            Load { index, .. } => *index = f(*index),
            Store {
                index,
                value,
                ..
            } => {
                *index = f(*index);
                *value = f(*value);
            },
            Intrinsic(_, args) => {
                for arg in args {
                    *arg = f(*arg);
                }
            },
            Nop => (),
            Call(_, args) => {
                for arg in args {
                    *arg = f(*arg);
                }
            }
            Return(values) => {
                for value in values {
                    *value = f(*value);
                }
            }
            Results {
                call_instruction,
                results,
            } => {
                *call_instruction = f(*call_instruction);
                for result in results {
                    *result = f(*result);
                }
            },
        }
    }

    /// This is the same as map_id but doesn't return a new Operation
    pub fn for_each_id(&self, mut f: impl FnMut(NodeId)) {
        use Operation::*;
        match self {
            Binary(self::Binary { lhs, rhs, .. }) => {
                f(*lhs);
                f(*rhs);
            },
            Cast(value) => f(*value),
            Truncate {
                value,
                ..
            } => f(*value),
            Not(id) => f(*id),
            Jne(id, _) => f(*id),
            Jeq(id, _) => f(*id),
            Jmp(_) => (),
            Phi { root, block_args } => {
                f(*root);
                for (id, _block) in block_args {
                    f(*id);
                }
            },
            Load { index, .. } => f(*index),
            Store {
                index,
                value,
                ..
            } => {
                f(*index);
                f(*value);
            },
            Intrinsic(_, args) => args.iter().copied().for_each(f),
            Nop => (),
            Call(_, args) => args.iter().copied().for_each(f),
            Return(values) => values.iter().copied().for_each(f),
            Results {
                call_instruction,
                results,
            } => {
                f(*call_instruction);
                results.iter().copied().for_each(f);
            },
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
                | BinaryOp::Constrain(ConstrainOp::Eq | ConstrainOp::Neq)
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
