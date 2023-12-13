contract Blank {
    use dep::aztec::{
        abi,
        oracle::{
            get_public_key::get_public_key,
        },
    };

    #[aztec(private)]
    fn constructor() {}

    #[aztec(private)]
    fn getPublicKey(
        address: Field,
    ) -> [Field; 2]{
        let pub_key = get_public_key(address);
        
        [pub_key.x, pub_key.y]
    }
}
