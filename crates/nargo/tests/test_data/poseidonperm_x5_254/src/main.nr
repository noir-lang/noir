use dep::std::hash::poseidon;

fn main(x1: [Field; 3], y1: pub Field, x2: [Field; 5], y2: pub Field)
{
    let perm1 = poseidon::bn254::perm::x5_3(x1);
    constrain perm1[0] == y1;

    let perm2 = poseidon::bn254::perm::x5_5(x2);
    constrain perm2[0] == y2;
}
