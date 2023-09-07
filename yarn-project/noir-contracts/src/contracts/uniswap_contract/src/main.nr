mod non_native_token_interface;

// Demonstrates how to send a message to a portal contract on L1. We use Uniswap here as it's the most typical example.
contract Uniswap {
    use dep::aztec::oracle::{
        public_call,
        context::get_portal_address
    };
    use dep::aztec::private_call_stack_item::PrivateCallStackItem;
    use dep::aztec::public_call_stack_item::PublicCallStackItem;
    use dep::aztec::types::point::Point;
    
    use crate::non_native_token_interface::NonNativeTokenPrivateContextInterface;
    
    #[aztec(private)]
    fn constructor() {}

    // What we need to make this nicer:
    // 1. A way to access other (and my own) contract's portal addresses (we need many additional args for no good reason now)
    // 2. don't pass all args manually to initial context
    // 3. Variable length arrays for input arguments (8 not enough)
    // 4. Should have storage for fixed values.
    // 5. account abstraction using its own "custom" constants instead of using shared constants...
    // 6. currently have to manually set the args array for calling other contracts which is inconvenient when using structs etc.

    #[aztec(private)]
    fn swap(
        inputAsset: Field, 
        inputAmount: Field,
        uniswapFeeTier: Field, // which uniswap tier to use (eg 3000 for 0.3% fee)
        outputAsset: Field,
        minimumOutputAmount: Field, // minimum output amount to receive (slippage protection for the swap)
        sender: Field, 
        recipient: Field, // receiver address of output asset after the swap
        secretHash: Field, // for when l1 uniswap portal inserts the message to consume output assets on L2
        deadlineForL1ToL2Message: Field, // for when l1 uniswap portal inserts the message to consume output assets on L2
        cancellerForL1ToL2Message: Field, // L1 address of who can cancel the message to consume assets on L2.
        callerOnL1: Field, // ethereum address that can call this function on the L1 portal (0x0 if anyone can call)
    ) -> Field {
        // Get portal addresses
        let l1UniswapPortal = context.this_portal_address();
        let inputAssetPortalAddress = get_portal_address(inputAsset);
        let outputAssetPortalAddress = get_portal_address(outputAsset);

        // inputAsset.withdraw(inputAmount, sender, recipient=l1UniswapPortal, callerOnL1=l1UniswapPortal) 
        // only uniswap portal can call this (done to safeguard ordering of message consumption)
        // ref: https://docs.aztec.network/aztec/how-it-works/l1-l2-messaging#designated-caller
        let inputAssetInterface = NonNativeTokenPrivateContextInterface::at(inputAsset);
        let return_value = inputAssetInterface.withdraw(
            &mut context,
            inputAmount,
            sender,
            l1UniswapPortal,
            l1UniswapPortal,
        )[0];
        
        // Send the swap message to L1 portal
        let content_hash = _compute_swap_content_hash(
           inputAssetPortalAddress,
           inputAmount,
           uniswapFeeTier,
           outputAssetPortalAddress,
           minimumOutputAmount,
           recipient,
           secretHash,
           deadlineForL1ToL2Message,
           cancellerForL1ToL2Message,
           callerOnL1,
        );
        context.message_portal(content_hash);

        return_value
    }

    // refer `l1-contracts/test/portals/UniswapPortal.sol` on how L2 to L1 message is expected
    fn _compute_swap_content_hash(
        inputTokenPortal: Field,
        inAmount: Field,
        uniswapFeeTier: Field,
        outputTokenPortal: Field,
        amountOutMin: Field,
        aztecRecipientAddress: Field,
        secretHash: Field,
        deadlineForL1ToL2Message: Field,
        canceller: Field,
        callerOnL1: Field, // ethereum address that can call this function on the L1 portal (0x0 if anyone can call)
    ) -> pub Field {

        let mut hash_bytes: [u8; 324] = [0; 324]; // 10 fields of 32 bytes each + 4 bytes fn selector

        let inputTokenPortal_bytes = inputTokenPortal.to_be_bytes(32);
        let inAmount_bytes = inAmount.to_be_bytes(32);
        let uniswapFeeTier_bytes = uniswapFeeTier.to_be_bytes(32);
        let outputTokenPortal_bytes = outputTokenPortal.to_be_bytes(32);
        let amountOutMin_bytes = amountOutMin.to_be_bytes(32);
        let aztecRecipientAddress_bytes = aztecRecipientAddress.to_be_bytes(32);
        let secretHash_bytes = secretHash.to_be_bytes(32);
        let deadlineForL1ToL2Message_bytes = deadlineForL1ToL2Message.to_be_bytes(32);
        let canceller_bytes = canceller.to_be_bytes(32);
        let callerOnL1_bytes = callerOnL1.to_be_bytes(32);

        // function selector: 0x9c073c81 keccak256("swap(address,uint256,uint24,address,uint256,bytes32,bytes32,uint32,address,address)")
        hash_bytes[0] = 0x9c;
        hash_bytes[1] = 0x07;
        hash_bytes[2] = 0x3c;
        hash_bytes[3] = 0x81;

        for i in 0..32 {
            hash_bytes[i + 4] = inputTokenPortal_bytes[i];
            hash_bytes[i + 36] = inAmount_bytes[i];
            hash_bytes[i + 68] = uniswapFeeTier_bytes[i];
            hash_bytes[i + 100] = outputTokenPortal_bytes[i];
            hash_bytes[i + 132] = amountOutMin_bytes[i];
            hash_bytes[i + 164] = aztecRecipientAddress_bytes[i];
            hash_bytes[i + 196] = secretHash_bytes[i];
            hash_bytes[i + 228] = deadlineForL1ToL2Message_bytes[i];
            hash_bytes[i + 260] = canceller_bytes[i];
            hash_bytes[i + 292] = callerOnL1_bytes[i];
        }

        let content_sha256 = dep::std::hash::sha256(hash_bytes);

        // // Convert the content_sha256 to a field element
        let mut v = 1;
        let mut high = 0 as Field;
        let mut low = 0 as Field;

        for i in 0..16 {
            high = high + (content_sha256[15 - i] as Field) * v;
            low = low + (content_sha256[16 + 15 - i] as Field) * v;
            v = v * 256;
        }

        // Abuse that a % p + b % p = (a + b) % p and that low < p
        let content_hash = low + high * v;

        content_hash
    }
}