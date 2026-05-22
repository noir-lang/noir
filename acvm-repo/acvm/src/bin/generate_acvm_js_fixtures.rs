/// Generates TypeScript files in `acvm-repo/acvm_js/test/shared` for acvm_js tests.
use std::{
    cell::RefCell,
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use acir::brillig::{ForeignCallParam, ForeignCallResult};
use acir::{
    AcirField, FieldElement,
    circuit::Program,
    native_types::{Witness, WitnessMap, WitnessStack},
    test_fixtures,
};
use acvm::pwg::{ACVM, ACVMStatus, AcirCallWaitInfo, ForeignCallWaitInfo};
use bn254_blackbox_solver::Bn254BlackBoxSolver;

fn field_to_hex(f: &FieldElement) -> String {
    format!("'0x{}'", f.to_hex())
}

fn witness_map_to_ts(map: &WitnessMap<FieldElement>) -> String {
    let entries: Vec<String> = map
        .clone()
        .into_iter()
        .map(|(w, f)| format!("  [{}, {}]", w.0, field_to_hex(&f)))
        .collect();
    format!("new Map([\n{}\n])", entries.join(",\n"))
}

fn bytes_to_ts(bytes: &[u8]) -> String {
    let nums: Vec<String> = bytes.iter().map(|b| b.to_string()).collect();
    format!("Uint8Array.from([\n{}\n])", nums.join(","))
}

fn foreign_call_inputs_to_ts(inputs: &[ForeignCallParam<FieldElement>]) -> String {
    let params: Vec<String> = inputs
        .iter()
        .map(|p| {
            let fields = p.fields();
            let hexes: Vec<String> = fields.iter().map(field_to_hex).collect();
            format!("[{}]", hexes.join(", "))
        })
        .collect();
    format!("[{}]", params.join(", "))
}

fn foreign_call_response_to_ts(values: &[ForeignCallParam<FieldElement>]) -> String {
    let params: Vec<String> = values
        .iter()
        .map(|p| match p {
            ForeignCallParam::Single(f) => field_to_hex(f),
            ForeignCallParam::Array(fs) => {
                let hexes: Vec<String> = fs.iter().map(field_to_hex).collect();
                format!("[\n    {}\n  ]", hexes.join(",\n    "))
            }
        })
        .collect();
    format!("[\n  {}\n]", params.join(",\n  "))
}

/// Executes a single function within a program, recursively handling ACIR sub-calls.
/// Witnesses for each completed function are appended to `witness_stack` in post-order
/// (innermost calls first), matching the order expected by the acvm_js test suite.
fn execute_function(
    program: &Program<FieldElement>,
    func_index: usize,
    initial_witness: WitnessMap<FieldElement>,
    backend: &Bn254BlackBoxSolver,
    oracle_resolver: &dyn Fn(&ForeignCallWaitInfo<FieldElement>) -> ForeignCallResult<FieldElement>,
    witness_stack: &mut WitnessStack<FieldElement>,
) -> WitnessMap<FieldElement> {
    let circuit = &program.functions[func_index];
    let mut acvm = ACVM::new(
        backend,
        &circuit.opcodes,
        initial_witness,
        &program.unconstrained_functions,
        &[],
    );

    loop {
        match acvm.solve() {
            ACVMStatus::Solved => break,
            ACVMStatus::InProgress => unreachable!(),
            ACVMStatus::Failure(err) => panic!("ACVM failure in func {func_index}: {err}"),
            ACVMStatus::RequiresForeignCall(info) => {
                let result = oracle_resolver(&info);
                acvm.resolve_pending_foreign_call(result);
            }
            ACVMStatus::RequiresAcirCall(AcirCallWaitInfo { id, initial_witness }) => {
                let called_idx = id.as_usize();
                let sub_witness = execute_function(
                    program,
                    called_idx,
                    initial_witness,
                    backend,
                    oracle_resolver,
                    witness_stack,
                );
                let return_witnesses = &program.functions[called_idx].return_values.0;
                let results: Vec<FieldElement> =
                    return_witnesses.iter().map(|w| *sub_witness.get(w).unwrap()).collect();
                acvm.resolve_pending_acir_call(results);
            }
        }
    }

    let witness = acvm.finalize();
    witness_stack.push(func_index as u32, witness.clone());
    witness
}

fn execute_simple(
    program: &Program<FieldElement>,
    initial_witness: WitnessMap<FieldElement>,
    backend: &Bn254BlackBoxSolver,
    oracle_resolver: &dyn Fn(&ForeignCallWaitInfo<FieldElement>) -> ForeignCallResult<FieldElement>,
) -> WitnessMap<FieldElement> {
    let mut stack = WitnessStack::default();
    let witness =
        execute_function(program, 0, initial_witness, backend, oracle_resolver, &mut stack);
    witness
}

fn no_oracles(_: &ForeignCallWaitInfo<FieldElement>) -> ForeignCallResult<FieldElement> {
    panic!("unexpected oracle call")
}

fn output_path(filename: &str) -> PathBuf {
    Path::new("acvm-repo/acvm_js/test/shared").join(filename)
}

fn write(path: &Path, content: &str) {
    std::fs::write(path, content)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", path.display()));
    println!("wrote {}", path.display());
}

fn main() {
    let backend = Bn254BlackBoxSolver::default();

    // ── addition ─────────────────────────────────────────────────────────────
    {
        let program = test_fixtures::addition_program();
        let bytecode = Program::serialize_program(&program);

        let initial: WitnessMap<FieldElement> = BTreeMap::from_iter([
            (Witness(1), FieldElement::from(1u128)),
            (Witness(2), FieldElement::from(2u128)),
        ])
        .into();

        let result = execute_simple(&program, initial.clone(), &backend, &no_oracles);

        let circuit = &program.functions[0];
        let result_witness = circuit.return_values.0.iter().next().unwrap();
        let expected_result = result.get(result_witness).unwrap();

        let content = format!(
            "// This file was automatically generated by running `just generate-acvm-js-fixtures`
// See `addition_circuit` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = {bytecode};

export const initialWitnessMap = {initial_map};
export const resultWitness = {rw};

export const expectedResult = {er};
",
            bytecode = bytes_to_ts(&bytecode),
            initial_map = witness_map_to_ts(&initial),
            rw = result_witness.0,
            er = field_to_hex(expected_result),
        );
        write(&output_path("addition.ts"), &content);
    }

    // ── multi_scalar_mul ─────────────────────────────────────────────────────
    {
        let program = test_fixtures::multi_scalar_mul_program();
        let bytecode = Program::serialize_program(&program);

        // Generator point of BN254 (G1): x=1, y as below; scalar=1; predicate=1.
        let initial: WitnessMap<FieldElement> = BTreeMap::from_iter([
            (Witness(1), FieldElement::from(1u128)),
            (
                Witness(2),
                FieldElement::from_hex(
                    "0000000000000002cf135e7506a45d632d270d45f1181294833fc48d823f272c",
                )
                .unwrap(),
            ),
            (Witness(3), FieldElement::zero()),
            (Witness(4), FieldElement::from(1u128)),
            (Witness(5), FieldElement::zero()),
            (Witness(6), FieldElement::from(1u128)),
        ])
        .into();

        let result = execute_simple(&program, initial.clone(), &backend, &no_oracles);

        let content = format!(
            "// This file was automatically generated by running `just generate-acvm-js-fixtures`
// See `multi_scalar_mul_circuit` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = {bytecode};

export const initialWitnessMap = {initial_map};
export const expectedWitnessMap = {expected_map};
",
            bytecode = bytes_to_ts(&bytecode),
            initial_map = witness_map_to_ts(&initial),
            expected_map = witness_map_to_ts(&result),
        );
        write(&output_path("multi_scalar_mul.ts"), &content);
    }

    // ── foreign_call ─────────────────────────────────────────────────────────
    {
        let program = test_fixtures::simple_brillig_foreign_call_program();
        let bytecode = Program::serialize_program(&program);

        let initial: WitnessMap<FieldElement> =
            BTreeMap::from_iter([(Witness(1), FieldElement::from(5u128))]).into();

        let oracle_call_name = RefCell::new(String::new());
        let oracle_call_inputs = RefCell::new(Vec::new());
        let oracle_response = RefCell::new(Vec::new());

        let result = execute_simple(&program, initial.clone(), &backend, &|info| {
            assert_eq!(info.function, "invert");
            *oracle_call_name.borrow_mut() = info.function.clone();
            *oracle_call_inputs.borrow_mut() = info.inputs.clone();

            let value = info.inputs[0].unwrap_field();
            let inverse = FieldElement::one() / value;
            let response = ForeignCallResult { values: vec![ForeignCallParam::Single(inverse)] };
            *oracle_response.borrow_mut() = response.values.clone();
            response
        });

        let oracle_call_name = oracle_call_name.into_inner();
        let oracle_call_inputs = oracle_call_inputs.into_inner();
        let oracle_response = oracle_response.into_inner();

        let content = format!(
            "// This file was automatically generated by running `just generate-acvm-js-fixtures`
// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = {bytecode};

export const initialWitnessMap = {initial_map};
export const expectedWitnessMap = {expected_map};

export const oracleCallName = '{oracle_name}';
export const oracleCallInputs = {oracle_inputs};
export const oracleResponse = {oracle_resp};
",
            bytecode = bytes_to_ts(&bytecode),
            initial_map = witness_map_to_ts(&initial),
            oracle_name = oracle_call_name,
            oracle_inputs = foreign_call_inputs_to_ts(&oracle_call_inputs),
            oracle_resp = foreign_call_response_to_ts(&oracle_response),
            expected_map = witness_map_to_ts(&result),
        );
        write(&output_path("foreign_call.ts"), &content);
    }

    // ── complex_foreign_call ─────────────────────────────────────────────────
    {
        let program = test_fixtures::complex_brillig_foreign_call_program();
        let bytecode = Program::serialize_program(&program);

        let initial: WitnessMap<FieldElement> = BTreeMap::from_iter([
            (Witness(1), FieldElement::from(1u128)),
            (Witness(2), FieldElement::from(2u128)),
            (Witness(3), FieldElement::from(3u128)),
        ])
        .into();

        let oracle_call_name = RefCell::new(String::new());
        let oracle_call_inputs = RefCell::new(Vec::new());
        let oracle_response = RefCell::new(Vec::new());

        let result = execute_simple(&program, initial.clone(), &backend, &|info| {
            assert_eq!(info.function, "complex");
            *oracle_call_name.borrow_mut() = info.function.clone();
            *oracle_call_inputs.borrow_mut() = info.inputs.clone();

            let arr = info.inputs[0].fields();
            let scalar = info.inputs[1].unwrap_field();
            let result_arr = vec![
                arr[0] * FieldElement::from(2u128),
                arr[1] * FieldElement::from(3u128),
                arr[2] * FieldElement::from(4u128),
            ];
            let response = ForeignCallResult {
                values: vec![
                    ForeignCallParam::Array(result_arr),
                    ForeignCallParam::Single(scalar),
                    ForeignCallParam::Single(scalar * FieldElement::from(2u128)),
                ],
            };
            *oracle_response.borrow_mut() = response.values.clone();
            response
        });

        let oracle_call_name = oracle_call_name.into_inner();
        let oracle_call_inputs = oracle_call_inputs.into_inner();
        let oracle_response = oracle_response.into_inner();

        let content = format!(
            "// This file was automatically generated by running `just generate-acvm-js-fixtures`
// See `complex_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = {bytecode};

export const initialWitnessMap = {initial_map};
export const expectedWitnessMap = {expected_map};

export const oracleCallName = '{oracle_name}';
export const oracleCallInputs = {oracle_inputs};
export const oracleResponse = {oracle_resp};
",
            bytecode = bytes_to_ts(&bytecode),
            initial_map = witness_map_to_ts(&initial),
            oracle_name = oracle_call_name,
            oracle_inputs = foreign_call_inputs_to_ts(&oracle_call_inputs),
            oracle_resp = foreign_call_response_to_ts(&oracle_response),
            expected_map = witness_map_to_ts(&result),
        );
        write(&output_path("complex_foreign_call.ts"), &content);
    }

    // ── memory_op ────────────────────────────────────────────────────────────
    {
        let program = test_fixtures::memory_op_program();
        let bytecode = Program::serialize_program(&program);

        // w1=1, w2=1 initialize the block; w3=2 is the value to write; w5=1 is the index.
        let initial: WitnessMap<FieldElement> = BTreeMap::from_iter([
            (Witness(1), FieldElement::from(1u128)),
            (Witness(2), FieldElement::from(1u128)),
            (Witness(3), FieldElement::from(2u128)),
            (Witness(5), FieldElement::from(1u128)),
        ])
        .into();

        let result = execute_simple(&program, initial.clone(), &backend, &no_oracles);

        let content = format!(
            "// This file was automatically generated by running `just generate-acvm-js-fixtures`
// See `memory_op_circuit` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = {bytecode};

export const initialWitnessMap = {initial_map};
export const expectedWitnessMap = {expected_map};
",
            bytecode = bytes_to_ts(&bytecode),
            initial_map = witness_map_to_ts(&initial),
            expected_map = witness_map_to_ts(&result),
        );
        write(&output_path("memory_op.ts"), &content);
    }

    // ── nested_acir_call ─────────────────────────────────────────────────────
    {
        let program = test_fixtures::nested_acir_call_program();
        let bytecode = Program::serialize_program(&program);

        // main: private w0=8, public w1=10
        let initial: WitnessMap<FieldElement> = BTreeMap::from_iter([
            (Witness(0), FieldElement::from(8u128)),
            (Witness(1), FieldElement::from(10u128)),
        ])
        .into();

        let mut witness_stack = WitnessStack::default();
        execute_function(&program, 0, initial.clone(), &backend, &no_oracles, &mut witness_stack);

        let compressed = witness_stack.serialize().expect("failed to serialize witness stack");

        // Drain stack into a Vec (pop() returns items last-in-first-out, so reverse).
        let mut stack_items = Vec::new();
        while let Some(item) = witness_stack.pop() {
            stack_items.push(item);
        }
        stack_items.reverse();

        // Build the TypeScript WitnessStack literal from the collected StackItems.
        let stack_items_ts: Vec<String> = stack_items
            .iter()
            .map(|item| {
                let entries: Vec<String> = item
                    .witness
                    .clone()
                    .into_iter()
                    .map(|(w, f)| format!("    [{}, {}]", w.0, field_to_hex(&f)))
                    .collect();
                format!(
                    "  {{ index: {}, witness: new Map([\n{}\n  ]) }}",
                    item.index,
                    entries.join(",\n")
                )
            })
            .collect();

        let content = format!(
            "// This file was automatically generated by running `just generate-acvm-js-fixtures`
// See `nested_acir_call_circuit` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = {bytecode};

export const initialWitnessMap = {initial_map};
export const expectedWitnessStack = [{stack_items}];
export const expectedCompressedWitnessStack = {compressed};
",
            bytecode = bytes_to_ts(&bytecode),
            initial_map = witness_map_to_ts(&initial),
            stack_items = stack_items_ts.join(",\n"),
            compressed = bytes_to_ts(&compressed),
        );
        write(&output_path("nested_acir_call.ts"), &content);
    }
}
