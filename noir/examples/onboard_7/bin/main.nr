use dep::std;

fn main(x : [5]u8, low : Public, high: Public) {

     let digest = std::hash::blake2s(x);
     
     constrain digest[0] == low
     constrain digest[1] == high
}