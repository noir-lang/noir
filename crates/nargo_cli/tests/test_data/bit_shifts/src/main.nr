fn main(x: u64, y: u64) {
	let two: u64 = 2;
	let three: u64 = 3;

	// comptime shifts on comptime values
	constrain two << 2 == 8;
	constrain (two << 3) / 8 == two;
	constrain (three >> 1) == 1;

	// comptime shifts on runtime values
	constrain x << 1 == 128;
	constrain x >> 2 == 16;

	// runtime shifts on comptime values
	// These are currently unimplemented and panic with "ShiftLeft and ShiftRight operations with shifts which are only known at runtime are not yet implemented."
	// See: https://github.com/noir-lang/noir/issues/1265
	// constrain 64 << y == 128;
	// constrain 64 >> y == 32;

	// runtime shifts on runtime values
	// These are currently unimplemented and panic with "ShiftLeft and ShiftRight operations with shifts which are only known at runtime are not yet implemented."
	// See: https://github.com/noir-lang/noir/issues/1265
	// constrain x << y == 128;
	// constrain x >> y == 32;
	constrain y == 1;
}
