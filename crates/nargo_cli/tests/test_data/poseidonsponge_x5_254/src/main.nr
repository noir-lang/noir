use dep::std::hash::poseidon;

fn main(x: [Field; 7])
{
    // Test optimised sponge
    let result = poseidon::bn254::sponge(x);
    
    constrain result == 0x080ae1669d62f0197190573d4a325bfb8d8fc201ce3127cbac0c47a7ac81ac48;

    // Test unoptimised sponge
//    let result2 = poseidon::absorb(poseidon::bn254::consts::x5_5_config(), [0;5], 4, 1, x)[1];

 //   constrain result2 == result;
}
