//! Sha256 fallback function.
use super::uint32::UInt32;
use super::utils::{byte_decomposition, round_to_nearest_byte};
use crate::helpers::VariableStore;
use acir::{
    brillig,
    circuit::{
        brillig::{Brillig, BrilligInputs, BrilligOutputs},
        opcodes::{BlackBoxFuncCall, FunctionInput},
        Opcode,
    },
    native_types::{Expression, Witness},
    FieldElement,
};

const INIT_CONSTANTS: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

const ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

pub fn sha256(
    inputs: Vec<(Expression, u32)>,
    outputs: Vec<Witness>,
    mut num_witness: u32,
) -> (u32, Vec<Opcode>) {
    let mut new_opcodes = Vec::new();
    let mut new_inputs = Vec::new();
    let mut total_num_bytes = 0;

    // Decompose the input field elements into bytes and collect the resulting witnesses.
    for (witness, num_bits) in inputs {
        let num_bytes = round_to_nearest_byte(num_bits);
        total_num_bytes += num_bytes;
        let (extra_opcodes, extra_inputs, updated_witness_counter) =
            byte_decomposition(witness, num_bytes, num_witness);
        new_opcodes.extend(extra_opcodes);
        new_inputs.extend(extra_inputs);
        num_witness = updated_witness_counter;
    }

    let (result, num_witness, extra_opcodes) =
        create_sha256_constraint(new_inputs, total_num_bytes, num_witness);
    new_opcodes.extend(extra_opcodes);

    // constrain the outputs to be the same as the result of the circuit
    for i in 0..outputs.len() {
        let mut expr = Expression::from(outputs[i]);
        expr.push_addition_term(-FieldElement::one(), result[i]);
        new_opcodes.push(Opcode::Arithmetic(expr));
    }
    (num_witness, new_opcodes)
}

fn create_sha256_constraint(
    mut input: Vec<Witness>,
    total_num_bytes: u32,
    num_witness: u32,
) -> (Vec<Witness>, u32, Vec<Opcode>) {
    let mut new_opcodes = Vec::new();

    // pad the bytes according to sha256 padding rules
    let message_bits = total_num_bytes * 8;
    let (mut num_witness, pad_witness, extra_opcodes) = pad(128, 8, num_witness);
    new_opcodes.extend(extra_opcodes);
    input.push(pad_witness);
    let bytes_per_block = 64;
    let num_bytes = (input.len() + 8) as u32;
    let num_blocks = num_bytes / bytes_per_block + ((num_bytes % bytes_per_block != 0) as u32);
    let num_total_bytes = num_blocks * bytes_per_block;
    for _ in num_bytes..num_total_bytes {
        let (updated_witness_counter, pad_witness, extra_opcodes) = pad(0, 8, num_witness);
        num_witness = updated_witness_counter;
        new_opcodes.extend(extra_opcodes);
        input.push(pad_witness);
    }
    let (num_witness, pad_witness, extra_opcodes) = pad(message_bits, 64, num_witness);
    new_opcodes.extend(extra_opcodes);
    let (extra_opcodes, pad_witness, num_witness) =
        byte_decomposition(pad_witness.into(), 8, num_witness);
    new_opcodes.extend(extra_opcodes);
    input.extend(pad_witness);

    // turn witness into u32 and load sha256 state
    let (input, extra_opcodes, num_witness) = UInt32::from_witnesses(&input, num_witness);
    new_opcodes.extend(extra_opcodes);
    let (mut rolling_hash, extra_opcodes, num_witness) = prepare_state_constants(num_witness);
    new_opcodes.extend(extra_opcodes);
    let (round_constants, extra_opcodes, mut num_witness) = prepare_round_constants(num_witness);
    new_opcodes.extend(extra_opcodes);
    // split the input into blocks of size 16
    let input: Vec<Vec<UInt32>> = input.chunks(16).map(|block| block.to_vec()).collect();

    // process sha256 blocks
    for i in &input {
        let (new_rolling_hash, extra_opcodes, updated_witness_counter) =
            sha256_block(i, rolling_hash.clone(), round_constants.clone(), num_witness);
        new_opcodes.extend(extra_opcodes);
        num_witness = updated_witness_counter;
        rolling_hash = new_rolling_hash;
    }

    // decompose the result bytes in u32 to u8
    let (extra_opcodes, byte1, num_witness) =
        byte_decomposition(Expression::from(rolling_hash[0].inner), 4, num_witness);
    new_opcodes.extend(extra_opcodes);
    let (extra_opcodes, byte2, num_witness) =
        byte_decomposition(Expression::from(rolling_hash[1].inner), 4, num_witness);
    new_opcodes.extend(extra_opcodes);
    let (extra_opcodes, byte3, num_witness) =
        byte_decomposition(Expression::from(rolling_hash[2].inner), 4, num_witness);
    new_opcodes.extend(extra_opcodes);
    let (extra_opcodes, byte4, num_witness) =
        byte_decomposition(Expression::from(rolling_hash[3].inner), 4, num_witness);
    new_opcodes.extend(extra_opcodes);
    let (extra_opcodes, byte5, num_witness) =
        byte_decomposition(Expression::from(rolling_hash[4].inner), 4, num_witness);
    new_opcodes.extend(extra_opcodes);
    let (extra_opcodes, byte6, num_witness) =
        byte_decomposition(Expression::from(rolling_hash[5].inner), 4, num_witness);
    new_opcodes.extend(extra_opcodes);
    let (extra_opcodes, byte7, num_witness) =
        byte_decomposition(Expression::from(rolling_hash[6].inner), 4, num_witness);
    new_opcodes.extend(extra_opcodes);
    let (extra_opcodes, byte8, num_witness) =
        byte_decomposition(Expression::from(rolling_hash[7].inner), 4, num_witness);
    new_opcodes.extend(extra_opcodes);

    let result = vec![byte1, byte2, byte3, byte4, byte5, byte6, byte7, byte8]
        .into_iter()
        .flatten()
        .collect();

    (result, num_witness, new_opcodes)
}

pub(crate) fn pad(number: u32, bit_size: u32, mut num_witness: u32) -> (u32, Witness, Vec<Opcode>) {
    let mut new_opcodes = Vec::new();
    let mut variables = VariableStore::new(&mut num_witness);
    let pad = variables.new_variable();

    let brillig_opcode = Opcode::Brillig(Brillig {
        inputs: vec![BrilligInputs::Single(Expression {
            mul_terms: vec![],
            linear_combinations: vec![],
            q_c: FieldElement::from(number as u128),
        })],
        outputs: vec![BrilligOutputs::Simple(pad)],
        foreign_call_results: vec![],
        bytecode: vec![brillig::Opcode::Stop],
        predicate: None,
    });
    new_opcodes.push(brillig_opcode);

    let range = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
        input: FunctionInput { witness: pad, num_bits: bit_size },
    });
    new_opcodes.push(range);

    (num_witness, pad, new_opcodes)
}

fn sha256_block(
    input: &[UInt32],
    rolling_hash: Vec<UInt32>,
    round_constants: Vec<UInt32>,
    mut num_witness: u32,
) -> (Vec<UInt32>, Vec<Opcode>, u32) {
    let mut new_opcodes = Vec::new();
    let mut w = Vec::new();
    w.extend(input.to_owned());

    for i in 16..64 {
        // calculate s0 `w[i - 15].ror(7) ^ w[i - 15].ror(18) ^ (w[i - 15] >> 3)`
        let (a1, extra_opcodes, updated_witness_counter) = w[i - 15].ror(7, num_witness);
        new_opcodes.extend(extra_opcodes);
        let (a2, extra_opcodes, updated_witness_counter) =
            w[i - 15].ror(18, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (a3, extra_opcodes, updated_witness_counter) =
            w[i - 15].rightshift(3, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (a4, extra_opcodes, updated_witness_counter) = a1.xor(&a2, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (s0, extra_opcodes, updated_witness_counter) = a4.xor(&a3, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);

        // calculate s1 `w[i - 2].ror(17) ^ w[i - 2].ror(19) ^ (w[i - 2] >> 10)`
        let (b1, extra_opcodes, updated_witness_counter) =
            w[i - 2].ror(17, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (b2, extra_opcodes, updated_witness_counter) =
            w[i - 2].ror(19, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (b3, extra_opcodes, updated_witness_counter) =
            w[i - 2].rightshift(10, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (b4, extra_opcodes, updated_witness_counter) = b1.xor(&b2, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (s1, extra_opcodes, updated_witness_counter) = b4.xor(&b3, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);

        // calculate w[i] `w[i - 16] + w[i - 7] + s0 + s1`
        let (c1, extra_opcodes, updated_witness_counter) =
            w[i - 16].add(&w[i - 7], updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (c2, extra_opcodes, updated_witness_counter) = c1.add(&s0, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (c3, extra_opcodes, updated_witness_counter) = c2.add(&s1, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        w.push(c3);
        num_witness = updated_witness_counter;
    }

    let mut a = rolling_hash[0];
    let mut b = rolling_hash[1];
    let mut c = rolling_hash[2];
    let mut d = rolling_hash[3];
    let mut e = rolling_hash[4];
    let mut f = rolling_hash[5];
    let mut g = rolling_hash[6];
    let mut h = rolling_hash[7];

    #[allow(non_snake_case)]
    for i in 0..64 {
        // calculate S1 `e.ror(6) ^ e.ror(11) ^ e.ror(25)`
        let (a1, extra_opcodes, updated_witness_counter) = e.ror(6, num_witness);
        new_opcodes.extend(extra_opcodes);
        let (a2, extra_opcodes, updated_witness_counter) = e.ror(11, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (a3, extra_opcodes, updated_witness_counter) = e.ror(25, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (a4, extra_opcodes, updated_witness_counter) = a1.xor(&a2, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (S1, extra_opcodes, updated_witness_counter) = a4.xor(&a3, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);

        // calculate ch `(e & f) + (~e & g)`
        let (b1, extra_opcodes, updated_witness_counter) = e.and(&f, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (b2, extra_opcodes, updated_witness_counter) = e.not(updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (b3, extra_opcodes, updated_witness_counter) = b2.and(&g, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (ch, extra_opcodes, updated_witness_counter) = b1.add(&b3, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);

        // caculate temp1 `h + S1 + ch + round_constants[i] + w[i]`
        let (c1, extra_opcodes, updated_witness_counter) = h.add(&S1, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (c2, extra_opcodes, updated_witness_counter) = c1.add(&ch, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (c3, extra_opcodes, updated_witness_counter) =
            c2.add(&round_constants[i], updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (temp1, extra_opcodes, updated_witness_counter) =
            c3.add(&w[i], updated_witness_counter);
        new_opcodes.extend(extra_opcodes);

        // calculate S0 `a.ror(2) ^ a.ror(13) ^ a.ror(22)`
        let (d1, extra_opcodes, updated_witness_counter) = a.ror(2, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (d2, extra_opcodes, updated_witness_counter) = a.ror(13, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (d3, extra_opcodes, updated_witness_counter) = a.ror(22, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (d4, extra_opcodes, updated_witness_counter) = d1.xor(&d2, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (S0, extra_opcodes, updated_witness_counter) = d4.xor(&d3, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);

        // calculate T0 `b & c`
        let (T0, extra_opcodes, updated_witness_counter) = b.and(&c, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);

        // calculate maj `(a & (b + c - (T0 + T0))) + T0` which is the same as `(a & b) ^ (a & c) ^ (b & c)`
        let (e1, extra_opcodes, updated_witness_counter) = T0.add(&T0, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (e2, extra_opcodes, updated_witness_counter) = c.sub(&e1, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (e3, extra_opcodes, updated_witness_counter) = b.add(&e2, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (e4, extra_opcodes, updated_witness_counter) = a.and(&e3, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        let (maj, extra_opcodes, updated_witness_counter) = e4.add(&T0, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);

        // calculate temp2 `S0 + maj`
        let (temp2, extra_opcodes, updated_witness_counter) = S0.add(&maj, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);

        h = g;
        g = f;
        f = e;
        let (new_e, extra_opcodes, updated_witness_counter) =
            d.add(&temp1, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        d = c;
        c = b;
        b = a;
        let (new_a, extra_opcodes, updated_witness_counter) =
            temp1.add(&temp2, updated_witness_counter);
        new_opcodes.extend(extra_opcodes);
        num_witness = updated_witness_counter;
        a = new_a;
        e = new_e;
    }

    let mut output = Vec::new();
    let (output0, extra_opcodes, num_witness) = a.add(&rolling_hash[0], num_witness);
    new_opcodes.extend(extra_opcodes);
    let (output1, extra_opcodes, num_witness) = b.add(&rolling_hash[1], num_witness);
    new_opcodes.extend(extra_opcodes);
    let (output2, extra_opcodes, num_witness) = c.add(&rolling_hash[2], num_witness);
    new_opcodes.extend(extra_opcodes);
    let (output3, extra_opcodes, num_witness) = d.add(&rolling_hash[3], num_witness);
    new_opcodes.extend(extra_opcodes);
    let (output4, extra_opcodes, num_witness) = e.add(&rolling_hash[4], num_witness);
    new_opcodes.extend(extra_opcodes);
    let (output5, extra_opcodes, num_witness) = f.add(&rolling_hash[5], num_witness);
    new_opcodes.extend(extra_opcodes);
    let (output6, extra_opcodes, num_witness) = g.add(&rolling_hash[6], num_witness);
    new_opcodes.extend(extra_opcodes);
    let (output7, extra_opcodes, num_witness) = h.add(&rolling_hash[7], num_witness);
    new_opcodes.extend(extra_opcodes);

    output.push(output0);
    output.push(output1);
    output.push(output2);
    output.push(output3);
    output.push(output4);
    output.push(output5);
    output.push(output6);
    output.push(output7);

    (output, new_opcodes, num_witness)
}

/// Load initial state constants of Sha256
pub(crate) fn prepare_state_constants(mut num_witness: u32) -> (Vec<UInt32>, Vec<Opcode>, u32) {
    let mut new_opcodes = Vec::new();
    let mut new_witnesses = Vec::new();

    for i in INIT_CONSTANTS {
        let (new_witness, extra_opcodes, updated_witness_counter) =
            UInt32::load_constant(i, num_witness);
        new_opcodes.extend(extra_opcodes);
        new_witnesses.push(new_witness);
        num_witness = updated_witness_counter;
    }

    (new_witnesses, new_opcodes, num_witness)
}

/// Load round constants of Sha256
pub(crate) fn prepare_round_constants(mut num_witness: u32) -> (Vec<UInt32>, Vec<Opcode>, u32) {
    let mut new_opcodes = Vec::new();
    let mut new_witnesses = Vec::new();

    for i in ROUND_CONSTANTS {
        let (new_witness, extra_opcodes, updated_witness_counter) =
            UInt32::load_constant(i, num_witness);
        new_opcodes.extend(extra_opcodes);
        new_witnesses.push(new_witness);
        num_witness = updated_witness_counter;
    }

    (new_witnesses, new_opcodes, num_witness)
}
