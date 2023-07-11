use dep::std::hash::poseidon;

fn main(x: [Field; 7])
{
    // Test optimised sponge
    let result = poseidon::bn254::sponge(x);
    
    assert(result == 0x080ae1669d62f0197190573d4a325bfb8d8fc201ce3127cbac0c47a7ac81ac48);
}
