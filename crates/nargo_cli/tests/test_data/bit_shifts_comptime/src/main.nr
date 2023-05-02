fn main(x: u64) {
	let two: u64 = 2;
	let three: u64 = 3;

	// comptime shifts on comptime values
	constrain two << 2 == 8;
	constrain (two << 3) / 8 == two;
	constrain (three >> 1) == 1;

	// comptime shifts on runtime values
	constrain x << 1 == 128;
	constrain x >> 2 == 16;
}
