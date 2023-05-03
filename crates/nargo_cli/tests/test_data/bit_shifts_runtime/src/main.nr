fn main(x: u64, y: u64) {
	// These are currently unimplemented and panic with "ShiftLeft and ShiftRight operations with shifts which are only known at runtime are not yet implemented."
	// See: https://github.com/noir-lang/noir/issues/1265
		
	// runtime shifts on comptime values
	assert(64 << y == 128);
	assert(64 >> y == 32);

	// runtime shifts on runtime values
	assert(x << y == 128);
	assert(x >> y == 32);
}
