use std::convert::TryInto;
use std::ops::Add;

use acvm::acir::native_types::Witness;
use acvm::acir::OpCode;
use acvm::FieldElement;
use arena;
use noirc_frontend::hir_def::expr::HirBinaryOpKind;
use noirc_frontend::node_interner::IdentId;
use noirc_frontend::{Signedness, Type};
use num_bigint::BigUint;
use num_traits::{One, FromPrimitive};

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

#[derive(Debug)]
pub struct Variable {
    pub id: NodeId,
    pub obj_type: ObjectType,
    pub name: String,
    //pub cur_value: arena::Index, //for generating the SSA form, current value of the object during parsing of the AST
    pub root: Option<NodeId>, //when generating SSA, assignment of an object creates a new one which is linked to the original one
    pub def: Option<IdentId>, //TODO redundant with root - should it be an option?
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
        def: Option<IdentId>,
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
            ObjectType::Pointer(_) => unreachable!(),
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
    pub is_deleted: bool,
    pub res_name: String,
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
            is_deleted: false,
            parent_block: p_block,
        }
    }

    /// Indicates whether the left and/or right operand of the instruction is required to be truncated to its bit-width
    pub fn truncate_required(&self, lhs_bits: u32, rhs_bits: u32) -> (bool, bool) {
        match self.operator {
            Operation::Binary(binary) => binary.truncate_required(),
            Operation::Not(..) => (true, true),
            Operation::Cast(..) => {
                if self.res_type.bits() > lhs_bits {
                    return (true, false);
                }
                (false, false)
            }
            Operation::Truncate { .. } | Operation::Phi { .. } => (false, false),
            Operation::Nop | Operation::Jne | Operation::Jeq | Operation::Jmp => (false, false),
            Operation::Load(_) | Operation::Store(_) => (false, false),
            Operation::Intrinsic(_) => (true, true), //TODO to check
        }
    }

    //Returns the field element as i128 and the bit size of the constant node
    pub fn get_const_value(c: FieldElement, ctype: ObjectType) -> (u128, u32) {
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

    pub fn node_evaluate(n: &NodeEval) -> (bool, Option<u128>, u32) {
        match n {
            &NodeEval::Const(c, t) => {
                let cv = Instruction::get_const_value(c, t);
                (c.is_zero(), Some(cv.0), cv.1)
            }
            _ => (false, None, 0),
        }
    }

    //Evaluate the instruction value when its operands are constant (constant folding)
    pub fn evaluate(&self, l_eval: &NodeEval, r_eval: &NodeEval) -> NodeEval {
        match self.operator {
            Operation::Binary(binary) => return binary.evaluate(l_eval, r_eval, self.id, self.res_type),
            Operation::Cast(..) => {
                let (_, l_constant, _) = Instruction::node_evaluate(l_eval);

                if let Some(l_const) = l_constant {
                    if self.res_type == ObjectType::NativeField {
                        return NodeEval::Const(FieldElement::from(l_const), self.res_type);
                    }
                    return NodeEval::Const(
                        FieldElement::from(l_const % (1_u128 << self.res_type.bits())),
                        self.res_type,
                    );
                }
            }
            Operation::Not(_) => {
                let (_, l_constant, _) = Instruction::node_evaluate(l_eval);
                if let Some(l_const) = l_constant {
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

    pub fn standard_form(&mut self) {
        match &mut self.operator {
            Operation::Binary(binary) => {
                if let BinaryOp::Constrain(op) = &binary.operator {
                    match op {
                        ConstrainOp::Eq => {
                            if binary.rhs == binary.rhs {
                                self.is_deleted = true;
                                self.operator = Operation::Nop;
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
    }, //truncate

    Not(NodeId), //(!) Bitwise Not

    //control flow
    Jne, //jump on not equal
    Jeq, //jump on equal
    Jmp, //unconditional jump
    Phi { block_args: Vec<(NodeId, BlockId)> },

    //memory
    Load(u32),
    Store(u32),

    Intrinsic(OpCode), //Custom implementation of usefull primitives which are more performant with Aztec backend

    Nop, // no op
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Binary {
    lhs: NodeId,
    rhs: NodeId,
    operator: BinaryOp,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum BinaryOp {
    Add,     //(+)
    SafeAdd, //(+) safe addition
    Sub { max_rhs_value: BigUint }, //(-)
    SafeSub { max_rhs_value: BigUint }, //(-) safe subtraction
    Mul,     //(*)
    SafeMul, //(*) safe multiplication
    Udiv,    //(/) unsigned division
    Sdiv,    //(/) signed division
    Urem,    //(%) modulo; remainder of unsigned division
    Srem,    //(%) remainder of signed division
    Div,     //(/) field division
    Eq,      //(==) equal
    Ne,      //(!=) not equal
    Ult,     //(<) unsigned less than
    Ule,     //(<=) unsigned less or equal
    Slt,     //(<) signed less than
    Sle,     //(<=) signed less or equal
    Lt,      //(<) field less
    Lte,     //(<=) field less or equal
    And,     //(&) Bitwise And
    Or,      //(|) Bitwise Or
    Xor,     //(^) Bitwise Xor

    Assign,
    Constrain(ConstrainOp), //write gates enforcing the ContrainOp to be true
}

impl Binary {
    fn new(operator: BinaryOp, lhs: NodeId, rhs: NodeId) -> Binary {
        Binary { operator, lhs, rhs }
    }

    fn evaluate(&self, l_eval: &NodeEval, r_eval: &NodeEval, id: NodeId, res_type: ObjectType) -> NodeEval {
        let (l_is_zero, l_constant, l_bsize) = Instruction::node_evaluate(l_eval);
        let (r_is_zero, r_constant, r_bsize) = Instruction::node_evaluate(r_eval);
        let r_is_const = r_constant.is_some();
        let l_is_const = l_constant.is_some();

        match &self.operator {
            BinaryOp::Add | BinaryOp::SafeAdd => {
                if r_is_zero {
                    return *l_eval;
                } else if l_is_zero {
                    return *r_eval;
                } else if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    //constant folding
                    if l_bsize == 256 {
                        //NO modulo for field elements - May be we should have a different opcode?
                        if let (NodeEval::Const(a, _), NodeEval::Const(b, _)) = (l_eval, r_eval) {
                            let res_value = a.add(*b);
                            return NodeEval::Const(res_value, res_type);
                        }
                        unreachable!();
                    }
                    assert!(l_bsize == r_bsize);
                    let res_value = (l_const + r_const) % (1_u128 << l_bsize) as u128;
                    return NodeEval::Const(FieldElement::from(res_value), res_type);
                }
                //if only one is const, we could try to do constant propagation but this will be handled by the arithmetization step anyways
                //so it is probably not worth it.
                //same for x+x vs 2*x
            }
            BinaryOp::Sub { .. } | BinaryOp::SafeSub { .. } => {
                if r_is_zero {
                    return *l_eval;
                }
                if self.lhs == self.rhs {
                    return NodeEval::Const(FieldElement::zero(), res_type);
                }
                //constant folding
                if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    if l_bsize == 256 {
                        //NO modulo for field elements - May be we should have a different opcode?
                        if let (NodeEval::Const(a, _), NodeEval::Const(b, _)) = (l_eval, r_eval) {
                            let res_value = a.add(-*b);
                            return NodeEval::Const(res_value, res_type);
                        }
                        unreachable!();
                    }
                    //if l_constant.is_some() && r_constant.is_some() {
                    assert_eq!(l_bsize, r_bsize);

                    let res_value =
                        l_const.overflowing_sub(r_const).0 % (1_u128 << l_bsize) as u128;
                    return NodeEval::Const(FieldElement::from(res_value), res_type);
                }
            }
            BinaryOp::Mul | BinaryOp::SafeMul => {
                if r_is_zero {
                    return *r_eval;
                } else if l_is_zero {
                    return *l_eval;
                } else if l_is_const && l_constant.unwrap() == 1 {
                    return *r_eval;
                } else if r_is_const && r_constant.unwrap() == 1 {
                    return *l_eval;
                } else if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    //constant folding
                    if l_bsize == 256 {
                        //NO modulo for field elements - May be we should have a different opcode?
                        if let (NodeEval::Const(a, _), NodeEval::Const(b, _)) = (l_eval, r_eval) {
                            let res_value = a.mul(*b);
                            return NodeEval::Const(res_value, res_type);
                        }
                        unreachable!();
                    }

                    assert!(l_bsize == r_bsize);
                    let res_value = (l_const * r_const) % (1_u128 << l_bsize) as u128;
                    return NodeEval::Const(FieldElement::from(res_value), res_type);
                }
                //if only one is const, we could try to do constant propagation but this will be handled by the arithmetization step anyways
                //so it is probably not worth it.
            }
            BinaryOp::Udiv | BinaryOp::Sdiv | BinaryOp::Div => {
                if r_is_zero {
                    todo!("Panic - division by zero");
                } else if l_is_zero {
                    return *l_eval; //TODO should we ensure rhs != 0 ???
                }
                //constant folding - TODO
                else if l_constant.is_some() && r_constant.is_some() {
                    todo!();
                } else if r_constant.is_some() {
                    //same as lhs*1/r
                    todo!();
                    //return (Some(self.lhs), None, None);
                }
            }
            BinaryOp::Urem | BinaryOp::Srem => {
                if r_is_zero {
                    todo!("Panic - division by zero");
                } else if l_is_zero {
                    return *l_eval; //TODO what is the correct result? and should we ensure rhs != 0 ???
                }
                //constant folding - TODO
                else if l_constant.is_some() && r_constant.is_some() {
                    todo!("divide l_constant/r_constant but take sign into account");
                }
            }
            BinaryOp::Ult => {
                if r_is_zero {
                    return NodeEval::Const(FieldElement::zero(), ObjectType::Boolean);
                    //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this
                } else if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    assert!(l_bsize < 256); //comparisons are not implemented for field elements
                    let res = if l_const < r_const {
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
                } else if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    assert!(l_bsize < 256); //comparisons are not implemented for field elements
                    let res = if l_const <= r_const {
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
                } else if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    if l_bsize == 256 {
                        if let (NodeEval::Const(a, _), NodeEval::Const(b, _)) = (l_eval, r_eval) {
                            if a == b {
                                return NodeEval::Const(FieldElement::one(), ObjectType::Boolean);
                            } else {
                                return NodeEval::Const(FieldElement::zero(), ObjectType::Boolean);
                            }
                        }
                        unreachable!();
                    }
                    if l_const == r_const {
                        return NodeEval::Const(FieldElement::one(), ObjectType::Boolean);
                    } else {
                        return NodeEval::Const(FieldElement::zero(), ObjectType::Boolean);
                    }
                }
            }
            BinaryOp::Ne => {
                if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    if l_bsize == 256 {
                        if let (NodeEval::Const(a, _), NodeEval::Const(b, _)) = (l_eval, r_eval) {
                            if a != b {
                                return NodeEval::Const(FieldElement::one(), ObjectType::Boolean);
                            } else {
                                return NodeEval::Const(FieldElement::zero(), ObjectType::Boolean);
                            }
                        }
                        unreachable!();
                    }
                    if l_const != r_const {
                        return NodeEval::Const(FieldElement::one(), ObjectType::Boolean);
                    } else {
                        return NodeEval::Const(FieldElement::zero(), ObjectType::Boolean);
                    }
                }
            }
            BinaryOp::And => {
                //Bitwise AND
                if l_is_zero {
                    return *l_eval;
                } else if r_is_zero {
                    return *r_eval;
                } else if self.lhs == self.rhs {
                    return *l_eval;
                } else if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    return NodeEval::Const(FieldElement::from(l_const & r_const), res_type);
                }
                //TODO if boolean and not zero, also checks this is correct for field elements
            }
            BinaryOp::Or => {
                //Bitwise OR
                if l_is_zero {
                    return *r_eval;
                } else if r_is_zero {
                    return *l_eval;
                } else if self.lhs == self.rhs {
                    return *r_eval;
                } else if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    return NodeEval::Const(FieldElement::from(l_const | r_const), res_type);
                }
                //TODO if boolean and not zero, also checks this is correct for field elements
            }
            BinaryOp::Xor => {
                if self.lhs == self.rhs {
                    return NodeEval::Const(FieldElement::zero(), res_type);
                }
                if l_is_zero {
                    return *r_eval;
                }
                if r_is_zero {
                    return *l_eval;
                }
                if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    return NodeEval::Const(FieldElement::from(l_const ^ r_const), res_type);
                }
                //TODO handle case when l_const is one (or r_const is one) by generating 'not rhs' instruction (or 'not lhs' instruction)
            }
            BinaryOp::Constrain(op) => {
                if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    match op {
                        ConstrainOp::Eq => assert_eq!(l_const, r_const),
                        ConstrainOp::Neq => assert_ne!(l_const, r_const),
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

    fn truncate_required(&self) -> (/*truncate_left*/bool, /*truncate_right*/bool) {
        match &self.operator {
            BinaryOp::Add => (false, false),
            BinaryOp::SafeAdd => (false, false),
            BinaryOp::Sub { .. } => (false, false),
            BinaryOp::SafeSub { .. } => (false, false),
            BinaryOp::Mul => (false, false),
            BinaryOp::SafeMul => (false, false),
            BinaryOp::Udiv => (true, true),
            BinaryOp::Sdiv => (true, true),
            BinaryOp::Urem => (true, true),
            BinaryOp::Srem => (true, true),
            BinaryOp::Div => (false, false),
            BinaryOp::Eq => (true, true),
            BinaryOp::Ne => (true, true),
            BinaryOp::Ugt => (true, true),
            BinaryOp::Uge => (true, true),
            BinaryOp::Ult => (true, true),
            BinaryOp::Ule => (true, true),
            BinaryOp::Sgt => (true, true),
            BinaryOp::Sge => (true, true),
            BinaryOp::Slt => (true, true),
            BinaryOp::Sle => (true, true),
            BinaryOp::Lt => (true, true),
            BinaryOp::Gt => (true, true),
            BinaryOp::Lte => (true, true),
            BinaryOp::Gte => (true, true),
            BinaryOp::And => (true, true),
            BinaryOp::Or => (true, true),
            BinaryOp::Xor => (true, true),
            BinaryOp::Constrain(..) => (true, true),
            BinaryOp::Assign => (false, false),
        }
    }
}

impl Operation {
    pub fn is_binary(&self) -> bool {
        matches!(self, Operation::Binary(..))
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

pub fn to_binary(op_kind: HirBinaryOpKind, op_type: ObjectType, lhs: NodeId, rhs: NodeId) -> Binary {
    let operator = match op_kind {
        HirBinaryOpKind::Add => BinaryOp::Add,
        HirBinaryOpKind::Subtract => BinaryOp::Sub { max_rhs_value: BigUint::from_u8(0).unwrap() },
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

pub fn get_witness_from_object(obj: &Object) -> Option<Witness> {
    match obj {
        Object::Integer(i) => Some(i.witness),
        Object::Array(_) => unreachable!("Array has multiple witnesses"),
        _ => obj.witness(),
    }
}
