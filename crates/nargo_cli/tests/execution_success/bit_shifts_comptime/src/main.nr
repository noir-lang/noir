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

	// Fails as expected now
	// regression_2250();
}

fn regression_2250() {
	let a: u1 = 1 >> 1;
	assert(a == 0);
	
	let b: u32 = 1 >> 32;
	assert(b == 0);
}