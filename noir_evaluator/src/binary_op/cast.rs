use crate::{Environment, Evaluator, Integer, Linear, Object, Type};

pub fn handle_cast_op(
    left: Object,
    right: Type,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Object {
    let num_bits = match right {
        Type::Integer(sign, num_bits) => num_bits,
        _ => panic!("Currently we do not support type casting for non integers"),
    };

    let casted_integer = match left {
        Object::Arithmetic(arith) => {
            let casted_integer = Integer::from_arithmetic(arith, num_bits, env, evaluator);
            casted_integer.constrain(evaluator);
            casted_integer
        }
        Object::Constants(_) => {
            panic!("Currently we do not support casting constants to a type")
        }
        Object::Linear(linear) => {
            let casted_integer = Integer::from_arithmetic(linear.into(), num_bits, env, evaluator);
            casted_integer.constrain(evaluator);
            casted_integer
        }
        Object::Integer(integer) => {
            // If we are casting a u8 to a u32, then this would require no extra constraints
            // Since all u8s can fit into u32
            // If we are casting a u32 to a u8, then this would require constraints

            let casted_integer = Integer::from_arithmetic(
                Linear::from(integer.witness.clone()).into(),
                num_bits,
                env,
                evaluator,
            );

            let should_constrain = integer.num_bits > num_bits;
            if should_constrain {
                casted_integer.constrain(evaluator);
            };
            casted_integer
        }
        x => panic!("Cannot cast type {:?} to an integer", x),
    };
    Object::Integer(casted_integer)
}
