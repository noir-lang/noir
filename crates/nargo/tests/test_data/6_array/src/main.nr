use dep::std;

fn main(x: Field, y: Field, z: Field) {
    let _a = x+y+z;
    let mut h1 = std::hash::mimc_bn254([x,y]) as u32;
    for i in 0..5 {
        if x as u32 +1000<5 {
            h1 = std::hash::mimc_bn254([x+i,y]) as u32;
        } else {
            h1 = std::hash::mimc_bn254([x+500,y]) as u32;
        }
    };
    constrain h1 > 1;
}
