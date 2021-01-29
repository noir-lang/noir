   
   // Here is an example of doing merkle tree stuff :)
   // description to be added later

    // To create a smart contract for this Noir program, type `cargo run contract` 
   fn main(x : Witness,low : Public, _high: Public) {

       // We use the `as` keyword here to denote the fact that we want to take just the first byte from the x Witness
       // The padding is taken care of by the program
        let digest = std::hash::sha256([x as u8]);
        
        constrain digest[0] == low;
   }