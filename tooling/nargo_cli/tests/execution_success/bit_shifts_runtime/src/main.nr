fn main(x: u64, y: u64) {
	// runtime shifts on compile-time known values
	assert(64 << y == 128);
	assert(64 >> y == 32);

	// runtime shifts on runtime values
	assert(x << y == 128);
	assert(x >> y == 32);
}
