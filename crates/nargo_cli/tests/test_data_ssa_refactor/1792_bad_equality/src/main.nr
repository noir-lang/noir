fn main() {
    let randomness = dep::std::hash::pedersen_with_separator([0], 0)[0];
    let note0_is_real = ((randomness as u32) % 2) as bool;

    let mut note0_value = 0;
    if note0_is_real { 
        note0_value = randomness + 1;
    } else {
        note0_value = 0;
    }
    
    assert(note0_is_real == true);
}
