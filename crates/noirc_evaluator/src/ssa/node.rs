use std::convert::TryInto;
use std::ops::Add;

use acvm::acir::native_types::Witness;
use acvm::FieldElement;
use arena;
use noirc_frontend::hir_def::expr::HirBinaryOpKind;
use noirc_frontend::node_interner::IdentId;
use num_bigint::BigUint;
use num_traits::One;

use crate::object::Object;
use num_traits::identities::Zero;
use std::ops::Mul;

use super::block::BlockId;
use super::code_gen::IRGenerator;

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
        NodeId(IRGenerator::dummy_id())
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
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ObjectType {
    //Numeric(NumericType),
    NativeField,
    // custom_field(BigUint), //TODO requires copy trait for BigUint
    Boolean,
    Unsigned(u32), //bit size
    Signed(u32),   //bit size
    //custom(u32),   //user-defined struct, u32 refers to the id of the type in...?todo
    //array(ObjectType),  TODO we should have primitive type and composite types
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

impl ObjectType {
    pub fn get_type_from_object(obj: &Object) -> ObjectType {
        match obj {
            Object::Arithmetic(_) => {
                todo!();
                //ObjectType::native_field
            }
            Object::Array(_) => {
                todo!();
                //ObjectType::none
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

    pub fn from_type(t: noirc_frontend::Type) -> ObjectType {
        match t {
            noirc_frontend::Type::FieldElement(_) => ObjectType::NativeField,

            noirc_frontend::Type::Integer(_ftype, sign, bit_size) => {
                assert!(
                    bit_size < super::integer::short_integer_max_bit_size(),
                    "long integers are not yet supported"
                );
                match sign {
                    noirc_frontend::Signedness::Signed => ObjectType::Signed(bit_size),
                    noirc_frontend::Signedness::Unsigned => ObjectType::Unsigned(bit_size),
                }
            }
            noirc_frontend::Type::Bool => ObjectType::Boolean,
            x => {
                let err = format!("currently we do not support type casting to {}", x);
                todo!("{}", err);
            }
        }
    }

    pub fn bits(&self) -> u32 {
        match self {
            ObjectType::Boolean => 1,
            ObjectType::NativeField => FieldElement::max_num_bits(),
            ObjectType::NotAnObject => 0,
            ObjectType::Signed(c) => *c,
            ObjectType::Unsigned(c) => *c,
        }
    }

    //maximum size of the representation (e.g. signed(8).max_size() return 255, not 128.)
    pub fn max_size(&self) -> BigUint {
        (BigUint::one() << self.bits()) - BigUint::one()
    }
}

#[derive(Clone, Debug)]
pub struct Instruction {
    pub id: NodeId,
    pub operator: Operation,
    pub rhs: NodeId,
    pub lhs: NodeId,
    pub res_type: ObjectType, //result type
    //prev,next: should have been a double linked list so that we can easily remove an instruction during optimisation phases
    pub parent_block: BlockId,
    pub is_deleted: bool,
    pub res_name: String,
    pub bit_size: u32, //TODO only for the truncate instruction...: bits size of the max value of the lhs.. a merger avec ci dessous!!!TODO
    pub max_value: BigUint, //TODO only for sub instruction: max value of the rhs

    //temp: todo phi subtype
    pub phi_arguments: Vec<(NodeId, BlockId)>,
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.res_name.is_empty() {
            write!(f, "{:?}", self.id.0.into_raw_parts().0)
        } else {
            write!(f, "{}", self.res_name.clone())
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NodeEval {
    Const(FieldElement, ObjectType),
    Instruction(NodeId),
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
            NodeEval::Instruction(i) => Some(i),
            NodeEval::Const(_, _) => None,
        }
    }
}

impl Instruction {
    pub fn new(
        op_code: Operation,
        lhs: NodeId,
        rhs: NodeId,
        r_type: ObjectType,
        parent_block: Option<BlockId>,
    ) -> Instruction {
        let id = NodeId::dummy();
        let p_block = parent_block.unwrap_or_else(BlockId::dummy);

        Instruction {
            id,
            operator: op_code,
            lhs,
            rhs,
            res_type: r_type,
            res_name: String::new(),
            is_deleted: false,
            parent_block: p_block,
            bit_size: 0,
            max_value: BigUint::zero(),
            phi_arguments: Vec::new(),
        }
    }

    //indicates whether the left and/or right operand of the instruction is required to be truncated to its bit-width
    pub fn truncate_required(&self, lhs_bits: u32, rhs_bits: u32) -> (bool, bool) {
        match self.operator {
            Operation::Add => (false, false),
            Operation::SafeAdd => (false, false),
            Operation::Sub => (false, false),
            Operation::SafeSub => (false, false),
            Operation::Mul => (false, false),
            Operation::SafeMul => (false, false),
            Operation::Udiv => (true, true),
            Operation::Sdiv => (true, true),
            Operation::Urem => (true, true),
            Operation::Srem => (true, true),
            Operation::Div => (false, false),
            Operation::Eq => (true, true),
            Operation::Ne => (true, true),
            Operation::Ugt => (true, true),
            Operation::Uge => (true, true),
            Operation::Ult => (true, true),
            Operation::Ule => (true, true),
            Operation::Sgt => (true, true),
            Operation::Sge => (true, true),
            Operation::Slt => (true, true),
            Operation::Sle => (true, true),
            Operation::Lt => (true, true),
            Operation::Gt => (true, true),
            Operation::Lte => (true, true),
            Operation::Gte => (true, true),
            Operation::And => (true, true),
            Operation::Not => (true, true),
            Operation::Or => (true, true),
            Operation::Xor => (true, true),
            Operation::Cast => {
                if self.res_type.bits() > lhs_bits {
                    return (true, false);
                }
                (false, false)
            }
            Operation::Ass => {
                assert!(lhs_bits == rhs_bits);
                (false, false)
            }
            Operation::Trunc | Operation::Phi => (false, false),
            Operation::Nop | Operation::Jne | Operation::Jeq | Operation::Jmp => (false, false),
            Operation::EqGate => (true, true),
        }
    }

    //Returns the field element as i128 and the bit size of the constant node
    pub fn get_const_value(c: FieldElement, ctype: ObjectType) -> (u128, u32) {
        match ctype {
            ObjectType::Boolean => (if c.is_zero() { 0 } else { 1 }, 1),
            ObjectType::NativeField => (0, 256),
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

    //Evaluate the instruction value when its operands are constant
    pub fn evaluate(&self, lhs: &NodeEval, rhs: &NodeEval) -> NodeEval {
        //let mut l_sign = false; //TODO
        let (l_is_zero, l_constant, l_bsize) = Instruction::node_evaluate(lhs);
        let (r_is_zero, r_constant, r_bsize) = Instruction::node_evaluate(rhs);
        let r_is_const = r_constant.is_some();
        let l_is_const = l_constant.is_some();

        match self.operator {
            Operation::Add | Operation::SafeAdd => {
                if r_is_zero {
                    return *lhs;
                } else if l_is_zero {
                    return *rhs;
                } else if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    //constant folding
                    if l_bsize == 256 {
                        //NO modulo for field elements - May be we should have a different opcode?
                        if let (NodeEval::Const(a, _), NodeEval::Const(b, _)) = (lhs, rhs) {
                            let res_value = a.add(*b);
                            return NodeEval::Const(res_value, self.res_type);
                        }
                        unreachable!();
                    }
                    assert!(l_bsize == r_bsize);
                    let res_value = (l_const + r_const) % l_bsize as u128;
                    return NodeEval::Const(FieldElement::from(res_value), self.res_type);
                }
                //if only one is const, we could try to do constant propagation but this will be handled by the arithmetization step anyways
                //so it is probably not worth it.
                //same for x+x vs 2*x
            }
            Operation::Sub | Operation::SafeSub => {
                if r_is_zero {
                    return *lhs;
                }
                if self.lhs == self.rhs {
                    return NodeEval::Const(FieldElement::zero(), self.res_type);
                }
                //constant folding
                if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    if l_bsize == 256 {
                        //NO modulo for field elements - May be we should have a different opcode?
                        if let (NodeEval::Const(a, _), NodeEval::Const(b, _)) = (lhs, rhs) {
                            let res_value = a.add(-*b);
                            return NodeEval::Const(res_value, self.res_type);
                        }
                        unreachable!();
                    }
                    //if l_constant.is_some() && r_constant.is_some() {
                    assert!(l_bsize == r_bsize);
                    let res_value = (l_const - r_const) % l_bsize as u128;
                    return NodeEval::Const(FieldElement::from(res_value), self.res_type);
                }
            }
            Operation::Mul | Operation::SafeMul => {
                if r_is_zero {
                    return *rhs;
                } else if l_is_zero {
                    return *lhs;
                } else if l_is_const && l_constant.unwrap() == 1 {
                    return *rhs;
                } else if r_is_const && r_constant.unwrap() == 1 {
                    return *lhs;
                } else if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    //constant folding
                    if l_bsize == 256 {
                        //NO modulo for field elements - May be we should have a different opcode?
                        if let (NodeEval::Const(a, _), NodeEval::Const(b, _)) = (lhs, rhs) {
                            let res_value = a.mul(*b);
                            return NodeEval::Const(res_value, self.res_type);
                        }
                        unreachable!();
                    }

                    assert!(l_bsize == r_bsize);
                    let res_value = (l_const * r_const) % l_bsize as u128;
                    return NodeEval::Const(FieldElement::from(res_value), self.res_type);
                }
                //if only one is const, we could try to do constant propagation but this will be handled by the arithmetization step anyways
                //so it is probably not worth it.
            }
            Operation::Udiv | Operation::Sdiv | Operation::Div => {
                if r_is_zero {
                    todo!("Panic - division by zero");
                } else if l_is_zero {
                    return *lhs; //TODO should we ensure rhs != 0 ???
                }
                //constant folding - TODO
                else if l_constant.is_some() && r_constant.is_some() {
                    todo!();
                } else if r_constant.is_some() {
                    //same as lhs*1/r
                    todo!("");
                    //return (Some(self.lhs), None, None);
                }
            }
            Operation::Urem | Operation::Srem => {
                if r_is_zero {
                    todo!("Panic - division by zero");
                } else if l_is_zero {
                    return *lhs; //TODO what is the correct result? and should we ensure rhs != 0 ???
                }
                //constant folding - TODO
                else if l_constant.is_some() && r_constant.is_some() {
                    todo!("divide l_constant/r_constant but take sign into account");
                }
            }
            Operation::Uge => {
                if r_is_zero {
                    return NodeEval::Const(FieldElement::zero(), ObjectType::Boolean);
                    //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this
                } else if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    assert!(l_bsize < 256); //comparisons are not implemented for field elements
                    let res = if l_const >= r_const {
                        FieldElement::one()
                    } else {
                        FieldElement::zero()
                    };
                    return NodeEval::Const(res, ObjectType::Boolean);
                }
            }
            Operation::Ult => {
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
            Operation::Ule => {
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
            Operation::Ugt => {
                if l_is_zero {
                    return NodeEval::Const(FieldElement::zero(), ObjectType::Boolean);
                // u<0 is false for unsigned u
                //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this
                } else if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    assert!(l_bsize < 256); //comparisons are not implemented for field elements
                    let res = if l_const > r_const {
                        FieldElement::one()
                    } else {
                        FieldElement::zero()
                    };
                    return NodeEval::Const(res, ObjectType::Boolean);
                }
            }
            Operation::Eq => {
                if self.lhs == self.rhs {
                    return NodeEval::Const(FieldElement::one(), ObjectType::Boolean);
                } else if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    if l_bsize == 256 {
                        if let (NodeEval::Const(a, _), NodeEval::Const(b, _)) = (lhs, rhs) {
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
            Operation::Ne => {
                if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    if l_bsize == 256 {
                        if let (NodeEval::Const(a, _), NodeEval::Const(b, _)) = (lhs, rhs) {
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
            Operation::And => {
                //Bitwise AND
                if l_is_zero {
                    return *lhs;
                } else if r_is_zero {
                    return *rhs;
                } else if self.lhs == self.rhs {
                    return *lhs;
                } else if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    return NodeEval::Const(FieldElement::from(l_const & r_const), self.res_type);
                }
                //TODO if boolean and not zero, also checks this is correct for field elements
            }
            Operation::Or => {
                //Bitwise OR
                if l_is_zero {
                    return *rhs;
                } else if r_is_zero {
                    return *lhs;
                } else if self.lhs == self.rhs {
                    return *rhs;
                } else if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    return NodeEval::Const(FieldElement::from(l_const | r_const), self.res_type);
                }
                //TODO if boolean and not zero, also checks this is correct for field elements
            }

            Operation::Not => {
                if let Some(l_const) = l_constant {
                    return NodeEval::Const(FieldElement::from(!l_const), self.res_type);
                }
            }
            Operation::Xor => {
                if self.lhs == self.rhs {
                    return NodeEval::Const(FieldElement::zero(), self.res_type);
                }
                if l_is_zero {
                    return *rhs;
                }
                if r_is_zero {
                    return *lhs;
                }
                if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    return NodeEval::Const(FieldElement::from(l_const ^ r_const), self.res_type);
                }
                //TODO handle case when l_const is one (or r_const is one) by generating 'not rhs' instruction (or 'not lhs' instruction)
            }
            Operation::Cast => {
                if l_constant.is_some() {
                    todo!("need to cast l_constant into self.res_type.bits() bit size")
                }
            }
            Operation::Phi => (), //Phi are simplified by simply_phi() later on; they must not be simplified here
            _ => (),
        }
        NodeEval::Instruction(self.id)
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
        match self.operator {
            //convert greater into less
            Operation::Ugt => {
                std::mem::swap(&mut self.rhs, &mut self.lhs);
                self.operator = Operation::Ult
            }
            Operation::Uge => {
                std::mem::swap(&mut self.rhs, &mut self.lhs);
                self.operator = Operation::Ule
            }
            Operation::Sgt => {
                std::mem::swap(&mut self.rhs, &mut self.lhs);
                self.operator = Operation::Slt
            }
            Operation::Sge => {
                std::mem::swap(&mut self.rhs, &mut self.lhs);
                self.operator = Operation::Sle
            }
            Operation::Gt => {
                std::mem::swap(&mut self.rhs, &mut self.lhs);
                self.operator = Operation::Lt
            }
            Operation::Gte => {
                std::mem::swap(&mut self.rhs, &mut self.lhs);
                self.operator = Operation::Lte
            }
            //TODO replace a<b with a<=b+1, but beware of edge cases!
            Operation::EqGate => {
                if self.rhs == self.lhs {
                    self.rhs = self.id;
                    self.is_deleted = true;
                    self.operator = Operation::Nop;
                }
            }
            _ => (),
        }
        if is_commutative(self.operator) && self.rhs < self.lhs {
            std::mem::swap(&mut self.rhs, &mut self.lhs);
        }
    }
}

//adapted from LLVM IR
#[allow(dead_code)] //Some enums are not used yet, allow dead_code should be removed once they are all implemented.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Operation {
    Add,     //(+)
    SafeAdd, //(+) safe addition
    Sub,     //(-)
    SafeSub, //(-) safe subtraction
    Mul,     //(*)
    SafeMul, //(*) safe multiplication
    Udiv,    //(/) unsigned division
    Sdiv,    //(/) signed division
    Urem,    //(%) modulo; remainder of unsigned division
    Srem,    //(%) remainder of signed division
    Div,     //(/) field division
    Eq,      //(==) equal
    Ne,      //(!=) not equal
    Ugt,     //(>) unsigned greater than
    Uge,     //(>=) unsigned greater or equal
    Ult,     //(<) unsigned less than
    Ule,     //(<=) unsigned less or equal
    Sgt,     //(>) signed greater than
    Sge,     //(>=) signed greater or equal
    Slt,     //(<) signed less than
    Sle,     //(<=) signed less or equal
    Lt,      //(<) field less
    Gt,      //(>) field greater
    Lte,     //(<=) field less or equal
    Gte,     //(<=) field greater or equal
    And,     //(&) Bitwise And
    Not,     //(!) Bitwise Not
    Or,      //(|) Bitwise Or
    Xor,     //(^) Bitwise Xor
    Cast,    //convert type
    Ass,     //assignement
    Trunc,   //truncate

    //control flow
    Jne, //jump on not equal
    Jeq, //jump on equal
    Jmp, //unconditional jump
    Phi,
    Nop,    // no op
    EqGate, //write a gate enforcing equality of the two sides (to support the constrain statement)
}

pub fn is_commutative(op_code: Operation) -> bool {
    matches!(
        op_code,
        Operation::Add
            | Operation::SafeAdd
            | Operation::Mul
            | Operation::SafeMul
            | Operation::And
            | Operation::Or
            | Operation::Xor
    )
}

pub fn is_binary(op_code: Operation) -> bool {
    matches!(
        op_code,
        Operation::Add
            | Operation::SafeAdd
            | Operation::Sub
            | Operation::SafeSub
            | Operation::Mul
            | Operation::SafeMul
            | Operation::Udiv
            | Operation::Sdiv
            | Operation::Urem
            | Operation::Srem
            | Operation::Div
            | Operation::Eq
            | Operation::Ne
            | Operation::Ugt
            | Operation::Uge
            | Operation::Ult
            | Operation::Ule
            | Operation::Sgt
            | Operation::Sge
            | Operation::Slt
            | Operation::Sle
            | Operation::Lt
            | Operation::Gt
            | Operation::Lte
            | Operation::Gte
            | Operation::And
            | Operation::Or
            | Operation::Xor
            | Operation::Trunc
            | Operation::EqGate
    )

    //For the record:  Operation::not | Operation::cast => false | Operation::ass | Operation::trunc
    //  | Operation::jne | Operation::jeq | Operation::jmp | Operation::phi | Operation::nop
}

pub fn to_operation(op_kind: HirBinaryOpKind, op_type: ObjectType) -> Operation {
    match op_kind {
        HirBinaryOpKind::Add => Operation::Add,
        HirBinaryOpKind::Subtract => Operation::Sub,
        HirBinaryOpKind::Multiply => Operation::Mul,
        HirBinaryOpKind::Equal => Operation::Eq,
        HirBinaryOpKind::NotEqual => Operation::Ne,
        HirBinaryOpKind::And => Operation::And,
        HirBinaryOpKind::Or => Operation::Or,
        HirBinaryOpKind::Xor => Operation::Xor,
        HirBinaryOpKind::Divide => {
            let num_type: NumericType = op_type.into();
            match num_type {
                NumericType::Signed(_) => Operation::Sdiv,
                NumericType::Unsigned(_) => Operation::Udiv,
                NumericType::NativeField => Operation::Div,
            }
        }
        HirBinaryOpKind::Less => {
            let num_type: NumericType = op_type.into();
            match num_type {
                NumericType::Signed(_) => Operation::Slt,
                NumericType::Unsigned(_) => Operation::Ult,
                NumericType::NativeField => Operation::Lt,
            }
        }
        HirBinaryOpKind::Greater => {
            let num_type: NumericType = op_type.into();
            match num_type {
                NumericType::Signed(_) => Operation::Sgt,
                NumericType::Unsigned(_) => Operation::Ugt,
                NumericType::NativeField => Operation::Gt,
            }
        }
        HirBinaryOpKind::LessEqual => {
            let num_type: NumericType = op_type.into();
            match num_type {
                NumericType::Signed(_) => Operation::Sle,
                NumericType::Unsigned(_) => Operation::Ult,
                NumericType::NativeField => Operation::Lte,
            }
        }
        HirBinaryOpKind::GreaterEqual => {
            let num_type: NumericType = op_type.into();
            match num_type {
                NumericType::Signed(_) => Operation::Sge,
                NumericType::Unsigned(_) => Operation::Uge,
                NumericType::NativeField => Operation::Gte,
            }
        }
        HirBinaryOpKind::Assign => Operation::Ass,
        HirBinaryOpKind::MemberAccess => todo!(),
    }
}

pub fn get_witness_from_object(obj: &Object) -> Option<Witness> {
    match obj {
        Object::Integer(i) => Some(i.witness),
        Object::Array(_) => unreachable!("Array has multiple witnesses"),
        _ => obj.witness(),
    }
}
