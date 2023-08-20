use dep::std;

fn main(x: u64) {
	let two: u64 = 2;
	let three: u64 = 3;

	// shifts on constant values
	std::assert_eq(two << 2, 8);
	std::assert_eq((two << 3) / 8, two);
	std::assert_eq((three >> 1), 1);

	// shifts on runtime values
	std::assert_eq(x << 1, 128);
	std::assert_eq(x >> 2, 16);
}
