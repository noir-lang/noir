use dep::std;

fn main() {
    let commitment = std::hash::pedersen_commitment([0, 1]);
    assert_eq(commitment.x, 0x054aa86a73cb8a34525e5bbed6e43ba1198e860f5f3950268f71df4591bde402);
    assert_eq(commitment.y, 0x209dcfbf2cfb57f9f6046f44d71ac6faf87254afc7407c04eb621a6287cac126);
}

