fn main(x: u128, y: u128) -> pub u128 {
    // Multiplying 257295058452732708167448229940904157557 * 85070591730234615865843651857942052864
    // using bn254 field elements wraps the maximum field element value,
    // resulting in 31631953497925087476338759149270597631 which is less than u128::MAX.
    // We want this to be caught as an overflow.
    x * y
}
