use std::convert::TryInto;

use acvm::acir::native_types::Witness;
use acvm::FieldElement;
use arena;
use noirc_frontend::hir_def::expr::HirBinaryOpKind;
use noirc_frontend::node_interner::IdentId;
use num_bigint::BigUint;

use crate::object::Object;
use num_traits::identities::Zero;

pub trait Node {
    fn get_type(&self) -> ObjectType;
    fn print(&self) -> String;
    //fn get_bit_size(&self) -> u32;
    fn get_id(&self) -> arena::Index;
    fn bits(&self) -> u32; //bit size of the node
}

impl Node for Variable {
    fn get_type(&self) -> ObjectType {
        self.obj_type
    }

    fn print(&self) -> String {
        //format!("{}({:?})", self.name, self.id)
        self.name.to_string()
    }

    fn bits(&self) -> u32 {
        self.get_type().bits()
    }

    fn get_id(&self) -> arena::Index {
        self.id
    }
}

impl NodeObj {
    // pub fn new_constant_bool(value: bool) -> NodeObj {
    //     let val = if value { 1_u32 } else { 0_u32 };
    //     NodeObj::Const(Constant {
    //         id: crate::ssa::code_gen::IRGenerator::dummy_id(),
    //         value: BigUint::from(val),
    //         value_str: String::new(),
    //         value_type: ObjectType::boolean,
    //     })
    // }

    // pub fn new_constant_int(value: u32, bit_size: u32, is_signed: bool) -> NodeObj {
    //     NodeObj::Const(Constant {
    //         id: crate::ssa::code_gen::IRGenerator::dummy_id(),
    //         value: BigUint::from(value),
    //         value_str: String::new(),
    //         value_type: if is_signed {
    //             ObjectType::signed(bit_size)
    //         } else {
    //             ObjectType::unsigned(bit_size)
    //         },
    //     })
    // }
}

impl Node for NodeObj {
    fn get_type(&self) -> ObjectType {
        match self {
            NodeObj::Obj(o) => o.get_type(),
            NodeObj::Instr(i) => i.res_type,
            NodeObj::Const(o) => o.value_type,
        }
    }

    fn print(&self) -> String {
        match self {
            NodeObj::Obj(o) => o.print(),
            NodeObj::Instr(i) => i.print_i(),
            NodeObj::Const(c) => c.print(),
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

    fn print(&self) -> String {
        self.value.to_string()
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

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub enum ObjectType {
    native_field,
    // custom_field(BigUint), //TODO requires copy trait for BigUint
    boolean,
    unsigned(u32), //bit size
    signed(u32),   //bit size
    //custom(u32),   //user-defined struct, u32 refers to the id of the type in...?todo
    //array(ObjectType),  TODO we should have primitive type and composite types
    //TODO big_int
    //TODO floats
    none, //not an object
}

impl ObjectType {
    fn is_signed(&self) -> bool {
        matches!(self, ObjectType::signed(_))
    }

    fn is_unsigned(&self) -> bool {
        matches!(self, ObjectType::unsigned(_))
    }

    fn is_field(&self) -> bool {
        matches!(
            self,
            ObjectType::native_field //| ObjectType::custom_field
        )
    }

    // pub fn from_type(t: noirc_frontend::Type) -> ObjectType {
    //     match t {
    //         noirc_frontend::Type::FieldElement(_) => ObjectType::native_field,
    //         noirc_frontend::Type::Array(_,_,_) => ObjectType::none, //TODO
    //         noirc_frontend::Type::Integer(_ftype, sign, bit_size) => {
    //             match sign {
    //                 //todo FieldElementType?
    //                 noirc_frontend::Signedness::Signed => ObjectType::signed(bit_size),
    //                 noirc_frontend::Signedness::Unsigned => ObjectType::unsigned(bit_size),
    //             }
    //         }
    //         noirc_frontend::Type::Bool => ObjectType::boolean,
    //         _ => ObjectType::none, //todo Error,Unspecified, Unknown,Unit
    //     }
    // }

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
            Object::Constants(_) => ObjectType::native_field, //TODO
            Object::Integer(i) => ObjectType::unsigned(i.num_bits), //TODO signed or unsigned?
            Object::Linear(_) => {
                todo!();
                //ObjectType::native_field
            }
            Object::Null => ObjectType::none,
        }
    }

    pub fn bits(&self) -> u32 {
        match self {
            ObjectType::boolean => 1,
            ObjectType::native_field => FieldElement::max_num_bits(), //TODO is it the correct value?
            ObjectType::none => 0,
            ObjectType::signed(c) => *c,
            ObjectType::unsigned(c) => *c,
            //ObjectType::custom(_) => todo!(),
        }
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

#[derive(Debug, Clone, Copy)]
pub enum NodeEval {
    Const(FieldElement, ObjectType),
    Idx(arena::Index),
}

impl NodeEval {
    pub fn to_const_value(&self) -> Option<FieldElement> {
        match self {
            NodeEval::Const(c, _) => Some(*c),
            _ => None,
        }
    }
    pub fn to_index(&self) -> Option<arena::Index> {
        match self {
            NodeEval::Idx(i) => Some(*i),
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

    pub fn print_i(&self) -> String {
        if self.res_name.is_empty() {
            format!("{:?}", self.idx)
        } else {
            self.res_name.clone()
        }
    }

    //indicates if the operation is a substraction
    pub fn is_sub(&self) -> bool {
        matches!(self.operator, Operation::sub | Operation::safe_sub)
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
            Operation::fmod => (true, true),   //to check
            Operation::fneg => (false, false), //to check
            Operation::fdiv => (true, true),   //to check
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
                if lhs_bits != rhs_bits {
                    return (false, true);
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

    pub fn get_max_value(&self, lhs_max: BigUint, rhs_max: BigUint) -> BigUint {
        match self.operator {
            Operation::add => lhs_max + rhs_max,
            Operation::safe_add => todo!(),
            Operation::sub => lhs_max + rhs_max,
            Operation::safe_sub => todo!(),
            Operation::mul => lhs_max * rhs_max,
            Operation::safe_mul => todo!(),
            Operation::udiv => lhs_max,
            Operation::sdiv => todo!(),
            Operation::urem => rhs_max - BigUint::from(1_u32),
            Operation::srem => todo!(),
            Operation::fmod => todo!(),
            Operation::fneg => todo!(),
            Operation::fdiv => todo!(),
            Operation::div => todo!(),
            Operation::eq => BigUint::from(1_u32),
            Operation::ne => BigUint::from(1_u32),
            Operation::ugt => BigUint::from(1_u32),
            Operation::uge => BigUint::from(1_u32),
            Operation::ult => BigUint::from(1_u32),
            Operation::ule => BigUint::from(1_u32),
            Operation::sgt => BigUint::from(1_u32),
            Operation::sge => BigUint::from(1_u32),
            Operation::slt => BigUint::from(1_u32),
            Operation::sle => BigUint::from(1_u32),
            Operation::lt => BigUint::from(1_u32),
            Operation::gt => BigUint::from(1_u32),
            Operation::lte => BigUint::from(1_u32),
            Operation::gte => BigUint::from(1_u32),
            Operation::and => BigUint::from(1_u32),
            Operation::not => BigUint::from(1_u32),
            Operation::or => BigUint::from(1_u32),
            Operation::xor => BigUint::from(1_u32),
            //'a cast b' means we cast b into a: (a) b
            //we assume that eventual truncate has been done so rhs_max must fit into type of a.
            Operation::cast => rhs_max,
            Operation::trunc => BigUint::min(
                lhs_max,
                BigUint::from(2_u32).pow(rhs_max.try_into().unwrap()) - BigUint::from(1_u32),
            ),
            //'a = b': a and b must be of same type.
            Operation::ass => rhs_max,
            Operation::nop | Operation::jne | Operation::jeq | Operation::jmp => todo!(),
            Operation::phi => BigUint::max(lhs_max, rhs_max), //TODO operands are in phi_arguments, not lhs/rhs!!
            Operation::eq_gate => BigUint::min(lhs_max, rhs_max),
        }
    }

    // pub fn get_const_value(c: &Constant) -> (u32, u32) {
    //     //....todo...only u32 for now... should also provide the sign
    //     match c.value_type {
    //         ObjectType::boolean => (if c.value.is_zero() { 0 } else { 1 }, 1),
    //         ObjectType::native_field => todo!(), //to field(value), field:prime
    //         ObjectType::signed(b) | ObjectType::unsigned(b) => {
    //             (c.value.clone().try_into().unwrap(), b)
    //         }
    //         _ => todo!(),
    //     }
    // }

    pub fn get_const_value2(c: FieldElement, ctype: ObjectType) -> (u32, u32) {
        //....todo...only u32 for now... should also provide the sign
        match ctype {
            ObjectType::boolean => (if c.is_zero() { 0 } else { 1 }, 1),
            ObjectType::native_field => (c.to_u128().try_into().unwrap(), 32), //TODO! //to field(value), field:prime
            ObjectType::signed(b) | ObjectType::unsigned(b) => (c.to_u128().try_into().unwrap(), b),
            _ => todo!(),
        }
    }

    // pub fn to_const(value: FieldElement, obj_type: ObjectType) -> NodeObj {
    //     let c = Constant {
    //         id: crate::ssa::code_gen::IRGenerator::dummy_id(),
    //         value: BigUint::from_bytes_be(&value.to_bytes()),
    //         value_str: String::new(),
    //         value_type: obj_type,
    //     };
    //     NodeObj::Const(c)
    // }

    pub fn node_evaluate(
        n: &NodeEval,
        is_zero: &mut bool,
        const_value: &mut Option<u32>,
        bit_size: &mut u32,
    ) {
        match n {
            &NodeEval::Const(c, t) => {
                *is_zero = c.is_zero();
                let cv = Instruction::get_const_value2(c, t);
                *const_value = Some(cv.0);
                *bit_size = cv.1;
            }
            _ => {
                *const_value = None;
            }
        }
    }

    //Evalaute the instruction value when its operands are constant
    pub fn evaluate(&self, lhs: &NodeEval, rhs: &NodeEval) -> NodeEval {
        let mut r_is_zero = false;
        let mut l_is_zero = false;
        let mut l_constant = None;
        let mut r_constant = None;
        let mut r_bsize = 0;
        let mut l_bsize = 0;
        //let mut l_sign = false; //TODO
        Instruction::node_evaluate(lhs, &mut l_is_zero, &mut l_constant, &mut l_bsize);
        Instruction::node_evaluate(rhs, &mut r_is_zero, &mut r_constant, &mut r_bsize);

        match self.operator {
            Operation::add | Operation::safe_add => {
                if r_is_zero {
                    return *lhs;
                } else if l_is_zero {
                    return *rhs;
                } else if l_constant.is_some() && r_constant.is_some() {
                    //constant folding - TODO - only for integers; NO modulo for field elements - May be we should have a different opcode for field addition?
                    assert!(l_bsize == r_bsize);
                    let res_value = (l_constant.unwrap() + r_constant.unwrap()) % l_bsize;
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
                //constant folding - TODO - only for integers; NO modulo for field elements - May be we should have a different opcode?
                if l_constant.is_some() && r_constant.is_some() {
                    assert!(l_bsize == r_bsize);
                    let res_value = (l_constant.unwrap() - r_constant.unwrap()) % l_bsize;
                    return NodeEval::Const(FieldElement::from(res_value as i128), self.res_type);
                }
            }
            Operation::mul | Operation::safe_mul => {
                if r_is_zero {
                    return *rhs;
                } else if l_is_zero {
                    return *lhs;
                } else if l_constant.is_some() && l_constant.unwrap() == 1 {
                    return *rhs;
                } else if r_constant.is_some() && r_constant.unwrap() == 1 {
                    return *lhs;
                } else if l_constant.is_some() && r_constant.is_some() {
                    //constant folding - TODO - only for integers; NO modulo for field elements - May be we should have a different opcode?
                    assert!(l_bsize == r_bsize);
                    let res_value = (l_constant.unwrap() * r_constant.unwrap()) % l_bsize;
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
                //else if r_constant.is_some() {
                //TODO same as lhs*1/r
                //return (Some(self.lhs), None, None);
                //}
                //constant folding - TODO
                else if l_constant.is_some() && r_constant.is_some() {
                    todo!();
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
                    return NodeEval::Const(FieldElement::zero(), ObjectType::boolean);
                    //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this
                } else if l_constant.is_some() && r_constant.is_some() {
                    let res = if l_constant.unwrap() >= r_constant.unwrap() {
                        FieldElement::one()
                    } else {
                        FieldElement::zero()
                    };
                    return NodeEval::Const(res, ObjectType::boolean);
                }
            }
            Operation::ult => {
                if r_is_zero {
                    return NodeEval::Const(FieldElement::zero(), ObjectType::boolean);
                    //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this
                } else if l_constant.is_some() && r_constant.is_some() {
                    let res = if l_constant.unwrap() < r_constant.unwrap() {
                        FieldElement::one()
                    } else {
                        FieldElement::zero()
                    };
                    return NodeEval::Const(res, ObjectType::boolean);
                }
            }
            Operation::ule => {
                if l_is_zero {
                    return NodeEval::Const(FieldElement::one(), ObjectType::boolean);
                    //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this
                } else if l_constant.is_some() && r_constant.is_some() {
                    let res = if l_constant.unwrap() <= r_constant.unwrap() {
                        FieldElement::one()
                    } else {
                        FieldElement::zero()
                    };
                    return NodeEval::Const(res, ObjectType::boolean);
                }
            }
            Operation::ugt => {
                if l_is_zero {
                    return NodeEval::Const(FieldElement::zero(), ObjectType::boolean);
                // u<0 is false for unsigned u
                //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this
                } else if l_constant.is_some() && r_constant.is_some() {
                    let res = if l_constant.unwrap() > r_constant.unwrap() {
                        FieldElement::one()
                    } else {
                        FieldElement::zero()
                    };
                    return NodeEval::Const(res, ObjectType::boolean);
                }
            }
            Operation::eq => {
                if self.lhs == self.rhs {
                    return NodeEval::Const(FieldElement::one(), ObjectType::boolean);
                } else if l_constant.is_some() && r_constant.is_some() {
                    if l_constant.unwrap() == r_constant.unwrap() {
                        return NodeEval::Const(FieldElement::one(), ObjectType::boolean);
                    } else {
                        return NodeEval::Const(FieldElement::zero(), ObjectType::boolean);
                    }
                }
            }
            Operation::ne => {
                if l_constant.is_some() && r_constant.is_some() {
                    if r_constant.unwrap() != l_constant.unwrap() {
                        return NodeEval::Const(FieldElement::one(), ObjectType::boolean);
                    } else {
                        return NodeEval::Const(FieldElement::zero(), ObjectType::boolean);
                    }
                }
            }
            Operation::and => {
                if l_is_zero {
                    return *lhs;
                } else if r_is_zero {
                    return *rhs;
                } else if l_constant.is_some() {
                    return *rhs;
                } else if r_constant.is_some() {
                    return *lhs;
                } else if self.lhs == self.rhs {
                    return *lhs;
                }
            }
            Operation::or => {
                if l_is_zero {
                    return *rhs;
                } else if r_is_zero {
                    return *lhs;
                } else if l_constant.is_some() {
                    return *lhs;
                } else if r_constant.is_some() {
                    return *rhs;
                } else if self.lhs == self.rhs {
                    return *lhs;
                }
            }
            Operation::not => {
                if l_is_zero {
                    return NodeEval::Const(FieldElement::one(), ObjectType::boolean);
                } else if l_constant.is_some() {
                    return NodeEval::Const(FieldElement::zero(), ObjectType::boolean);
                }
            }
            Operation::xor => {
                if self.lhs == self.rhs {
                    return NodeEval::Const(FieldElement::zero(), ObjectType::boolean);
                }
                if l_is_zero {
                    return *rhs;
                }
                if r_is_zero {
                    return *lhs;
                } else if l_constant.is_some() && r_constant.is_some() {
                    return NodeEval::Const(FieldElement::zero(), ObjectType::boolean);
                } else if l_constant.is_some() {
                    todo!();
                    //TODO generate 'not rhs' instruction
                } else if r_constant.is_some() {
                    todo!();
                    ////TODO generate 'not lhs' instruction
                }
            }
            Operation::phi => (), //Phi are simplified by simply_phi()
            _ => (),
            // Operation::fmod
            // Operation::fneg
            // Operation::fdiv

            // Operation::sgt
            // Operation::sge
            // Operation::slt
            // Operation::sle

            // Operation::gt
            // Operation::lte
            // Operation::gte
            // Operation::cast =
            // Operation::trunc
            // jumps, ass, nop => do not evaluate
            // Operation::ass => todo!(),
            // Operation::nop => todo!(),
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
                }
            }
            _ => (),
        }
        if is_commutative(self.operator) && self.rhs < self.lhs {
            std::mem::swap(&mut self.rhs, &mut self.lhs);
        }
    }
}

//adpated from LLVM IR
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Operation {
    add,      //(+)
    safe_add, //(+) safe addtion
    sub,      //(-)
    safe_sub, //(-) safe substraction
    mul,      //(*)
    safe_mul, //(*) safe multiplication
    udiv,     //(/) unsigned division
    sdiv,     //(/) signed division
    urem,     //(%) modulo; remainder of unsigned division
    srem,     //(%) remainder of signed division
    fmod,     //(%) remainder of the floating point division
    fneg,     //(-) negation of a float
    fdiv,     //(/) floating point division
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
    // todo: call, br,..
    nop, // no op

    eq_gate, //write a gate enforcing equality of the two sides (to support the constrain statement)
             //memory todo: load, store, getelementptr?
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
    match op_code {
        Operation::add => true,
        Operation::safe_add => true,
        Operation::sub => true,
        Operation::safe_sub => true,
        Operation::mul => true,
        Operation::safe_mul => true,
        Operation::udiv => true, //(/) unsigned division
        Operation::sdiv => true,
        Operation::urem => true,
        Operation::srem => true,
        Operation::fmod => true,
        Operation::fneg => true,
        Operation::fdiv => true,
        Operation::div => true,
        Operation::eq => true,
        Operation::ne => true,
        Operation::ugt => true,
        Operation::uge => true,
        Operation::ult => true,
        Operation::ule => true,
        Operation::sgt => true,
        Operation::sge => true,
        Operation::slt => true,
        Operation::sle => true,
        Operation::lt => true,
        Operation::gt => true,
        Operation::lte => true,
        Operation::gte => true,
        Operation::and => true,
        Operation::not => false,
        Operation::or => true,
        Operation::xor => true,
        Operation::cast => false,
        Operation::ass => false,
        Operation::trunc => true,
        Operation::jne | Operation::jeq | Operation::jmp => false,
        Operation::phi => false,
        Operation::nop => false,
        Operation::eq_gate => true,
    }
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
    }
}

pub fn get_witness_from_object(obj: &Object) -> Option<Witness> {
    match obj {
        Object::Integer(i) => Some(i.witness),
        Object::Array(_) => todo!(),
        Object::Constants(_) => None,
        _ => obj.witness(), // These will
    }
}
