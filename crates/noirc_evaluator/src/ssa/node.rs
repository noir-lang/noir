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

pub trait Node: std::fmt::Display {
    fn get_type(&self) -> ObjectType;
    //fn get_bit_size(&self) -> u32;
    fn get_id(&self) -> arena::Index;
    fn bits(&self) -> u32; //bit size of the node
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

    fn bits(&self) -> u32 {
        self.get_type().bits()
    }

    fn get_id(&self) -> arena::Index {
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

    fn bits(&self) -> u32 {
        match self {
            NodeObj::Obj(o) => o.bits(),
            NodeObj::Instr(i) => i.res_type.bits(),
            NodeObj::Const(c) => c.bits(),
        }
    }

    fn get_id(&self) -> arena::Index {
        match self {
            NodeObj::Obj(o) => o.get_id(),
            NodeObj::Instr(i) => i.idx,
            NodeObj::Const(c) => c.get_id(),
        }
    }
}

impl Node for Constant {
    fn get_type(&self) -> ObjectType {
        self.value_type
    }

    fn bits(&self) -> u32 {
        self.value.bits().try_into().unwrap()
    }

    fn get_id(&self) -> arena::Index {
        self.id
    }
}

#[derive(Debug)]
pub enum NodeObj {
    Obj(Variable),
    Instr(Instruction),
    Const(Constant),
}

#[derive(Debug)]
pub struct Constant {
    pub id: arena::Index,
    pub value: BigUint,    //TODO use FieldElement instead
    pub value_str: String, //TODO ConstStr subtype
    pub value_type: ObjectType,
}

#[derive(Debug)]
pub struct Variable {
    pub id: arena::Index,
    pub obj_type: ObjectType,
    pub name: String,
    //pub cur_value: arena::Index, //for generating the SSA form, current value of the object during parsing of the AST
    pub root: Option<arena::Index>, //when generating SSA, assignment of an object creates a new one which is linked to the original one
    pub def: Option<IdentId>,       //TODO redondant with root - should it be an option?
    //TODO clarify where cur_value and root is stored, and also this:
    //  pub max_bits: u32,                  //max possible bit size of the expression
    //  pub max_value: Option<BigUInt>,     //maximum possible value of the expression, if less than max_bits
    pub witness: Option<Witness>,
    pub parent_block: arena::Index,
}

impl Variable {
    pub fn get_root(&self) -> arena::Index {
        match self.root {
            Some(r) => r,
            _ => self.id,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ObjectType {
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

impl ObjectType {
    fn is_signed(&self) -> bool {
        matches!(self, ObjectType::Signed(_))
    }

    fn is_unsigned(&self) -> bool {
        matches!(self, ObjectType::Unsigned(_))
    }

    fn is_field(&self) -> bool {
        matches!(
            self,
            ObjectType::NativeField //| ObjectType::custom_field
        )
    }

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
            Object::Integer(i) => ObjectType::Unsigned(i.num_bits), //TODO signed or unsigned?
            Object::Linear(_) => {
                ObjectType::NativeField //TODO check with Kev!
            }
            Object::Null => ObjectType::NotAnObject,
        }
    }

    pub fn from_type(t: noirc_frontend::Type) -> ObjectType {
        match t {
            noirc_frontend::Type::FieldElement(_) => ObjectType::NativeField,

            noirc_frontend::Type::Integer(_ftype, sign, bit_size) => match sign {
                noirc_frontend::Signedness::Signed => ObjectType::Signed(bit_size),
                noirc_frontend::Signedness::Unsigned => ObjectType::Unsigned(bit_size),
            },
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
    pub idx: arena::Index,
    pub operator: Operation,
    pub rhs: arena::Index,
    pub lhs: arena::Index,
    pub res_type: ObjectType, //result type
    //prev,next: should have been a double linked list so that we can easily remove an instruction during optimisation phases
    pub parent_block: arena::Index,
    pub is_deleted: bool,
    pub res_name: String,
    pub bit_size: u32, //TODO only for the truncate instruction...: bits size of the max value of the lhs.. a merger avec ci dessous!!!TODO
    pub max_value: BigUint, //TODO only for sub instruction: max value of the rhs

    //temp: todo phi subtype
    pub phi_arguments: Vec<(arena::Index, arena::Index)>,
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.res_name.is_empty() {
            write!(f, "{:?}", self.idx.into_raw_parts().0)
        } else {
            write!(f, "{}", self.res_name.clone())
        }
    }
}

// pub fn print_i(&self) -> String {
//     if self.res_name.is_empty() {
//         format!("{:?}", self.idx.into_raw_parts().0)
//     } else {
//         self.res_name.clone()
//     }
// }

#[derive(Debug, Clone, Copy)]
pub enum NodeEval {
    Const(FieldElement, ObjectType),
    Idx(arena::Index),
}

impl NodeEval {
    pub fn to_const_value(self) -> Option<FieldElement> {
        match self {
            NodeEval::Const(c, _) => Some(c),
            _ => None,
        }
    }
    pub fn to_index(self) -> Option<arena::Index> {
        match self {
            NodeEval::Idx(i) => Some(i),
            NodeEval::Const(_, _) => None,
        }
    }
}

impl Instruction {
    pub fn new(
        op_code: Operation,
        lhs: arena::Index,
        rhs: arena::Index,
        r_type: ObjectType,
        parent_block: Option<arena::Index>,
    ) -> Instruction {
        let id0 = crate::ssa::code_gen::IRGenerator::dummy_id();
        let p_block = parent_block.unwrap_or(id0);
        Instruction {
            idx: id0,
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
            Operation::add => (false, false),
            Operation::safe_add => (false, false),
            Operation::sub => (false, false),
            Operation::safe_sub => (false, false),
            Operation::mul => (false, false),
            Operation::safe_mul => (false, false),
            Operation::udiv => (true, true),
            Operation::sdiv => (true, true),
            Operation::urem => (true, true),
            Operation::srem => (true, true),
            Operation::div => (false, false),
            Operation::eq => (true, true),
            Operation::ne => (true, true),
            Operation::ugt => (true, true),
            Operation::uge => (true, true),
            Operation::ult => (true, true),
            Operation::ule => (true, true),
            Operation::sgt => (true, true),
            Operation::sge => (true, true),
            Operation::slt => (true, true),
            Operation::sle => (true, true),
            Operation::lt => (true, true),
            Operation::gt => (true, true),
            Operation::lte => (true, true),
            Operation::gte => (true, true),
            Operation::and => (true, true),
            Operation::not => (true, true),
            Operation::or => (true, true),
            Operation::xor => (true, true),
            Operation::cast => {
                if self.res_type.bits() > lhs_bits {
                    return (true, false);
                }
                (false, false)
            }
            Operation::ass => {
                assert!(lhs_bits == rhs_bits);
                (false, false)
            }
            Operation::trunc | Operation::phi => (false, false),
            Operation::nop | Operation::jne | Operation::jeq | Operation::jmp => (false, false),
            Operation::eq_gate => (true, true),
        }
    }

    //Returns the field element as i128 and the bit size of the constant node
    pub fn get_const_value(c: FieldElement, ctype: ObjectType) -> (u128, u32) {
        match ctype {
            ObjectType::Boolean => (if c.is_zero() { 0 } else { 1 }, 1),
            ObjectType::NativeField => (0, 256),
            ObjectType::Signed(b) | ObjectType::Unsigned(b) => (c.to_u128(), b), //TODO check how to handle signed integers
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
            Operation::add | Operation::safe_add => {
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
                    return NodeEval::Const(FieldElement::from(res_value as i128), self.res_type);
                }
                //if only one is const, we could try to do constant propagation but this will be handled by the arithmetization step anyways
                //so it is probably not worth it.
                //same for x+x vs 2*x
            }
            Operation::sub | Operation::safe_sub => {
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
                    return NodeEval::Const(FieldElement::from(res_value as i128), self.res_type);
                }
            }
            Operation::mul | Operation::safe_mul => {
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
                    return NodeEval::Const(FieldElement::from(res_value as i128), self.res_type);
                }
                //if only one is const, we could try to do constant propagation but this will be handled by the arithmetization step anyways
                //so it is probably not worth it.
            }
            Operation::udiv | Operation::sdiv | Operation::div => {
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
            Operation::urem | Operation::srem => {
                if r_is_zero {
                    todo!("Panic - division by zero");
                } else if l_is_zero {
                    return *lhs; //TODO what is the correct result? and should we ensure rhs != 0 ???
                }
                //constant folding - TODO
                else if l_constant.is_some() && r_constant.is_some() {
                    todo!();
                }
            }
            Operation::uge => {
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
            Operation::ult => {
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
            Operation::ule => {
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
            Operation::ugt => {
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
            Operation::eq => {
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
            Operation::ne => {
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
            Operation::and => {
                //Bitwise AND
                if l_is_zero {
                    return *lhs;
                } else if r_is_zero {
                    return *rhs;
                } else if self.lhs == self.rhs {
                    return *lhs;
                } else if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    return NodeEval::Const(
                        FieldElement::from((l_const & r_const) as i128),
                        self.res_type,
                    );
                }
                //TODO if boolean and not zero, also checks this is correct for field elements
                //TODO use from u128
            }
            Operation::or => {
                //Bitwise OR
                if l_is_zero {
                    return *rhs;
                } else if r_is_zero {
                    return *lhs;
                } else if self.lhs == self.rhs {
                    return *rhs;
                } else if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    return NodeEval::Const(
                        FieldElement::from((l_const | r_const) as i128),
                        self.res_type,
                    );
                }
                //TODO if boolean and not zero, also checks this is correct for field elements
                //TODO use from u128
            }

            Operation::not => {
                //todo..
                // if l_is_zero {
                //     return NodeEval::Const(FieldElement::one(), ObjectType::Boolean);
                // } else if l_is_const {
                //     return NodeEval::Const(FieldElement::zero(), ObjectType::Boolean);
                // }
                if let Some(l_const) = l_constant {
                    return NodeEval::Const(FieldElement::from((!l_const) as i128), self.res_type);
                }
            }
            Operation::xor => {
                //todo
                // if self.lhs == self.rhs {
                //     return NodeEval::Const(FieldElement::zero(), ObjectType::Boolean);
                // }
                // if l_is_zero {
                //     return *rhs;
                // }
                // if r_is_zero {
                //     return *lhs;
                // } else if l_is_const && r_is_const {
                //     return NodeEval::Const(FieldElement::zero(), ObjectType::Boolean);
                // } else if l_is_const {
                //     todo!("generate 'not rhs' instruction");
                // } else if r_is_const {
                //     todo!("generate 'not lhs' instruction");
                // }
                if let (Some(l_const), Some(r_const)) = (l_constant, r_constant) {
                    return NodeEval::Const(
                        FieldElement::from((l_const ^ r_const) as i128),
                        self.res_type,
                    );
                }
                //TODO use from u128
            }
            Operation::phi => (), //Phi are simplified by simply_phi()
            Operation::cast => {
                if l_constant.is_some() {
                    todo!();
                }
            } //
            _ => (),
        }
        NodeEval::Idx(self.idx)
    }

    // Simplifies trivial Phi instructions by returning:
    // None, if the instruction is unreachable or in the root block and can be safely deleted
    // Some(id), if the instruction can be replaced by the node id
    // Some(ins_id), if the instruction is not trivial
    pub fn simplify_phi(
        ins_id: arena::Index,
        phi_arguments: &[(arena::Index, arena::Index)],
    ) -> Option<arena::Index> {
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
            Operation::ugt => {
                std::mem::swap(&mut self.rhs, &mut self.lhs);
                self.operator = Operation::ult
            }
            Operation::uge => {
                std::mem::swap(&mut self.rhs, &mut self.lhs);
                self.operator = Operation::ule
            }
            Operation::sgt => {
                std::mem::swap(&mut self.rhs, &mut self.lhs);
                self.operator = Operation::slt
            }
            Operation::sge => {
                std::mem::swap(&mut self.rhs, &mut self.lhs);
                self.operator = Operation::sle
            }
            Operation::gt => {
                std::mem::swap(&mut self.rhs, &mut self.lhs);
                self.operator = Operation::lt
            }
            Operation::gte => {
                std::mem::swap(&mut self.rhs, &mut self.lhs);
                self.operator = Operation::lte
            }
            //TODO replace a<b with a<=b+1, but beware of edge cases!
            Operation::eq_gate => {
                if self.rhs == self.lhs {
                    self.rhs = self.idx;
                    self.is_deleted = true;
                    self.operator = Operation::nop;
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
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Operation {
    add,      //(+)
    safe_add, //(+) safe addition
    sub,      //(-)
    safe_sub, //(-) safe subtraction
    mul,      //(*)
    safe_mul, //(*) safe multiplication
    udiv,     //(/) unsigned division
    sdiv,     //(/) signed division
    urem,     //(%) modulo; remainder of unsigned division
    srem,     //(%) remainder of signed division
    div,      //(/) field division
    eq,       //(==) equal
    ne,       //(!=) not equal
    ugt,      //(>) unsigned greater than
    uge,      //(>=) unsigned greater or equal
    ult,      //(<) unsigned less than
    ule,      //(<=) unsigned less or equal
    sgt,      //(>) signed greater than
    sge,      //(>=) signed greater or equal
    slt,      //(<) signed less than
    sle,      //(<=) signed less or equal
    lt,       //(<) field less
    gt,       //(>) field greater
    lte,      //(<=) field less or equal
    gte,      //(<=) field greater or equal
    and,
    not,
    or,
    xor,
    cast,  //convert type
    ass,   //assignement
    trunc, //truncate

    //control flow
    jne, //jump on not equal
    jeq, //jump on equal
    jmp, //unconditional jump
    phi,
    nop,     // no op
    eq_gate, //write a gate enforcing equality of the two sides (to support the constrain statement)
}

pub fn is_commutative(op_code: Operation) -> bool {
    matches!(
        op_code,
        Operation::add
            | Operation::safe_add
            | Operation::mul
            | Operation::safe_mul
            | Operation::and
            | Operation::or
            | Operation::xor
    )
}

pub fn is_binary(op_code: Operation) -> bool {
    matches!(
        op_code,
        Operation::add
            | Operation::safe_add
            | Operation::sub
            | Operation::safe_sub
            | Operation::mul
            | Operation::safe_mul
            | Operation::udiv
            | Operation::sdiv
            | Operation::urem
            | Operation::srem
            | Operation::div
            | Operation::eq
            | Operation::ne
            | Operation::ugt
            | Operation::uge
            | Operation::ult
            | Operation::ule
            | Operation::sgt
            | Operation::sge
            | Operation::slt
            | Operation::sle
            | Operation::lt
            | Operation::gt
            | Operation::lte
            | Operation::gte
            | Operation::and
            | Operation::or
            | Operation::xor
            | Operation::trunc
            | Operation::eq_gate
    )

    //For the record:  Operation::not | Operation::cast => false | Operation::ass | Operation::trunc
    //  | Operation::jne | Operation::jeq | Operation::jmp | Operation::phi | Operation::nop
}

pub fn to_operation(op_kind: HirBinaryOpKind, op_type: ObjectType) -> Operation {
    match op_kind {
        HirBinaryOpKind::Add => Operation::add,
        HirBinaryOpKind::Subtract => Operation::sub,
        HirBinaryOpKind::Multiply => Operation::mul,
        HirBinaryOpKind::Equal => Operation::eq,
        HirBinaryOpKind::NotEqual => Operation::ne,
        HirBinaryOpKind::And => Operation::and,
        HirBinaryOpKind::Or => Operation::or,
        HirBinaryOpKind::Xor => Operation::xor,
        HirBinaryOpKind::Divide => {
            if op_type.is_signed() {
                return Operation::sdiv;
            }
            if op_type.is_unsigned() {
                return Operation::udiv;
            }
            if op_type.is_field() {
                return Operation::div;
            }
            unreachable!("invalid type"); //TODO error
        }
        HirBinaryOpKind::Less => {
            if op_type.is_signed() {
                return Operation::slt;
            }
            if op_type.is_unsigned() {
                return Operation::ult;
            }
            if op_type.is_field() {
                return Operation::lt;
            }
            unreachable!("invalid type"); //TODO error
        }
        HirBinaryOpKind::Greater => {
            if op_type.is_signed() {
                return Operation::sgt;
            }
            if op_type.is_unsigned() {
                return Operation::ugt;
            }
            if op_type.is_field() {
                return Operation::gt;
            }
            unreachable!("invalid type"); //TODO error
        }
        HirBinaryOpKind::LessEqual => {
            if op_type.is_signed() {
                return Operation::sle;
            }
            if op_type.is_unsigned() {
                return Operation::ule;
            }
            if op_type.is_field() {
                return Operation::lte;
            }
            unreachable!("invalid type"); //TODO error
        }
        HirBinaryOpKind::GreaterEqual => {
            if op_type.is_signed() {
                return Operation::sge;
            }
            if op_type.is_unsigned() {
                return Operation::uge;
            }
            if op_type.is_field() {
                return Operation::gte;
            }
            unreachable!("invalid type"); //TODO error
        }
        HirBinaryOpKind::Assign => Operation::ass,
        HirBinaryOpKind::MemberAccess => todo!(),
    }
}

pub fn get_witness_from_object(obj: &Object) -> Option<Witness> {
    match obj {
        Object::Integer(i) => Some(i.witness),
        Object::Array(_) => unreachable!("Array has multiple witnesses"),
        Object::Constants(_) => None,
        _ => obj.witness(), //("This function should only be called for Integer objects"),
    }
}
