fn main(x: u64) {
	let two: u64 = 2;
	let three: u64 = 3;

	// shifts on constant values
	assert(two << 2 == 8);
	assert((two << 3) / 8 == two);
	assert((three >> 1) == 1);

	// shifts on runtime values
	assert(x << 1 == 128);
	assert(x >> 2 == 16);

	regression_2250();
	regression_2399();
}

fn regression_2250() {
	let a: u1 = 1 >> 1;
	assert(a == 0);
	
	let b: u32 = 1 >> 32;
	assert(b == 0);
}

fn regression_2399() {
    let result_0 = bitshift_variable(0);
    assert(result_0 == 1);

    let result_4 = bitshift_variable(4);
    assert(result_4 == 16);
}

fn bitshift_variable(idx: u64) -> u64 {
    let mut bits: u64 = 0;
    bits |= 1 << idx;

    bits
}