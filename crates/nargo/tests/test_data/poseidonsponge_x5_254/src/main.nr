use dep::std::hash::poseidon;

fn main(x: [Field; 7]) -> pub Field
{
    let perm = poseidon::bn254::sponge(x);
 
    perm
}
