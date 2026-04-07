// Test that type aliases to primitive types can use path-syntax method calls.
type MyField = Field;

// Chained alias
type AliasOfAlias = MyField;

fn main(x: Field) {
    let expected: [u1; 8] = x.to_le_bits();

    // Direct method on Field alias gives same result as dot syntax
    let bits: [u1; 8] = MyField::to_le_bits(x);
    assert(bits == expected);

    // Chained alias gives same result as dot syntax
    let bits2: [u1; 8] = AliasOfAlias::to_le_bits(x);
    assert(bits2 == expected);
}
