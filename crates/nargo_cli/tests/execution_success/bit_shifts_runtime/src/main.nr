use dep::std;

fn main(x: u64, y: u64) {
	// runtime shifts on compile-time known values
	std::assert_eq(64 << y, 128);
	std::assert_eq(64 >> y, 32);

	// runtime shifts on runtime values
	std::assert_eq(x << y, 128);
	std::assert_eq(x >> y, 32);
}
