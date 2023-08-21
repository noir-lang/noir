fn main(x: u64, y: u64) {
	// runtime shifts on compile-time known values
	assert_eq(64 << y, 128);
	assert_eq(64 >> y, 32);

	// runtime shifts on runtime values
	assert_eq(x << y, 128);
	assert_eq(x >> y, 32);
}
