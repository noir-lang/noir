// Demonstrates how to send a message to a portal contract on L1. We use Uniswap here as it's the most typical example.
contract Uniswap {
    use dep::aztec::abi;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::abi::PublicContextInputs;
    use dep::aztec::context::Context;
    use dep::aztec::oracle::public_call;
    use dep::aztec::private_call_stack_item::PrivateCallStackItem;
    use dep::aztec::public_call_stack_item::PublicCallStackItem;
    use dep::aztec::types::point::Point;
    
    fn constructor(
        inputs: PrivateContextInputs,
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        Context::new(inputs, 0).finish()
    }

    // What we need to make this nicer:
    // 1. A way to access other (and my own) contract's portal addresses (we need many additional args for no good reason now)
    // 2. don't pass all args manually to initial context
    // 3. Variable length arrays for input arguments (8 not enough)
    // 4. Should have storage for fixed values.
    // 5. account abstraction using its own "custom" constants instead of using shared constants...
    // 6. currently have to manually set the args array for calling other contracts which is inconvenient when using structs etc.

    fn swap(
        inputs: PrivateContextInputs,
        withdrawFnSelector: Field, // withdraw method on inputAsset (l2 contract) that would withdraw to L1
        inputAsset: Field, 
        inputAssetPortalAddress: Field, // l1 portal of input asset
        inputAmount: Field,
        uniswapFeeTier: Field, // which uniswap tier to use (eg 3000 for 0.3% fee)
        outputAsset: Field,
        outputAssetPortalAddress: Field, // l1 portal of output asset
        minimumOutputAmount: Field, // minimum output amount to receive (slippage protection for the swap)
        sender: Point, 
        recipient: Field, // receiver address of output asset after the swap
        secretHash: Field, // for when l1 uniswap portal inserts the message to consume output assets on L2
        deadlineForL1ToL2Message: Field, // for when l1 uniswap portal inserts the message to consume output assets on L2
        cancellerForL1ToL2Message: Field, // L1 address of who can cancel the message to consume assets on L2.
        l1UniswapPortal: Field, // L1 address of uniswap portal contract
        callerOnL1: Field, // ethereum address that can call this function on the L1 portal (0x0 if anyone can call)
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut initialContext = Context::new(inputs, abi::hash_args([
            withdrawFnSelector,
            inputAsset,
            inputAssetPortalAddress,
            inputAmount,
            uniswapFeeTier,
            outputAsset,
            outputAssetPortalAddress,
            minimumOutputAmount,
            sender.x,
            sender.y,
            recipient,
            secretHash,
            deadlineForL1ToL2Message,
            cancellerForL1ToL2Message,
            l1UniswapPortal,
            callerOnL1,
        ]));

        // inputAsset.withdraw(inputAmount, sender, recipient=l1UniswapPortal, callerOnL1=l1UniswapPortal) 
        // only uniswap portal can call this (done to safeguard ordering of message consumption)
        // ref: https://docs.aztec.network/aztec/how-it-works/l1-l2-messaging#designated-caller
        let (callStackItem, mut context) = PrivateCallStackItem::call(inputAsset, withdrawFnSelector, [
            inputAmount,
            sender.x,
            sender.y,
            l1UniswapPortal,
            l1UniswapPortal,
        ], initialContext);
        
        let result = callStackItem.public_inputs.return_values[0];
        context.return_values = context.return_values.push(result);

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
        context = context.message_portal(content_hash);

        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        context.finish()
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
        hash_bytes[4] = inputTokenPortal_bytes[0];
        hash_bytes[5] = inputTokenPortal_bytes[1];
        hash_bytes[6] = inputTokenPortal_bytes[2];
        hash_bytes[7] = inputTokenPortal_bytes[3];
        hash_bytes[8] = inputTokenPortal_bytes[4];
        hash_bytes[9] = inputTokenPortal_bytes[5];
        hash_bytes[10] = inputTokenPortal_bytes[6];
        hash_bytes[11] = inputTokenPortal_bytes[7];
        hash_bytes[12] = inputTokenPortal_bytes[8];
        hash_bytes[13] = inputTokenPortal_bytes[9];
        hash_bytes[14] = inputTokenPortal_bytes[10];
        hash_bytes[15] = inputTokenPortal_bytes[11];
        hash_bytes[16] = inputTokenPortal_bytes[12];
        hash_bytes[17] = inputTokenPortal_bytes[13];
        hash_bytes[18] = inputTokenPortal_bytes[14];
        hash_bytes[19] = inputTokenPortal_bytes[15];
        hash_bytes[20] = inputTokenPortal_bytes[16];
        hash_bytes[21] = inputTokenPortal_bytes[17];
        hash_bytes[22] = inputTokenPortal_bytes[18];
        hash_bytes[23] = inputTokenPortal_bytes[19];
        hash_bytes[24] = inputTokenPortal_bytes[20];
        hash_bytes[25] = inputTokenPortal_bytes[21];
        hash_bytes[26] = inputTokenPortal_bytes[22];
        hash_bytes[27] = inputTokenPortal_bytes[23];
        hash_bytes[28] = inputTokenPortal_bytes[24];
        hash_bytes[29] = inputTokenPortal_bytes[25];
        hash_bytes[30] = inputTokenPortal_bytes[26];
        hash_bytes[31] = inputTokenPortal_bytes[27];
        hash_bytes[32] = inputTokenPortal_bytes[28];
        hash_bytes[33] = inputTokenPortal_bytes[29];
        hash_bytes[34] = inputTokenPortal_bytes[30];
        hash_bytes[35] = inputTokenPortal_bytes[31];
        hash_bytes[36] = inAmount_bytes[0];
        hash_bytes[37] = inAmount_bytes[1];
        hash_bytes[38] = inAmount_bytes[2];
        hash_bytes[39] = inAmount_bytes[3];
        hash_bytes[40] = inAmount_bytes[4];
        hash_bytes[41] = inAmount_bytes[5];
        hash_bytes[42] = inAmount_bytes[6];
        hash_bytes[43] = inAmount_bytes[7];
        hash_bytes[44] = inAmount_bytes[8];
        hash_bytes[45] = inAmount_bytes[9];
        hash_bytes[46] = inAmount_bytes[10];
        hash_bytes[47] = inAmount_bytes[11];
        hash_bytes[48] = inAmount_bytes[12];
        hash_bytes[49] = inAmount_bytes[13];
        hash_bytes[50] = inAmount_bytes[14];
        hash_bytes[51] = inAmount_bytes[15];
        hash_bytes[52] = inAmount_bytes[16];
        hash_bytes[53] = inAmount_bytes[17];
        hash_bytes[54] = inAmount_bytes[18];
        hash_bytes[55] = inAmount_bytes[19];
        hash_bytes[56] = inAmount_bytes[20];
        hash_bytes[57] = inAmount_bytes[21];
        hash_bytes[58] = inAmount_bytes[22];
        hash_bytes[59] = inAmount_bytes[23];
        hash_bytes[60] = inAmount_bytes[24];
        hash_bytes[61] = inAmount_bytes[25];
        hash_bytes[62] = inAmount_bytes[26];
        hash_bytes[63] = inAmount_bytes[27];
        hash_bytes[64] = inAmount_bytes[28];
        hash_bytes[65] = inAmount_bytes[29];
        hash_bytes[66] = inAmount_bytes[30];
        hash_bytes[67] = inAmount_bytes[31];
        hash_bytes[68] = uniswapFeeTier_bytes[0];
        hash_bytes[69] = uniswapFeeTier_bytes[1];
        hash_bytes[70] = uniswapFeeTier_bytes[2];
        hash_bytes[71] = uniswapFeeTier_bytes[3];
        hash_bytes[72] = uniswapFeeTier_bytes[4];
        hash_bytes[73] = uniswapFeeTier_bytes[5];
        hash_bytes[74] = uniswapFeeTier_bytes[6];
        hash_bytes[75] = uniswapFeeTier_bytes[7];
        hash_bytes[76] = uniswapFeeTier_bytes[8];
        hash_bytes[77] = uniswapFeeTier_bytes[9];
        hash_bytes[78] = uniswapFeeTier_bytes[10];
        hash_bytes[79] = uniswapFeeTier_bytes[11];
        hash_bytes[80] = uniswapFeeTier_bytes[12];
        hash_bytes[81] = uniswapFeeTier_bytes[13];
        hash_bytes[82] = uniswapFeeTier_bytes[14];
        hash_bytes[83] = uniswapFeeTier_bytes[15];
        hash_bytes[84] = uniswapFeeTier_bytes[16];
        hash_bytes[85] = uniswapFeeTier_bytes[17];
        hash_bytes[86] = uniswapFeeTier_bytes[18];
        hash_bytes[87] = uniswapFeeTier_bytes[19];
        hash_bytes[88] = uniswapFeeTier_bytes[20];
        hash_bytes[89] = uniswapFeeTier_bytes[21];
        hash_bytes[90] = uniswapFeeTier_bytes[22];
        hash_bytes[91] = uniswapFeeTier_bytes[23];
        hash_bytes[92] = uniswapFeeTier_bytes[24];
        hash_bytes[93] = uniswapFeeTier_bytes[25];
        hash_bytes[94] = uniswapFeeTier_bytes[26];
        hash_bytes[95] = uniswapFeeTier_bytes[27];
        hash_bytes[96] = uniswapFeeTier_bytes[28];
        hash_bytes[97] = uniswapFeeTier_bytes[29];
        hash_bytes[98] = uniswapFeeTier_bytes[30];
        hash_bytes[99] = uniswapFeeTier_bytes[31];
        hash_bytes[100] = outputTokenPortal_bytes[0];
        hash_bytes[101] = outputTokenPortal_bytes[1];
        hash_bytes[102] = outputTokenPortal_bytes[2];
        hash_bytes[103] = outputTokenPortal_bytes[3];
        hash_bytes[104] = outputTokenPortal_bytes[4];
        hash_bytes[105] = outputTokenPortal_bytes[5];
        hash_bytes[106] = outputTokenPortal_bytes[6];
        hash_bytes[107] = outputTokenPortal_bytes[7];
        hash_bytes[108] = outputTokenPortal_bytes[8];
        hash_bytes[109] = outputTokenPortal_bytes[9];
        hash_bytes[110] = outputTokenPortal_bytes[10];
        hash_bytes[111] = outputTokenPortal_bytes[11];
        hash_bytes[112] = outputTokenPortal_bytes[12];
        hash_bytes[113] = outputTokenPortal_bytes[13];
        hash_bytes[114] = outputTokenPortal_bytes[14];
        hash_bytes[115] = outputTokenPortal_bytes[15];
        hash_bytes[116] = outputTokenPortal_bytes[16];
        hash_bytes[117] = outputTokenPortal_bytes[17];
        hash_bytes[118] = outputTokenPortal_bytes[18];
        hash_bytes[119] = outputTokenPortal_bytes[19];
        hash_bytes[120] = outputTokenPortal_bytes[20];
        hash_bytes[121] = outputTokenPortal_bytes[21];
        hash_bytes[122] = outputTokenPortal_bytes[22];
        hash_bytes[123] = outputTokenPortal_bytes[23];
        hash_bytes[124] = outputTokenPortal_bytes[24];
        hash_bytes[125] = outputTokenPortal_bytes[25];
        hash_bytes[126] = outputTokenPortal_bytes[26];
        hash_bytes[127] = outputTokenPortal_bytes[27];
        hash_bytes[128] = outputTokenPortal_bytes[28];
        hash_bytes[129] = outputTokenPortal_bytes[29];
        hash_bytes[130] = outputTokenPortal_bytes[30];
        hash_bytes[131] = outputTokenPortal_bytes[31];
        hash_bytes[132] = amountOutMin_bytes[0];
        hash_bytes[133] = amountOutMin_bytes[1];
        hash_bytes[134] = amountOutMin_bytes[2];
        hash_bytes[135] = amountOutMin_bytes[3];
        hash_bytes[136] = amountOutMin_bytes[4];
        hash_bytes[137] = amountOutMin_bytes[5];
        hash_bytes[138] = amountOutMin_bytes[6];
        hash_bytes[139] = amountOutMin_bytes[7];
        hash_bytes[140] = amountOutMin_bytes[8];
        hash_bytes[141] = amountOutMin_bytes[9];
        hash_bytes[142] = amountOutMin_bytes[10];
        hash_bytes[143] = amountOutMin_bytes[11];
        hash_bytes[144] = amountOutMin_bytes[12];
        hash_bytes[145] = amountOutMin_bytes[13];
        hash_bytes[146] = amountOutMin_bytes[14];
        hash_bytes[147] = amountOutMin_bytes[15];
        hash_bytes[148] = amountOutMin_bytes[16];
        hash_bytes[149] = amountOutMin_bytes[17];
        hash_bytes[150] = amountOutMin_bytes[18];
        hash_bytes[151] = amountOutMin_bytes[19];
        hash_bytes[152] = amountOutMin_bytes[20];
        hash_bytes[153] = amountOutMin_bytes[21];
        hash_bytes[154] = amountOutMin_bytes[22];
        hash_bytes[155] = amountOutMin_bytes[23];
        hash_bytes[156] = amountOutMin_bytes[24];
        hash_bytes[157] = amountOutMin_bytes[25];
        hash_bytes[158] = amountOutMin_bytes[26];
        hash_bytes[159] = amountOutMin_bytes[27];
        hash_bytes[160] = amountOutMin_bytes[28];
        hash_bytes[161] = amountOutMin_bytes[29];
        hash_bytes[162] = amountOutMin_bytes[30];
        hash_bytes[163] = amountOutMin_bytes[31];
        hash_bytes[164] = aztecRecipientAddress_bytes[0];
        hash_bytes[165] = aztecRecipientAddress_bytes[1];
        hash_bytes[166] = aztecRecipientAddress_bytes[2];
        hash_bytes[167] = aztecRecipientAddress_bytes[3];
        hash_bytes[168] = aztecRecipientAddress_bytes[4];
        hash_bytes[169] = aztecRecipientAddress_bytes[5];
        hash_bytes[170] = aztecRecipientAddress_bytes[6];
        hash_bytes[171] = aztecRecipientAddress_bytes[7];
        hash_bytes[172] = aztecRecipientAddress_bytes[8];
        hash_bytes[173] = aztecRecipientAddress_bytes[9];
        hash_bytes[174] = aztecRecipientAddress_bytes[10];
        hash_bytes[175] = aztecRecipientAddress_bytes[11];
        hash_bytes[176] = aztecRecipientAddress_bytes[12];
        hash_bytes[177] = aztecRecipientAddress_bytes[13];
        hash_bytes[178] = aztecRecipientAddress_bytes[14];
        hash_bytes[179] = aztecRecipientAddress_bytes[15];
        hash_bytes[180] = aztecRecipientAddress_bytes[16];
        hash_bytes[181] = aztecRecipientAddress_bytes[17];
        hash_bytes[182] = aztecRecipientAddress_bytes[18];
        hash_bytes[183] = aztecRecipientAddress_bytes[19];
        hash_bytes[184] = aztecRecipientAddress_bytes[20];
        hash_bytes[185] = aztecRecipientAddress_bytes[21];
        hash_bytes[186] = aztecRecipientAddress_bytes[22];
        hash_bytes[187] = aztecRecipientAddress_bytes[23];
        hash_bytes[188] = aztecRecipientAddress_bytes[24];
        hash_bytes[189] = aztecRecipientAddress_bytes[25];
        hash_bytes[190] = aztecRecipientAddress_bytes[26];
        hash_bytes[191] = aztecRecipientAddress_bytes[27];
        hash_bytes[192] = aztecRecipientAddress_bytes[28];
        hash_bytes[193] = aztecRecipientAddress_bytes[29];
        hash_bytes[194] = aztecRecipientAddress_bytes[30];
        hash_bytes[195] = aztecRecipientAddress_bytes[31];
        hash_bytes[196] = secretHash_bytes[0];
        hash_bytes[197] = secretHash_bytes[1];
        hash_bytes[198] = secretHash_bytes[2];
        hash_bytes[199] = secretHash_bytes[3];
        hash_bytes[200] = secretHash_bytes[4];
        hash_bytes[201] = secretHash_bytes[5];
        hash_bytes[202] = secretHash_bytes[6];
        hash_bytes[203] = secretHash_bytes[7];
        hash_bytes[204] = secretHash_bytes[8];
        hash_bytes[205] = secretHash_bytes[9];
        hash_bytes[206] = secretHash_bytes[10];
        hash_bytes[207] = secretHash_bytes[11];
        hash_bytes[208] = secretHash_bytes[12];
        hash_bytes[209] = secretHash_bytes[13];
        hash_bytes[210] = secretHash_bytes[14];
        hash_bytes[211] = secretHash_bytes[15];
        hash_bytes[212] = secretHash_bytes[16];
        hash_bytes[213] = secretHash_bytes[17];
        hash_bytes[214] = secretHash_bytes[18];
        hash_bytes[215] = secretHash_bytes[19];
        hash_bytes[216] = secretHash_bytes[20];
        hash_bytes[217] = secretHash_bytes[21];
        hash_bytes[218] = secretHash_bytes[22];
        hash_bytes[219] = secretHash_bytes[23];
        hash_bytes[220] = secretHash_bytes[24];
        hash_bytes[221] = secretHash_bytes[25];
        hash_bytes[222] = secretHash_bytes[26];
        hash_bytes[223] = secretHash_bytes[27];
        hash_bytes[224] = secretHash_bytes[28];
        hash_bytes[225] = secretHash_bytes[29];
        hash_bytes[226] = secretHash_bytes[30];
        hash_bytes[227] = secretHash_bytes[31];
        hash_bytes[228] = deadlineForL1ToL2Message_bytes[0];
        hash_bytes[229] = deadlineForL1ToL2Message_bytes[1];
        hash_bytes[230] = deadlineForL1ToL2Message_bytes[2];
        hash_bytes[231] = deadlineForL1ToL2Message_bytes[3];
        hash_bytes[232] = deadlineForL1ToL2Message_bytes[4];
        hash_bytes[233] = deadlineForL1ToL2Message_bytes[5];
        hash_bytes[234] = deadlineForL1ToL2Message_bytes[6];
        hash_bytes[235] = deadlineForL1ToL2Message_bytes[7];
        hash_bytes[236] = deadlineForL1ToL2Message_bytes[8];
        hash_bytes[237] = deadlineForL1ToL2Message_bytes[9];
        hash_bytes[238] = deadlineForL1ToL2Message_bytes[10];
        hash_bytes[239] = deadlineForL1ToL2Message_bytes[11];
        hash_bytes[240] = deadlineForL1ToL2Message_bytes[12];
        hash_bytes[241] = deadlineForL1ToL2Message_bytes[13];
        hash_bytes[242] = deadlineForL1ToL2Message_bytes[14];
        hash_bytes[243] = deadlineForL1ToL2Message_bytes[15];
        hash_bytes[244] = deadlineForL1ToL2Message_bytes[16];
        hash_bytes[245] = deadlineForL1ToL2Message_bytes[17];
        hash_bytes[246] = deadlineForL1ToL2Message_bytes[18];
        hash_bytes[247] = deadlineForL1ToL2Message_bytes[19];
        hash_bytes[248] = deadlineForL1ToL2Message_bytes[20];
        hash_bytes[249] = deadlineForL1ToL2Message_bytes[21];
        hash_bytes[250] = deadlineForL1ToL2Message_bytes[22];
        hash_bytes[251] = deadlineForL1ToL2Message_bytes[23];
        hash_bytes[252] = deadlineForL1ToL2Message_bytes[24];
        hash_bytes[253] = deadlineForL1ToL2Message_bytes[25];
        hash_bytes[254] = deadlineForL1ToL2Message_bytes[26];
        hash_bytes[255] = deadlineForL1ToL2Message_bytes[27];
        hash_bytes[256] = deadlineForL1ToL2Message_bytes[28];
        hash_bytes[257] = deadlineForL1ToL2Message_bytes[29];
        hash_bytes[258] = deadlineForL1ToL2Message_bytes[30];
        hash_bytes[259] = deadlineForL1ToL2Message_bytes[31];
        hash_bytes[260] = canceller_bytes[0];
        hash_bytes[261] = canceller_bytes[1];
        hash_bytes[262] = canceller_bytes[2];
        hash_bytes[263] = canceller_bytes[3];
        hash_bytes[264] = canceller_bytes[4];
        hash_bytes[265] = canceller_bytes[5];
        hash_bytes[266] = canceller_bytes[6];
        hash_bytes[267] = canceller_bytes[7];
        hash_bytes[268] = canceller_bytes[8];
        hash_bytes[269] = canceller_bytes[9];
        hash_bytes[270] = canceller_bytes[10];
        hash_bytes[271] = canceller_bytes[11];
        hash_bytes[272] = canceller_bytes[12];
        hash_bytes[273] = canceller_bytes[13];
        hash_bytes[274] = canceller_bytes[14];
        hash_bytes[275] = canceller_bytes[15];
        hash_bytes[276] = canceller_bytes[16];
        hash_bytes[277] = canceller_bytes[17];
        hash_bytes[278] = canceller_bytes[18];
        hash_bytes[279] = canceller_bytes[19];
        hash_bytes[280] = canceller_bytes[20];
        hash_bytes[281] = canceller_bytes[21];
        hash_bytes[282] = canceller_bytes[22];
        hash_bytes[283] = canceller_bytes[23];
        hash_bytes[284] = canceller_bytes[24];
        hash_bytes[285] = canceller_bytes[25];
        hash_bytes[286] = canceller_bytes[26];
        hash_bytes[287] = canceller_bytes[27];
        hash_bytes[288] = canceller_bytes[28];
        hash_bytes[289] = canceller_bytes[29];
        hash_bytes[290] = canceller_bytes[30];
        hash_bytes[291] = canceller_bytes[31];
        hash_bytes[292] = callerOnL1_bytes[0];
        hash_bytes[293] = callerOnL1_bytes[1];
        hash_bytes[294] = callerOnL1_bytes[2];
        hash_bytes[295] = callerOnL1_bytes[3];
        hash_bytes[296] = callerOnL1_bytes[4];
        hash_bytes[297] = callerOnL1_bytes[5];
        hash_bytes[298] = callerOnL1_bytes[6];
        hash_bytes[299] = callerOnL1_bytes[7];
        hash_bytes[300] = callerOnL1_bytes[8];
        hash_bytes[301] = callerOnL1_bytes[9];
        hash_bytes[302] = callerOnL1_bytes[10];
        hash_bytes[303] = callerOnL1_bytes[11];
        hash_bytes[304] = callerOnL1_bytes[12];
        hash_bytes[305] = callerOnL1_bytes[13];
        hash_bytes[306] = callerOnL1_bytes[14];
        hash_bytes[307] = callerOnL1_bytes[15];
        hash_bytes[308] = callerOnL1_bytes[16];
        hash_bytes[309] = callerOnL1_bytes[17];
        hash_bytes[310] = callerOnL1_bytes[18];
        hash_bytes[311] = callerOnL1_bytes[19];
        hash_bytes[312] = callerOnL1_bytes[20];
        hash_bytes[313] = callerOnL1_bytes[21];
        hash_bytes[314] = callerOnL1_bytes[22];
        hash_bytes[315] = callerOnL1_bytes[23];
        hash_bytes[316] = callerOnL1_bytes[24];
        hash_bytes[317] = callerOnL1_bytes[25];
        hash_bytes[318] = callerOnL1_bytes[26];
        hash_bytes[319] = callerOnL1_bytes[27];
        hash_bytes[320] = callerOnL1_bytes[28];
        hash_bytes[321] = callerOnL1_bytes[29];
        hash_bytes[322] = callerOnL1_bytes[30];
        hash_bytes[323] = callerOnL1_bytes[31];

        let content_sha256 = dep::std::hash::sha256(hash_bytes);

        // // Convert the content_sha256 to a field element
        let mut v = 1;
        let mut high = 0 as Field;
        let mut low = 0 as Field;

        // Unroll loops because otherwise takes forever to compile
        // for i in [15 - i for i in range(16)]:
        //     print('high = high + (content_sha256[{0}] as Field) * v;'.format(i))
        //     print('low = low + (content_sha256[16 + {0}] as Field) * v;'.format(i))
        //     print('v = v * 256;');
        high = high + (content_sha256[15] as Field) * v;
        low = low + (content_sha256[16 + 15] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[14] as Field) * v;
        low = low + (content_sha256[16 + 14] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[13] as Field) * v;
        low = low + (content_sha256[16 + 13] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[12] as Field) * v;
        low = low + (content_sha256[16 + 12] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[11] as Field) * v;
        low = low + (content_sha256[16 + 11] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[10] as Field) * v;
        low = low + (content_sha256[16 + 10] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[9] as Field) * v;
        low = low + (content_sha256[16 + 9] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[8] as Field) * v;
        low = low + (content_sha256[16 + 8] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[7] as Field) * v;
        low = low + (content_sha256[16 + 7] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[6] as Field) * v;
        low = low + (content_sha256[16 + 6] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[5] as Field) * v;
        low = low + (content_sha256[16 + 5] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[4] as Field) * v;
        low = low + (content_sha256[16 + 4] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[3] as Field) * v;
        low = low + (content_sha256[16 + 3] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[2] as Field) * v;
        low = low + (content_sha256[16 + 2] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[1] as Field) * v;
        low = low + (content_sha256[16 + 1] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[0] as Field) * v;
        low = low + (content_sha256[16 + 0] as Field) * v;
        v = v * 256;

        // Abuse that a % p + b % p = (a + b) % p and that low < p
        let content_hash = low + high * v;

        content_hash
    }
}