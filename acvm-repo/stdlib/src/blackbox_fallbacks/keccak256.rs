//! Keccak256 fallback function.
use super::{
    sha256::pad,
    uint8::UInt8,
    utils::{byte_decomposition, round_to_nearest_byte},
    UInt64,
};
use acir::{
    circuit::Opcode,
    native_types::{Expression, Witness},
    FieldElement,
};

const STATE_NUM_BYTES: usize = 200;
const BITS: usize = 256;
const WORD_SIZE: usize = 8;
const BLOCK_SIZE: usize = (1600 - BITS * 2) / WORD_SIZE;
const ROUND_CONSTANTS: [u64; 24] = [
    1,
    0x8082,
    0x800000000000808a,
    0x8000000080008000,
    0x808b,
    0x80000001,
    0x8000000080008081,
    0x8000000000008009,
    0x8a,
    0x88,
    0x80008009,
    0x8000000a,
    0x8000808b,
    0x800000000000008b,
    0x8000000000008089,
    0x8000000000008003,
    0x8000000000008002,
    0x8000000000000080,
    0x800a,
    0x800000008000000a,
    0x8000000080008081,
    0x8000000000008080,
    0x80000001,
    0x8000000080008008,
];
const RHO: [u32; 24] =
    [1, 3, 6, 10, 15, 21, 28, 36, 45, 55, 2, 14, 27, 41, 56, 8, 25, 43, 62, 18, 39, 61, 20, 44];
const PI: [usize; 24] =
    [10, 7, 11, 17, 18, 3, 5, 16, 8, 21, 24, 4, 15, 23, 19, 13, 12, 2, 20, 14, 22, 9, 6, 1];

pub fn keccak256(
    inputs: Vec<(Expression, u32)>,
    outputs: Vec<Witness>,
    mut num_witness: u32,
) -> (u32, Vec<Opcode>) {
    let mut new_opcodes = Vec::new();
    let mut new_inputs = Vec::new();

    // Decompose the input field elements into bytes and collect the resulting witnesses.
    for (witness, num_bits) in inputs {
        let num_bytes = round_to_nearest_byte(num_bits);
        let (extra_opcodes, extra_inputs, updated_witness_counter) =
            byte_decomposition(witness, num_bytes, num_witness);
        new_opcodes.extend(extra_opcodes);
        new_inputs.extend(extra_inputs);
        num_witness = updated_witness_counter;
    }

    let (result, num_witness, extra_opcodes) = create_keccak_constraint(new_inputs, num_witness);
    new_opcodes.extend(extra_opcodes);

    // constrain the outputs to be the same as the result of the circuit
    for i in 0..outputs.len() {
        let mut expr = Expression::from(outputs[i]);
        expr.push_addition_term(-FieldElement::one(), result[i]);
        new_opcodes.push(Opcode::Arithmetic(expr));
    }
    (num_witness, new_opcodes)
}

fn create_keccak_constraint(
    input: Vec<Witness>,
    num_witness: u32,
) -> (Vec<Witness>, u32, Vec<Opcode>) {
    let mut new_opcodes = Vec::new();
    let num_blocks = input.len() / BLOCK_SIZE + 1;

    // pad keccak
    let (input, extra_opcodes, mut num_witness) = pad_keccak(input, num_blocks, num_witness);
    new_opcodes.extend(extra_opcodes);

    // prepare state
    let mut state = Vec::with_capacity(200);
    for _ in 0..STATE_NUM_BYTES {
        let (zero, extra_opcodes, updated_witness_counter) = UInt8::load_constant(0, num_witness);
        new_opcodes.extend(extra_opcodes);
        state.push(zero);
        num_witness = updated_witness_counter;
    }

    // process block
    for i in 0..num_blocks {
        for j in 0..BLOCK_SIZE {
            let (new_state, extra_opcodes, updated_witness_counter) =
                state[j].xor(&UInt8::new(input[i * BLOCK_SIZE + j]), num_witness);
            new_opcodes.extend(extra_opcodes);
            state[j] = new_state;
            num_witness = updated_witness_counter;
        }
        let (new_state, extra_opcodes, updated_witness_counter) = keccakf(state, num_witness);
        new_opcodes.extend(extra_opcodes);
        num_witness = updated_witness_counter;
        state = new_state;
    }

    let result: Vec<Witness> = state[..32].iter().map(|x| x.inner).collect();
    (result, num_witness, new_opcodes)
}

fn keccakf(state: Vec<UInt8>, num_witness: u32) -> (Vec<UInt8>, Vec<Opcode>, u32) {
    let mut new_opcodes = Vec::new();

    // turn state into UInt64
    let mut state_witnesses: Vec<Witness> = Vec::new();
    for i in 0..state.len() / 8 {
        for j in 0..8 {
            state_witnesses.push(state[i * 8 + (7 - j)].inner);
        }
    }
    let (mut state_u64, extra_opcodes, mut num_witness) =
        UInt64::from_witnesses(&state_witnesses, num_witness);
    new_opcodes.extend(extra_opcodes);

    // process round
    for round_constant in ROUND_CONSTANTS {
        let (new_state_u64, extra_opcodes, updated_witness_counter) =
            keccak_round(state_u64, round_constant, num_witness);
        state_u64 = new_state_u64;
        new_opcodes.extend(extra_opcodes);
        num_witness = updated_witness_counter;
    }

    // turn state back to UInt8
    let state_u64_witnesses: Vec<Witness> = state_u64.into_iter().map(|x| x.inner).collect();
    let mut state_u8 = Vec::with_capacity(state_u64_witnesses.len());
    for state_u64_witness in state_u64_witnesses {
        let (extra_opcodes, mut u8s, updated_witness_counter) =
            byte_decomposition(Expression::from(state_u64_witness), 8, num_witness);
        new_opcodes.extend(extra_opcodes);
        u8s.reverse();
        state_u8.push(u8s);
        num_witness = updated_witness_counter;
    }

    let state_u8: Vec<UInt8> = state_u8.into_iter().flatten().map(UInt8::new).collect();
    (state_u8, new_opcodes, num_witness)
}

fn keccak_round(
    mut a: Vec<UInt64>,
    round_const: u64,
    mut num_witness: u32,
) -> (Vec<UInt64>, Vec<Opcode>, u32) {
    let mut new_opcodes = Vec::new();

    // theta
    let mut array = Vec::with_capacity(5);
    for _ in 0..5 {
        let (zero, extra_opcodes, updated_witness_counter) = UInt64::load_constant(0, num_witness);
        array.push(zero);
        new_opcodes.extend(extra_opcodes);
        num_witness = updated_witness_counter;
    }
    for x in 0..5 {
        for y_count in 0..5 {
            let y = y_count * 5;
            let (new_array_ele, extra_opcodes, updated_witness_counter) =
                array[x].xor(&a[x + y], num_witness);
            new_opcodes.extend(extra_opcodes);
            num_witness = updated_witness_counter;
            array[x] = new_array_ele;
        }
    }
    for x in 0..5 {
        for y_count in 0..5 {
            let y = y_count * 5;
            let (a_ele, extra_opcodes, updated_witness_counter) =
                array[(x + 1) % 5].rol(1, num_witness);
            new_opcodes.extend(extra_opcodes);
            let (b_ele, extra_opcodes, updated_witness_counter) =
                array[(x + 4) % 5].xor(&a_ele, updated_witness_counter);
            new_opcodes.extend(extra_opcodes);
            let (new_array_ele, extra_opcodes, updated_witness_counter) =
                a[x + y].xor(&b_ele, updated_witness_counter);
            new_opcodes.extend(extra_opcodes);
            num_witness = updated_witness_counter;
            a[x + y] = new_array_ele;
        }
    }

    // rho and pi
    let mut last = a[1];
    for x in 0..24 {
        array[0] = a[PI[x]];
        let (a_ele, extra_opcodes, updated_witness_counter) = last.rol(RHO[x], num_witness);
        new_opcodes.extend(extra_opcodes);
        a[PI[x]] = a_ele;
        num_witness = updated_witness_counter;
        last = array[0];
    }

    // chi
    for y_step in 0..5 {
        let y = y_step * 5;

        array[..5].copy_from_slice(&a[y..(5 + y)]);

        for x in 0..5 {
            let (a_ele, extra_opcodes, updated_witness_counter) =
                array[(x + 1) % 5].not(num_witness);
            new_opcodes.extend(extra_opcodes);
            let (b_ele, extra_opcodes, updated_witness_counter) =
                a_ele.and(&array[(x + 2) % 5], updated_witness_counter);
            new_opcodes.extend(extra_opcodes);
            let (c_ele, extra_opcodes, updated_witness_counter) =
                array[x].xor(&b_ele, updated_witness_counter);
            new_opcodes.extend(extra_opcodes);

            a[y + x] = c_ele;
            num_witness = updated_witness_counter;
        }
    }

    // iota
    let (rc, extra_opcodes, num_witness) = UInt64::load_constant(round_const, num_witness);
    new_opcodes.extend(extra_opcodes);
    let (a_ele, extra_opcodes, num_witness) = a[0].xor(&rc, num_witness);
    new_opcodes.extend(extra_opcodes);
    a[0] = a_ele;

    (a, new_opcodes, num_witness)
}

fn pad_keccak(
    mut input: Vec<Witness>,
    num_blocks: usize,
    num_witness: u32,
) -> (Vec<Witness>, Vec<Opcode>, u32) {
    let mut new_opcodes = Vec::new();
    let total_len = BLOCK_SIZE * num_blocks;

    let (mut num_witness, pad_witness, extra_opcodes) = pad(0x01, 8, num_witness);

    new_opcodes.extend(extra_opcodes);
    input.push(pad_witness);
    for _ in 0..total_len - input.len() {
        let (updated_witness_counter, pad_witness, extra_opcodes) = pad(0x00, 8, num_witness);
        new_opcodes.extend(extra_opcodes);
        input.push(pad_witness);
        num_witness = updated_witness_counter;
    }

    let (zero_x_80, extra_opcodes, num_witness) = UInt8::load_constant(0x80, num_witness);
    new_opcodes.extend(extra_opcodes);
    let (final_pad, extra_opcodes, num_witness) =
        UInt8::new(input[total_len - 1]).xor(&zero_x_80, num_witness);
    new_opcodes.extend(extra_opcodes);
    input[total_len - 1] = final_pad.inner;

    (input, new_opcodes, num_witness)
}
