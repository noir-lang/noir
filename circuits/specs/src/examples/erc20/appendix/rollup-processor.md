<!-- actually Solidity, but I put 'js', because vsCode doesn't recognise solidity -->
```js
contract RollupProcessor is IRollupProcessor {
    
    // Needs reordering to save space.
    struct PendingL2TxDatum {
        address feePayer;
        uint256 fee;
        address feeCurrency;
        bytes24 callback;
        uint256 callbackGasLimit; // TODO: maybe this can be more-cheaply communicated
                                  // through the L2 call (rollup calldata)?
        bytes24 callbackOnCancel;
        uint256 callbackOnCancelGasLimit; // TODO: as above.
    }

    // portalContractAddress => l2ContractAddress lookup.
    // mapping(address => uint32) l2ContractAddresses;
    // l2ContractAddress => portalContractAddress lookup.
    // mapping(uint32 => address) portalContractAddresses;
    
    // l2CallHash => PendingL2TxData[] (dynamic array to allow for duplicate calls)
    mapping(uint256 => PendingL2TxDatum[]) l2TxPool;
    
    event NewDeployment(...);

    // Events to inform the rollup provider of potential fees they can earn from L1.
    event NewPendingL2Tx(
        uint256 l2CallHash,
        uint256 callIndex,
        uint256 fee,
        address feeCurrency
    );
    event ProcessedPendingL2Tx(
        uint256 l2CallHash,
        uint256 callIndex
    );
    event CancelledPendingL2Tx(
        uint256 l2CallHash,
        uint256 callIndex
    );
    

    /**
     * This probably doesn't need to be in the RollupProcessor contract;
     * it can be separate.
     */
    function deployPortalContract(bytes memory bytecode, uint salt)
    external pure returns (address portalContractAddress) {
        // Do a CREATE2 call to deploy the portal contract
        assembly {
            portalContractAddress := create2(
                callvalue(), // wei sent with current call
                // Actual code starts after skipping the first 32 bytes
                add(bytecode, 0x20),
                mload(bytecode), // Load the size of code from the first 32 bytes
                salt
            )
            if iszero(extcodesize(portalContractAddress)) {
                revert(0, 0)
            }
        } 
    }
    
    
    /**
     * @param callback - contract address & function selector of an L1 callback
     * function which will be executed once the L2 tx is processed by this contract.
     * 
     * I think the `callback` will have to have a rigid functionSelector with params:
     * - l2CallHash (as a 'lookup' key which the portal contract can use)
     * - emittedPublicInputs
     */
    function callL2AndPayFee(
        uint256 l2CallHash,
        bytes24 callback,
        uint32 callbackGasLimit, // TODO: maybe this can more-cheaply be communicated
                                 // through the L2 call (rollup calldata)?
        bytes24 callbackOnCancel,
        uint32 callbackOnCancelGasLimit, // TODO (as above)
        uint256 fee, // pays for both the L2 call and the L1 callback gas
        address feeCurrency,
    ) external {
        // Note: an l2CallHash contains a callContext, which contains a msgSender,
        // so it is unique (unless the user calls twice). If the user calls twice,
        // the pool is a dynamic array, so we'll push the call again. Portal
        // Contracts will need to allow for duplicate l2CallHashes.

        PendingTxDatum memory tx = PendingL2TxDatum({
            feePayer: msg.sender,
            fee,
            feeCurrency,
            callback,
            callbackGasLimit,
            callbackOnCancel,
            callbackOnCancelGasLimit,
        });

        uint256 callIndex = l2TxPool[l2CallHash].push(tx);

        IERC20 erc20 = IERC20(feeCurrency);
        erc20.transferFrom(msg.sender, address(this), fee);

        emit NewPendingL2Tx(l2CallHash, callIndex, fee, feeCurrency);
    }


    // TODO: Should cancellation even be allowed? A user's L2 tx could be processed
    // by the rollup provider, but then get cancelled before the rollup provider 
    // publishes their rollup; thereby missing out on being paid.
    // On the other hand, cancellation does seem important.
    // Maybe the fee repayment could be delayed for a week? Seems nasty.
    cancelL2Call(l2CallHash, callIndex) external {

        PendingTxDatum memory tx = l2TxPool[l2CallHash][callIndex];

        // For now, let's say the feePayer is the person permitted to cancel:
        require(msg.sender == tx.feePayer, "Permission to cancel denied!");

        // Return the fee
        if (tx.fee > 0) {
            IERC20 erc20 = IERC20(tx.feeCurrency);
            erc20.transfer(tx.feePayer, tx.fee);
        }

        // This is pseudocode:
        tx.callbackOnCancel(l2CallHash, callIndex, { tx.callbackOnCancelGasLimit });

        // Delete the tx from the pool:
        delete l2TxPool[l2CallHash][callIndex];

        emit CancelledPendingL2Tx(l2CallHash, callIndex);
    }


    /**
     * Upon submission of a rollup, this contract will identify any L2 txs which were
     * invoked via 'this.callL2AndPayFee()'. It will then pass params to this
     * function.
     * @param rollupProvider - the person who submitted the rollup
     */
    function executeCallbackAndPayFee(
        uint256 l2CallHash,
        uint256 callIndex,
        bytes functionSignature,
        uint256[4] emittedPublicInputs,
        address rollupProvider,
    ) private {
        PendingTxDatum memory tx = l2TxPool[l2CallHash][callIndex];

        // Make a call to the callback function, passing the emittedPublicInputs as
        // params, and passing the callbackGasLimit. Handle the call so that even if
        // it fails or runs out of gas, the rollup provider still gets paid (and to
        // prevent griefing).
        // This is pseudocode!!!
        tx.callback(
            l2CallHash,
            callIndex,
            functionSignature,
            emittedPublicInputs,
            { tx.callbackGasLimit }
        );

        // Pay the rollup provider
        if (tx.fee > 0) {
            IERC20 erc20 = IERC20(tx.feeCurrency);
            erc20.transfer(rollupProvider, tx.fee);
        }

        // delete the pending tx from the pool:
        delete l2TxPool[l2CallHash][callIndex];

        emit ProcessedPendingL2Tx(l2CallHash, callIndex);
    }


    function processRollup(...) {
        
        // Verify the proof

        // Decode the calldata

        // Loop through all txs in the rollup.
        // Identify the following tx scenarios from the calldata:
        // (see the Public Input ABIs section for more details of these bools)
        //  - payFeeFromL1 -> call payFee(l2CallHash, callIndex, rollupProvider);
        //                    using l2CallHash, callIndex from the calldata.
        //                    The rollup processor must be designed so this fee
        //                    payment CANNOT fail. Otherwise we'd have L2 state
        //                    changes for free. Hold the fee in escrow. The rollup
        //                    provider shouldn't process the l2 tx unless he sees the
        //                    fee exists on-chain. (And don't allow the fee to be
        //                    withdrawn by the payer? (i.e. disallow cancellation?))
        //  - calledFromL1 -> call executeCallBackAndPayFee(...)
        //                    using data from the calldata.
        //  - isContractDeployment -> call something... TODO
        //
        // Loop through the l1CallStack of each tx in the rollup.
        //  - For each l1CallStackItem (which will be a keccak hash of data):
        //    - The unpacked data of the call (functionSelector, argumentEncoding)
        //      will need to have been provided as calldata when submitting the 
        //      rollup.
        //    - Reconcile the l1CallStackItem:
        //      - require(l1CallStackItem == keccak256(unpackedData));
        //    - Call the L1 function with args described by the unpacked data.
        //    - Note: if any L2 tx in a callstack makes an L1 call, NONE of the L2
        //      state changes may be finalised until all the L1 calls have been
        //      successful. The next rollup block must 'complete' commitments or 
        //      update the l1ResultsTree.

    }
}
```