fn main(x: u64) {
	let two: u64 = 2;
	let three: u64 = 3;

	// shifts on constant values
	assert_eq(two << 2, 8);
	assert_eq((two << 3) / 8, two);
	assert_eq((three >> 1), 1);

	// shifts on runtime values
	assert_eq(x << 1, 128);
	assert_eq(x >> 2, 16);
}
