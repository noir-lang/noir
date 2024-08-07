// main.nr
mod factor;

fn main(number: pub u32) {
    assert(number != 0, "The given number should not be zero.");
    factor::print_factors(number);
}
