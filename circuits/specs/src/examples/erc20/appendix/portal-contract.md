## Example Portal Contract

Here's an example (in pseudocode) of what the Portal Contract for our ERC20 shielding application might look like:


<!--
RECOMMEND YOU SET A VERTICAL LINE IN VSCODE when editing 'code snippets',
to guide word wrapping. The book (with its current settings) only shows the first 90
characters. Add the following to your settings json:
"editor.rulers": [85],
-->


<!-- actually Solidity, but I put 'js', because vsCode doesn't recognise solidity -->
```js
import IERC20 from '...';
import IRollupProcessor from '...';

contract ERC20Shield {

    IRollupProcessor rollupProcessor;
    uint32 public immutable l2ContractAddress;
    bytes public immutable l2DepositFunctionData;

    struct PendingDepositDatum {
        address depositorAddress;
        address depositCurrency;
        uint256 amount;
    }

    // l2CallHash => PendingDepositDatum[] (dynamic to allow for duplicate calls)
    mapping(uint256 => PendingDepositDatum[]) public pendingDepositData;

    modifier onlyRollupProcessor {
        require(
            msg.sender == address(rollupProcessor),
            "Only the rollup processor can call this function."
        );
        _;
    }

    constructor (
        address _rollupProcessorAddress,
        uint32 _l2ContractAddress,
        bytes _l2DepositFunctionData
    ) {
        rollupProcessor = IRollupProcessor(rollupProcessorAddress);
        l2ContractAddress = _l2ContractAddress;
        l2DepositFunctionData = _l2DepositFunctionData;
    }


    /********************************************************************************
     * DEPOSIT FUNCTIONS
     *******************************************************************************/

    /**
     * ERC20 token example only (not ETH).
     * The user MUST have called `approve` in the ERC20 contract of the token they
     * wish to deposit (and the fee contract), so that funds may be transferred by
     * this function.
     * Q: can the user pass a signed 'approve' tx blob as calldata instead?
     */
    function deposit(
        uint256 amount, // amount to deposit
        address depositCurrency, // currency of the deposit

        uint256 l2Fee,
        address l2FeeCurrency, // currency of the fee
        uint256 l2CallHash, // a callStackItemHash
    ) external {
        IERC20 depositContract = IERC20(depositCurrency);
        IERC20 feeContract = IERC20(l2FeeCurrency);

        depositContract.transferFrom(msg.sender, address(this), amount);

        // At the moment two fee transfers happen:
        // - From the user to the portal contract
        // - From the portal contract to the RollupProcessor contract.
        // The idea being, maybe we can avoid the user ever interacting with (and
        // giving 'approvals' to) the rollup contract directly. BUT, that's expensive
        // and perhaps wasteful.
        feeContract.transferFrom(msg.sender, address(this), l2Fee);
        feeContract.increaseAllowance(address(rollupProcessor), l2Fee);

        // Record the data about the deposit (indexed by the l2CallHash), for when
        // the callBack is executed later.
        PendingDepositDatum memory pendingDeposit = PendingDepositDatum({
            depositorAddress: msg.sender,
            depositCurrency,
            amount,
        });
        pendingDepositData[l2CallHash].push(pendingDeposit);

        // Will be hardcoded function selectors.
        bytes24 callback = // concat(address(this), functionSelector);
        bytes24 callbackOnCancel = // concat(address(this), functionSelector);
        
        // Call the RollupProcessor contract:
        rollupProcessor.callL2AndPayFee(
            l2CallHash,
            callback,
            callbackOnCancel,
            l2Fee,
            l2FeeCurrency,
        );
    }

    /**
     * Finalise a deposit tx in response to the L2 tx being processed.
     * @notice these @params are _rigidly_ dictated by the RollupProcessor's
     * interface, since it's _always_ the RollupProcessor which calls this.
     */
    function depositCallback(
        uint256 l2CallHash,
        uint256 callIndex,
        bytes functionData,
        uint256[4] emittedPublicInputs,
    ) external onlyRollupProcessor {
        // First check that the function which was executed on L2 was _actually_ 
        // the 'deposit' circuit of the L2 contract which is actually associated with
        // this portal contract:
        require(
            functionData == l2DepositFunctionData,
            "The wrong function was executed on L2!"
        );

        // Retrieve the pending deposit data:
        PendingDepositDatum memory pendingDeposit = 
            pendingDepositData[l2CallHash][callIndex];

        // The portal contract will receive all emittedPublicInputs (that were 
        // emitted by the circuit). It's up to the portal contract to interpret
        // these numbers in an app-specific way.
        uint256 amount = emittedPublicInputs[0]; // for example
        // This example ignores the other emittedPublicInputs.

        require(
            amount == pendingDeposit.amount,
            "The amount deposited on L2 does not match the amount deposited on L1!"
        );

        // Delete the pending deposit (as it's now finalised):
        delete pendingDepositData[l2CallHash][callIndex];
    }

    /**
     * Called if the L2 tx is cancelled in the RollupProcessor.
     * @notice these @params are _rigidly_ dictated by the RollupProcessor's
     * interface, since it's _always_ the RollupProcessor which calls this.
     */
    // TODO: NOT SURE IF AN L1 -> L2 CALL SHOULD BE CANCELLABLE! (The rollup might 
    // already be in-flight before the cancellation).
    function depositCancellationCallback(
        uint256 l2CallHash,
        uint256 callIndex,
    ) external onlyRollupProcessor {

        // Retrieve the pending deposit data:
        PendingDepositDatum memory pendingDeposit = 
            pendingDepositData[l2CallHash][callIndex];

        // Return the deposit back to the user:
        IERC20 erc20 = IERC20(pendingDeposit.depositCurrency);
        erc20.transfer(pendingDeposit.depositorAddress, pendingDeposit.amount);

        // Delete the pending deposit (as it's now been cancelled):
        delete pendingDepositData[l2CallHash][callIndex];
    }

    /********************************************************************************
     * No need for a transfer function. Private transfers don't interact with L1 state.
     *******************************************************************************/
 
    /********************************************************************************
     * WITHDRAWAL FUNCTIONS
     *******************************************************************************/
 
    // NOTE: the RollupProcessor MUST NOT call functions of portal contracts unless
    // the corresponding snark was verified as true.
    function withdraw(
        address msgSender, // not to be confused with msg.sender.
                           // This is the original user's address.
        calldata userWithdrawalSignature,
        address erc20Address,
        address to,
        uint256 amount
    ) external onlyRollupProcessor returns (bool success) {
        // Check withdrawal signature (that this user _actually_ wants to perform this action).
        // The rollup processor has already checked the circuit logic; that this user has sufficient funds.
        IERC20 erc20 = IERC20(erc20Address);
        require(erc20.balanceOf(address(this)) >= amount, "Insufficient funds"); // this should _never_ be possible, given the circuit logic.
        // Call the erc20's transfer function, transferring `amount` from this portal contract's address to the `to` address
        return success; // this will have been updated depending on the above steps.
    }

}
```