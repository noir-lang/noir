use crate::function_context::FunctionData;
use crate::fuzz_target_lib::fuzz_target;
use crate::fuzzer::FuzzerData;
use crate::instruction::{Instruction, InstructionBlock, Point, Scalar};
use crate::options::FuzzerOptions;
use crate::tests::common::{default_input_types, default_runtimes, default_witness};
use acvm::FieldElement;
use noir_ssa_fuzzer::typed_value::{NumericType, Type};

/// fn main(lo: Field) -> pub Field {
///     let scalar_1 = std::embedded_curve_ops::EmbeddedCurveScalar::new(lo, 0);
///     let scalar_2 = std::embedded_curve_ops::EmbeddedCurveScalar::new(lo * 2, 0);
///     let point_1 = std::embedded_curve_ops::fixed_base_scalar_mul(scalar_1);
///     let point_2 = std::embedded_curve_ops::fixed_base_scalar_mul(scalar_2);
///     let res = std::embedded_curve_ops::embedded_curve_add(point_1, point_2);
///     res.y
/// }
/// lo = 1
/// [nargo_tests] Circuit output: Field(8902249110305491597038405103722863701255802573786510474664632793109847672620)
#[test]
fn smoke_test_embedded_curve_add() {
    let _ = env_logger::try_init();
    let add_instruction = Instruction::PointAdd {
        p1: Point {
            scalar: Scalar { field_lo_idx: 1, field_hi_idx: 0 },
            derive_from_scalar_mul: true,
            is_infinite: false,
        },
        p2: Point {
            scalar: Scalar { field_lo_idx: 2, field_hi_idx: 0 },
            derive_from_scalar_mul: true,
            is_infinite: false,
        },
        predicate: true,
    };
    let block = InstructionBlock { instructions: vec![add_instruction] };
    let commands = vec![];
    let function = FunctionData {
        commands,
        input_types: default_input_types(),
        return_instruction_block_idx: 0,
        return_type: Type::Numeric(NumericType::Field),
    };
    let data = FuzzerData {
        instruction_blocks: vec![block],
        functions: vec![function],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(data, default_runtimes(), FuzzerOptions::default());
    assert_eq!(
        result.get_return_witnesses()[0],
        FieldElement::try_from_str(
            "8902249110305491597038405103722863701255802573786510474664632793109847672620"
        )
        .unwrap()
    );
}

/// fn main(lo1: Field, lo2: Field) -> pub Field {
///    let base_scalar = std::embedded_curve_ops::EmbeddedCurveScalar::new(1, 0); // 1
///    let scalar_1 = std::embedded_curve_ops::EmbeddedCurveScalar::new(lo1, 0);
///    let scalar_2 = std::embedded_curve_ops::EmbeddedCurveScalar::new(lo2, 0);
///    let generator = std::embedded_curve_ops::fixed_base_scalar_mul(base_scalar); // Generator * 1
///    
///    let res = std::embedded_curve_ops::multi_scalar_mul([generator, generator], [scalar_1, scalar_2]);
///    res.y
/// }
/// "lo1" = 2
/// "lo2" = 4
/// Circuit output: Field(-3851299760922698091325321774664553326049887197487063802849283717866939395465)
#[test]
fn smoke_test_embedded_multi_scalar_mul() {
    let _ = env_logger::try_init();
    let base_scalar = Scalar { field_lo_idx: 1, field_hi_idx: 0 };
    let scalar_1 = Scalar { field_lo_idx: 2, field_hi_idx: 0 };
    let scalar_2 = Scalar { field_lo_idx: 4, field_hi_idx: 0 };
    let gen_point = Point { scalar: base_scalar, derive_from_scalar_mul: true, is_infinite: false };
    let instruction = Instruction::MultiScalarMul {
        points_and_scalars: vec![(gen_point, scalar_1), (gen_point, scalar_2)],
        predicate: true,
    };
    let block = InstructionBlock { instructions: vec![instruction] };
    let commands = vec![];
    let function = FunctionData {
        commands,
        input_types: default_input_types(),
        return_instruction_block_idx: 0,
        return_type: Type::Numeric(NumericType::Field),
    };
    let data = FuzzerData {
        instruction_blocks: vec![block],
        functions: vec![function],
        initial_witness: default_witness(),
    };
    let result = fuzz_target(data, default_runtimes(), FuzzerOptions::default());
    assert_eq!(
        result.get_return_witnesses()[0],
        FieldElement::try_from_str(
            "-3851299760922698091325321774664553326049887197487063802849283717866939395465"
        )
        .unwrap()
    );
}
