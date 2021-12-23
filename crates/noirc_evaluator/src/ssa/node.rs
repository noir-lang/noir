use std::collections::LinkedList;
use std::collections::HashMap;
use std::convert::TryInto;

use num_bigint::BigUint;
use acvm::FieldElement;
use arena;
use noirc_frontend::{UnaryOp, hir_def::{
    expr::{
        HirBinaryOp, HirBinaryOpKind, HirBlockExpression, HirCallExpression, HirExpression,
        HirForExpression, HirLiteral,
    },
    stmt::{HirConstrainStatement, HirLetStatement, HirPrivateStatement, HirStatement},
}};

use crate::object::Object;
use num_traits::identities::Zero;
//n.b




pub trait Node {
    fn get_type(&self) -> ObjectType;
    fn print(&self) -> String;
    //fn get_bit_size(&self) -> u32;
    fn get_id(&self) -> arena::Index;
    fn bits(&self) -> u32;              //bit size of the node
}


impl Node for Variable {
    fn get_type(&self) -> ObjectType{
        self.obj_type
    }

    fn print(&self) -> String {
        //format!("{}({:?})", self.name, self.id)
        format!("{}", self.name)
    }

    fn bits(&self) -> u32 {
        self.get_type().bits()
    }

    fn get_id(&self) -> arena::Index {
        self.id
    }
 }

 impl NodeObj {
     pub fn new_constant_bool(value: bool) -> NodeObj {     
         let val = if value {1_u32} else {0_u32};
         NodeObj::Const(Constant{ id: crate::ssa::code_gen::ParsingContext::dummy_id(),
           value: BigUint::from(val),
            value_str: String::new(),
            value_type: ObjectType::boolean,
        })
     }

     pub fn new_constant_int(value: u32, bit_size: u32, is_signed: bool) -> NodeObj {     
        NodeObj::Const (Constant{ id: crate::ssa::code_gen::ParsingContext::dummy_id(),
          value: BigUint::from(value),
           value_str: String::new(),
           value_type: if is_signed {ObjectType::signed(bit_size)} else {ObjectType::unsigned(bit_size)},
       })
    }
 }

 impl Node for NodeObj{
    fn get_type(&self) -> ObjectType{
        match self {
            NodeObj::Obj(o)         => o.get_type(),
            NodeObj::Instr(i)       => i.res_type,
            NodeObj::Const(o)       => o.value_type,
        }
    }

    fn print(&self) -> String {
        match self {
            NodeObj::Obj(o)         => o.print(),
            NodeObj::Instr(i)       => i.print_i(),
            NodeObj::Const(c)       => c.print(),
        }
    }

    fn bits(&self) -> u32{
        match self {
            NodeObj::Obj(o)         => o.bits(),
            NodeObj::Instr(i)       => i.res_type.bits(), 
            NodeObj::Const(c)       => c.bits(),
        }
    }

    fn get_id(&self) -> arena::Index {
        match self {
            NodeObj::Obj(o)         => o.get_id(),
            NodeObj::Instr(i)       => i.idx,
            NodeObj::Const(c)       => c.get_id(),
        }
    }
 }

 impl Node for Constant{
    fn get_type(&self) -> ObjectType{
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
impl Variable
{
    // pub fn get_current_value(&self) -> arena::Index
    // {
    //     let ctx =  super::CFG.lock().unwrap();
    //     if (self.root.is_some())
    //     {
    //         let rootObj = ctx.get_object(self.root.unwrap());
    //         if (rootObj.is_some()) {
    //             match rootObj.unwrap() {
    //                 NodeObj::Obj(o) => {return o.get_current_value();},
    //                 _ => todo!(),//Err
    //             }
    //         }
    //     }
    //     let cur_valueObj = ctx.get_object(self.cur_value);
    //     if (cur_valueObj.is_some()) {
    //         match cur_valueObj.unwrap() {
    //             NodeObj::Obj(o) => {return o.id;},
    //             _ => todo!(),//Err
    //         }
    //     }
    //     return self.id;
    // }
}

#[derive(Debug)]
pub enum NodeObj {
    Obj(Variable),    
    Instr(Instruction),
    Const(Constant),
}

#[derive(Debug)]
pub struct Constant{
    pub id: arena::Index,
    pub value: BigUint,             //TODO use FieldElement instead
    pub value_str: String,          //TODO ConstStr subtype 
    pub value_type: ObjectType, 
}
#[derive(Debug)]
pub struct Variable {
    pub id : arena::Index,
    pub obj_type: ObjectType,
    pub name: String,   
    pub cur_value : arena::Index,               //for generating the SSA form, current value of the object during parsing of the AST
    pub root : Option<arena::Index>,             //when generating SSA, assignment of an object creates a new one which is linked to the original one
    //TODO clarify where cur_value and root is stored, and also this:
    //  pub max_bits: u32,                  //max possible bit size of the expression
    //  pub max_value: Option<BigUInt>,     //maximum possible value of the expression, if less than max_bits
}

#[derive(Copy, Clone, Debug)]
pub enum ObjectType{
    native_field,
   // custom_field(BigUint), //TODO requires copy trait for BigUint
    boolean,
    unsigned(u32),      //bit size
    signed(u32),        //bit size
    custom(u32),        //user-defined struct, u32 refers to the id of the type in...?todo
    //array(ObjectType),  TODO we should have primitive type and composite types
    //TODO big_int
    //TODO floats
    none,        //not an object
}

impl ObjectType {
    fn is_signed(&self) -> bool {
        match self {
            ObjectType::signed(_) => true,
            _ => false,
        }
    }

    fn is_unsigned(&self) -> bool {
        match self {
            ObjectType::unsigned(_) => true, 
            _ => false,
        }
    }

    fn is_field(&self) -> bool {
        match self {
            ObjectType::native_field => true,
           // ObjectType::custom_field(_)=> true,
            _ => false,
        }
    }

    pub fn from_type(t: noirc_frontend::Type) -> ObjectType
    {
        match t {
            noirc_frontend::Type::FieldElement(_) => ObjectType::native_field,
            Array   => ObjectType::none, //TODO
            noirc_frontend::Type::Integer(ftype, sign, bit_size) => {
                match sign {            //todo FieldElementType?
                    noirc_frontend::Signedness::Signed =>  ObjectType::signed(bit_size),
                    noirc_frontend::Signedness::Unsigned => ObjectType::unsigned(bit_size),
                }
            }
            Bool => ObjectType::boolean,
            _ => ObjectType::none, //todo Error,Unspecified, Unknown,Unit

        }        
    }

    pub fn get_type_from_object(obj: Object) -> ObjectType {
        let toto=match obj.clone() {
            Object::Arithmetic(a) =>dbg!("aa".to_string()),  //TODO
            Object::Array(a) => dbg!("aaa".to_string()),
            Object::Constants(_)  => dbg!("const- antive".to_string()),
            Object::Integer(i)  => dbg!(format!("int {:?}", i.num_bits)), //TODO signed or unsigned?
            Object::Linear(l) => dbg!("linear".to_string()),          //TODO
            Object::Null =>dbg!("none".to_string()),
        };

        match obj {
            Object::Arithmetic(a) =>{todo!();ObjectType::native_field},
            Object::Array(a) => {todo!();ObjectType::none},
            Object::Constants(_)  => ObjectType::native_field,
            Object::Integer(i)  =>  ObjectType::signed(i.num_bits), //TODO signed or unsigned?
            Object::Linear(l) => {todo!();ObjectType::native_field},     
            Object::Null => ObjectType::none,
        }
    }

    pub fn bits(&self) -> u32 {
        match self {
            ObjectType::boolean         => 1,
            ObjectType::native_field    => FieldElement::max_num_bits(), //TODO is it the correct value?
            ObjectType::none            => 0,
            ObjectType::signed(c)   => *c,
            ObjectType::unsigned(c) => *c,
            ObjectType::custom(_)        => todo!(),
        }
    }
}
#[derive(Clone, Debug)] 
pub struct Instruction {
    pub idx: arena::Index,
    pub operator : Operation,
    pub rhs : arena::Index,
    pub lhs: arena::Index,
    pub res_type: ObjectType,   //result type
    //prev,next: should have been a double linked list so that we can easily remove an instruction during optimisation phases, but I can't make it in rust...
    pub parent_block : arena::Index,
    pub is_deleted: bool,
    pub res_name: String,
    pub bit_size: u32,      //TODO only for the truncate instruction...: bits size of the max value of the lhs.. a merger avec ci dessous!!!TODO
    pub max_value: BigUint, //TODO only for sub instruction: max value of the rhs

    //temp:
    pub phi_arguments: Vec<(arena::Index, arena::Index)>,
}

impl Instruction{

    pub fn new(op_code: Operation, lhs: arena::Index, rhs: arena::Index, r_type: ObjectType, parent_block: Option<arena::Index>) -> Instruction {
        let id0 = crate::ssa::code_gen::ParsingContext::dummy_id();
        let p_block = parent_block.unwrap_or(id0);
        Instruction {
            idx: id0,
            operator: op_code,
            lhs: lhs,
            rhs: rhs,
            res_type: r_type,
            res_name: String::new(),
            is_deleted: false,
            parent_block: p_block,
            bit_size: 0,
            max_value: BigUint::zero(),
            phi_arguments: Vec::new(),
        }
    }


    pub fn print_i(&self) ->String { 
        if self.res_name.is_empty() {
            format!("{:?}", self.idx)
        }
        else {
            self.res_name.clone()
        }
    }

        //indicates if the operation is a substraction
        pub fn is_sub(&self) -> bool
        {
            match self.operator {
                Operation::sub | Operation::ssub => true,
                _ => false,
            }
        }

    //indicates whether the left and/or right operand of the instruction is required to be truncated to its bit-width
    pub fn truncate_required(&self, lhs_bits: u32, rhs_bits: u32) -> (bool,bool)
    {
        match self.operator {
            Operation::add => (false,false),
            Operation::sadd => (false,false), 
            Operation::sub => (false,false),
            Operation::ssub => (false,false),
            Operation::mul => (false,false),
            Operation::smul => (false,false),
            Operation::udiv => (true,true),  
            Operation::sdiv => (true,true),  
            Operation::urem => (true,true),  
            Operation::srem => (true,true),  
            Operation::fmod => (true,true),     //to check
            Operation::fneg => (false,false),     //to check
            Operation::fdiv => (true,true),      //to check
            Operation::div => (false,false),  
            Operation::eq => (true,true),  
            Operation::ne => (true,true),  
            Operation::ugt => (true,true),  
            Operation::uge => (true,true),  
            Operation::ult => (true,true),  
            Operation::ule => (true,true),  
            Operation::sgt => (true,true),  
            Operation::sge => (true,true),  
            Operation::slt => (true,true),  
            Operation::sle => (true,true),  
            Operation::lt => (true,true),  
            Operation::gt => (true,true),  
            Operation::lte => (true,true),  
            Operation::gte => (true,true),  
            Operation::and => (true,true),  
            Operation::not => (true,true),  
            Operation::or => (true,true),  
            Operation::xor => (true,true),  
            Operation::cast => {
                if lhs_bits != rhs_bits {
                    return (false, true);
                }
                (false,false)
            },
            Operation::ass => {
                assert!(lhs_bits == rhs_bits);
                (false,false)
            },
            Operation::trunc | Operation::phi => (false,false), 
            Operation::nop | Operation::jne | Operation::jeq | Operation::jmp => (false,false),
        }
    }

    pub fn get_max_value(&self, lhs_max: BigUint, rhs_max:  BigUint ) -> BigUint
    {
        match self.operator {
            Operation::add => lhs_max + rhs_max,
            Operation::sadd => todo!(), 
            Operation::sub => lhs_max + rhs_max,
            Operation::ssub => todo!(),
            Operation::mul => lhs_max * rhs_max,
            Operation::smul => todo!(),
            Operation::udiv => lhs_max,  
            Operation::sdiv => todo!(),
            Operation::urem => rhs_max -  BigUint::from(1_u32),    
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
            Operation::trunc => BigUint::min(lhs_max, BigUint::from(2_u32).pow(rhs_max.try_into().unwrap()) - BigUint::from(1_u32)), 
            //'a = b': a and b must be of same type.
            Operation::ass => rhs_max,     
            Operation::nop | Operation::jne | Operation::jeq | Operation::jmp => todo!(),
            Operation::phi => BigUint::max(lhs_max, rhs_max),   //TODO operands are in phi_arguments, not lhs/rhs!!
        }
    }



    pub fn get_const_value(c: &Constant) -> (u32, u32) {
        //....todo...only u32 for now... should also provide the sign
        match c.value_type {
            ObjectType::boolean => (if c.value.is_zero() {0} else {1} , 1),
            ObjectType::native_field => todo!(), //to field(value), field:prime
            ObjectType::signed(b) | ObjectType::unsigned(b)=> (c.value.clone().try_into().unwrap(), b),
            _ => todo!(),
        }
    }


    //Performs constant folding, arithmetic and boolean simplification
    //Returns the index of the simplified instruction, or, if no index, the constant which should replace the instruction
    //The third result is for boolean constant.
    pub fn simplify<'a>(&self, lhs: & NodeObj, rhs: & NodeObj) -> (Option<arena::Index>,Option<FieldElement>,Option<FieldElement>) {
        let mut r_is_zero = false;
        let mut l_is_zero = false;
        let mut l_constant = None;
        let mut r_constant = None;
        let mut r_bsize = 0;
        let mut l_bsize = 0;
        let mut l_sign = false;  //TODO
        match lhs {
            NodeObj::Const(c) => {
                l_is_zero = c.value == BigUint::zero();
                let cv = Instruction::get_const_value(c);
                l_constant = Some(cv.0);
                l_bsize = cv.1;
            },
            _ => (),
        }
        match rhs {
            NodeObj::Const(c) => {
                r_is_zero = c.value == BigUint::zero();
                let cv = Instruction::get_const_value(c);
                r_constant = Some(cv.0);
                r_bsize = cv.1;
            },
            _ => (),
        }
        let mut result = self;
        match self.operator {
            Operation::add | Operation::sadd => {
                if (r_is_zero) {
                    return (Some(self.lhs), None, None);
                } else if (l_is_zero) {
                    return (Some(self.rhs), None, None);
                } else
                //constant folding - TODO - only for integers; NO modulo for field elements - May be we should have a different opcode for field addition? 
                if l_constant.is_some() && r_constant.is_some() {
                    assert!(l_bsize == r_bsize);
                    let res_value = (l_constant.unwrap() + r_constant.unwrap()) % l_bsize;    
                    return (None, Some(FieldElement::from(res_value as i128)), None);
                }
                //if only one is const, we could try to do constant propagation but this will be handled by the arithmetization step anyways
                //so it is probably not worth it.
                //same for x+x vs 2*x
            },
            Operation::sub | Operation::ssub => {
                if (r_is_zero) {
                    return (Some(self.lhs), None, None);
                }
                if self.lhs == self.rhs {
                    return (None, Some(FieldElement::zero()), None);
                }
                //constant folding - TODO - only for integers; NO modulo for field elements - May be we should have a different opcode? 
                if l_constant.is_some() && r_constant.is_some() {
                    assert!(l_bsize == r_bsize);
                    let res_value = (l_constant.unwrap() - r_constant.unwrap()) % l_bsize;    
                    return (None, Some(FieldElement::from(res_value as i128)), None);
                }
            },
            Operation::mul | Operation::smul => {
                if (r_is_zero) {
                    return (Some(self.rhs), None, None);
                } else if (l_is_zero) {
                    return (Some(self.lhs), None, None);
                }else if l_constant.is_some() && l_constant.unwrap() == 1 {
                    return (Some(self.rhs), None, None);
                }
                else if r_constant.is_some() && r_constant.unwrap() == 1 {
                    return (Some(self.lhs), None, None);
                }
                //constant folding - TODO - only for integers; NO modulo for field elements - May be we should have a different opcode? 
                else if l_constant.is_some() && r_constant.is_some() {
                    assert!(l_bsize == r_bsize);
                    let res_value = (l_constant.unwrap() * r_constant.unwrap()) % l_bsize;    
                    return (None, Some(FieldElement::from(res_value as i128)), None);
                }
                //if only one is const, we could try to do constant propagation but this will be handled by the arithmetization step anyways
                //so it is probably not worth it.
            },
            Operation::udiv | Operation::sdiv | Operation::div => {
                if r_is_zero {
                    todo!("Panic - division by zero");
                } else if l_is_zero {
                    return (Some(self.lhs), None, None);  //TODO should we ensure rhs != 0 ???
                }
                //else if r_constant.is_some() {
                    //TODO same as lhs*1/r
                    //return (Some(self.lhs), None, None);
                //}        
                //constant folding - TODO 
                else if l_constant.is_some() && r_constant.is_some() {
                    todo!();
               }
            },
            Operation::urem | Operation::srem  => {
                if (r_is_zero) {
                    todo!("Panic - division by zero");
                } else if (l_is_zero) {
                    return (Some(self.lhs), None, None);  //TODO what is the correct result? and should we ensure rhs != 0 ???
                }
                //constant folding - TODO 
                else if l_constant.is_some() && r_constant.is_some() {
                todo!();
              }
            },
            Operation::uge  =>  { 
                if r_is_zero {
                    return (None, Some(FieldElement::zero()), None);
                    //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this
                } 
                else if l_constant.is_some() && r_constant.is_some() {
                    let res = if l_constant.unwrap() >= r_constant.unwrap() {FieldElement::one()} else {FieldElement::zero()};
                    return (Some(self.lhs), None, Some(res));
                }
            },
            Operation::ult=> { 
                if r_is_zero {
                    return (None, None, Some(FieldElement::zero()));
                    //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this   
                } 
                else if l_constant.is_some() && r_constant.is_some() {
                    let res = if l_constant.unwrap() < r_constant.unwrap() {FieldElement::one()} else {FieldElement::zero()};
                    return (Some(self.lhs), None, Some(res));
                }
            },
            Operation::ule  =>  { 
                if l_is_zero {
                    return (None, None, Some(FieldElement::one()));
                    //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this   
                } 
                else if l_constant.is_some() && r_constant.is_some() {
                    let res = if l_constant.unwrap() <= r_constant.unwrap() {FieldElement::one()} else {FieldElement::zero()};
                    return (Some(self.lhs), None, Some(res));
                }
            },
            Operation::ugt => { 
                if l_is_zero {
                    return (None, None, Some(FieldElement::zero()));    // u<0 is false for unsigned u
                    //n.b we assume the type of lhs and rhs is unsigned because of the opcode, we could also verify this       
                } 
                else if l_constant.is_some() && r_constant.is_some() {
                    let res = if l_constant.unwrap() > r_constant.unwrap() {FieldElement::one()} else {FieldElement::zero()};
                    return (Some(self.lhs), None, Some(res));
                }
            },
            Operation::eq => {
                if self.lhs == self.rhs {
                    return (None, None, Some(FieldElement::one()));
                }else if l_constant.is_some() && r_constant.is_some() {
                    if l_constant.unwrap() == r_constant.unwrap() {
                        return (None, None, Some(FieldElement::one()));
                    } else {
                        return (None, None, Some(FieldElement::zero()));
                    }
                }
            },  
            Operation::ne => {
                if l_constant.is_some()  && r_constant.is_some()  {
                    if r_constant.unwrap() != l_constant.unwrap() {
                    return (None, None, Some(FieldElement::one()));
                }
                else {
                    return (None, None, Some(FieldElement::zero())); 
                }
            }
            },
            Operation::and => {
                if l_is_zero {
                    return (Some(self.lhs), None, None);
                }else if r_is_zero {
                    return (Some(self.rhs), None, None);
                } else if l_constant.is_some() {
                    return (Some(self.rhs), None, None);
                } else 
                if r_constant.is_some() {
                    return (Some(self.lhs), None, None);
                } else
                if self.lhs == self.rhs {
                    return (Some(self.lhs), None, None);
                }
            },  
            Operation::or  => {
                if l_is_zero {
                    return (Some(self.rhs), None, None);
                } else
                if r_is_zero {
                    return (Some(self.lhs), None, None);
                } else
                if l_constant.is_some() {
                    return (Some(self.lhs), None, None);
                } else
                if r_constant.is_some() {
                    return (Some(self.rhs), None, None);
                } else
                if self.lhs == self.rhs {
                    return (Some(self.lhs), None, None);
                }
            },
            Operation::not  => {
                if l_is_zero {
                    return (None, None, Some(FieldElement::one()));
                } else
                if l_constant.is_some() {
                    return (None, None, Some(FieldElement::zero()));
                }

            },
             Operation::xor => {  
                if self.lhs == self.rhs {
                    return (None, None, Some(FieldElement::zero()));
                }         
             if l_is_zero {
                return (Some(self.rhs), None, None);
            }
            if r_is_zero {
                return (Some(self.lhs), None, None);
            } else 
            if (l_constant.is_some() &&  r_constant.is_some())
            {
                return (None, None, Some(FieldElement::zero())); 
            } else
            if l_constant.is_some() {
                todo!();
                //TODO generate 'not rhs' instruction
            } else
            if r_constant.is_some() {
                todo!();
                ////TODO generate 'not lhs' instruction
            }
            
            },
            _ => todo!(),
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
            // Operation::ass => todo!(),
            // Operation::nop => todo!(),
        }
        return (Some(self.idx), None, None);
    } 
}

//adpated from LLVM IR
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)] 
pub enum Operation {
    add,        //(+)
    sadd,       //(+) safe addtion
    sub,        //(-)
    ssub,       //(-) safe substraction
    mul,        //(*)
    smul,       //(*) safe multiplication
    udiv,       //(/) unsigned division
    sdiv,       //(/) signed division
    urem,        //(%) modulo; remainder of unsigned division
    srem,       //(%) remainder of signed division
    fmod,       //(%) remainder of the floating point division
    fneg,       //(-) negation of a float
    fdiv,       //(/) floating point division
    div,        //(/) field division
    eq,         //(==) equal 
    ne,         //(!=) not equal
    ugt,        //(>) unsigned greater than
    uge,        //(>=) unsigned greater or equal
    ult,        //(<) unsigned less than
    ule,        //(<=) unsigned less or equal
    sgt,        //(>) signed greater than
    sge,        //(>=) signed greater or equal
    slt,        //(<) signed less than
    sle,        //(<=) signed less or equal
    lt,         //(<) field less
    gt,         //(>) field greater
    lte,        //(<=) field less or equal
    gte,        //(<=) field greater or equal
    and,
    not, 
    or,
    xor,
    cast,       //convert type
    ass,        //assignement
    trunc,      //truncate

    //control flow
    jne,        //jump on not equal
    jeq,        //jump on equal
    jmp,        //unconditional jump
    phi,      
    // todo: call, br,..
    nop,        // no op
   

    //memory todo: load, store, getelementptr?
}

pub fn is_binary(op_code: Operation) -> bool
{
    match op_code {
        Operation::add => true,
        Operation::sadd => true,  
        Operation::sub => true,
        Operation::ssub => true,
        Operation::mul => true,
        Operation::smul => true,
        Operation::udiv => true,       //(/) unsigned division
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
    }
}

pub fn to_operation(op_kind: HirBinaryOpKind, op_type: ObjectType) -> Operation{
    match op_kind {
        HirBinaryOpKind::Add            => Operation::add,
        HirBinaryOpKind::Subtract       => Operation::sub,
        HirBinaryOpKind::Multiply       => Operation::mul,
        HirBinaryOpKind::Equal          => Operation::eq,
        HirBinaryOpKind::NotEqual       => Operation::ne,
        HirBinaryOpKind::And            => Operation::and,
        HirBinaryOpKind::Or             => Operation::or,
        HirBinaryOpKind::Xor            => Operation::xor,
        HirBinaryOpKind::Divide         => {
            if op_type.is_signed(){
                return Operation::sdiv;
            }
            if op_type.is_unsigned(){
                return Operation::udiv;
            }
            if op_type.is_field(){
                return Operation::div;
            }
            todo!("invalid type");//TODO error
            return  Operation::nop;
            },
        HirBinaryOpKind::Less         => {
            if op_type.is_signed(){
                    return Operation::slt;
            }
            if op_type.is_unsigned(){
                return Operation::ult;
            }
            if op_type.is_field(){
                return Operation::lt;
            }  
            return  Operation::nop;//TODO error
        },
        HirBinaryOpKind::Greater         => {
            if op_type.is_signed(){
                    return Operation::sgt;
            }
            if op_type.is_unsigned(){
                return Operation::ugt;
            }
            if op_type.is_field(){
                return Operation::gt;
            }  
            return  Operation::nop;//TODO error
        },
        HirBinaryOpKind::LessEqual         => {
            if op_type.is_signed(){
                    return Operation::sle;
            }
            if op_type.is_unsigned(){
                return Operation::ule;
            }
            if op_type.is_field(){
                return Operation::lte;
            }  
            return  Operation::nop;//TODO error
        },
        HirBinaryOpKind::GreaterEqual         => {
            if op_type.is_signed(){
                    return Operation::sge;
            }
            if op_type.is_unsigned(){
                return Operation::uge;
            }
            if op_type.is_field(){
                return Operation::gte;
            }  
            return  Operation::nop;//TODO error
        },
        HirBinaryOpKind::Assign => Operation::ass,  //TODO
        
    }

}
pub struct BasicBlock {   
    pub idx : arena::Index,
    //kind: type of block, e.g join, 
    pub dominator:  Option<arena::Index>,        //direct dominator
    pub dominated:Vec<arena::Index>,            //dominated sons
    pub predecessor : Vec<arena::Index>,       //for computing the dominator tree
    pub left: Option<arena::Index>,         //sequential successor
    pub right: Option<arena::Index>,        //jump successor
    pub instructions : Vec<arena::Index>,   
   
    pub value_array: HashMap<arena::Index, arena::Index>,   //for generating the ssa form
    pub value_name: HashMap<arena::Index, u32>,             //only for pretty print
}


impl BasicBlock{

    pub fn new(left: arena::Index) -> BasicBlock {
        BasicBlock {
            idx : crate::ssa::code_gen::ParsingContext::dummy_id(),
            predecessor : Vec::new(), 
            left: Some(left),
            right: None,
            instructions : Vec::new(), 
            value_array : HashMap::new(), 
            value_name : HashMap::new(), 
            dominator: None,//crate::ssa::code_gen::ParsingContext::dummy_id(),
            dominated: Vec::new(),  
        }
    }

    pub fn get_current_value(&self, idx: arena::Index) -> arena::Index
    {
        match self.value_array.get(&idx) {
            Some(cur_idx) => *cur_idx,
            None => idx
        }
    }

    //When generating a new instance of a variable because of ssa, we update the value array
    //to link the two variables and also increment the counter for the variable name
    pub fn update_variable(&mut self, old_value: arena::Index, new_value: arena::Index) {
        self.value_array.insert(old_value, new_value);
        self.value_name.entry(old_value).or_insert(1);
        self.value_name.insert(old_value,self.value_name[&old_value] + 1);
    }

    pub fn get_value_name(&self, idx: arena::Index) -> u32
    {
        if self.value_name.contains_key(&idx) { 
            return self.value_name[&idx];
        }
        0
    }

    pub fn get_first_instruction(&self) -> arena::Index
    {
        self.instructions[0]
    }


}
