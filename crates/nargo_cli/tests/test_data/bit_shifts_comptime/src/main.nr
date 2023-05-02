fn main(x: u64) {
	let two: u64 = 2;
	let three: u64 = 3;

	// comptime shifts on comptime values
	assert(two << 2 == 8);
	assert((two << 3) / 8 == two);
	assert((three >> 1) == 1);

	// comptime shifts on runtime values
	assert(x << 1 == 128);
	assert(x >> 2 == 16);
}
