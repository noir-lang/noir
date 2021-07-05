mod test1;
mod test2;

use dep::std;


// msg is the email and shipping address combined

fn main(y : Field, x : Field, answer1 : pub Field, answer2 :pub Field, answer3 : pub [32]u8, msg : [200]u8){    

    // Question 1, is about operators and number types see the docs
    //
    priv k = test1::run(x,y);
    constrain k == answer1;

    constrain x != y;
    constrain x != 0;


    // Question 2, is about loops
    //
    test2::run(x, y, answer2);

    // Question 3 is about private inputs and hashing.
    //
    // To generate a valid `msg` concatenate your email and shipping address eg `email shipping` 
    // Convert this to an ASCII Hex string and pad the start with 0's so the length is 400.
    // Convert this to a Uint8Array (Length 200)
    constrain answer3 == std::hash::sha256(msg);
}


