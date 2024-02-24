pragma solidity >=0.8.18;

import {IERC20} from "@oz/token/ERC20/IERC20.sol";
import {SafeERC20} from "@oz/token/ERC20/utils/SafeERC20.sol";

// Messaging
import {IRegistry} from "../../src/core/interfaces/messagebridge/IRegistry.sol";
import {IInbox} from "../../src/core/interfaces/messagebridge/IInbox.sol";
import {DataStructures} from "../../src/core/libraries/DataStructures.sol";
// docs:start:content_hash_sol_import
import {Hash} from "../../src/core/libraries/Hash.sol";
// docs:end:content_hash_sol_import

// docs:start:init
contract GasPortal {
  using SafeERC20 for IERC20;

  IRegistry public registry;
  IERC20 public underlying;
  bytes32 public l2TokenAddress;

  function initialize(address _registry, address _underlying, bytes32 _l2TokenAddress) external {
    registry = IRegistry(_registry);
    underlying = IERC20(_underlying);
    l2TokenAddress = _l2TokenAddress;
  }
  // docs:end:init

  // docs:start:deposit_public
  /**
   * @notice Deposit funds into the portal and adds an L2 message which can only be consumed publicly on Aztec
   * @param _to - The aztec address of the recipient
   * @param _amount - The amount to deposit
   * @param _canceller - The address that can cancel the L1 to L2 message
   * @param _deadline - The timestamp after which the entry can be cancelled
   * @param _secretHash - The hash of the secret consumable message. The hash should be 254 bits (so it can fit in a Field element)
   * @return - The key of the entry in the Inbox
   */
  function depositToAztecPublic(
    bytes32 _to,
    uint256 _amount,
    address _canceller,
    uint32 _deadline,
    bytes32 _secretHash
  ) external payable returns (bytes32) {
    // Preamble
    IInbox inbox = registry.getInbox();
    DataStructures.L2Actor memory actor = DataStructures.L2Actor(l2TokenAddress, 1);

    // Hash the message content to be reconstructed in the receiving contract
    bytes32 contentHash = Hash.sha256ToField(
      abi.encodeWithSignature("mint_public(bytes32,uint256,address)", _to, _amount, _canceller)
    );

    // Hold the tokens in the portal
    underlying.safeTransferFrom(msg.sender, address(this), _amount);

    // Send message to rollup
    return inbox.sendL2Message{value: msg.value}(actor, _deadline, contentHash, _secretHash);
  }

  /**
   * @notice Cancel a public depositToAztec L1 to L2 message
   * @dev only callable by the `canceller` of the message
   * @param _to - The aztec address of the recipient in the original message
   * @param _amount - The amount to deposit per the original message
   * @param _deadline - The timestamp after which the entry can be cancelled
   * @param _secretHash - The hash of the secret consumable message in the original message
   * @param _fee - The fee paid to the sequencer
   * @return The key of the entry in the Inbox
   */
  function cancelL1ToAztecMessagePublic(
    bytes32 _to,
    uint256 _amount,
    uint32 _deadline,
    bytes32 _secretHash,
    uint64 _fee
  ) external returns (bytes32) {
    IInbox inbox = registry.getInbox();
    DataStructures.L1Actor memory l1Actor = DataStructures.L1Actor(address(this), block.chainid);
    DataStructures.L2Actor memory l2Actor = DataStructures.L2Actor(l2TokenAddress, 1);
    DataStructures.L1ToL2Msg memory message = DataStructures.L1ToL2Msg({
      sender: l1Actor,
      recipient: l2Actor,
      content: Hash.sha256ToField(
        abi.encodeWithSignature("mint_public(bytes32,uint256,address)", _to, _amount, msg.sender)
        ),
      secretHash: _secretHash,
      deadline: _deadline,
      fee: _fee
    });
    bytes32 entryKey = inbox.cancelL2Message(message, address(this));
    // release the funds to msg.sender (since the content hash (& message key) is derived by hashing the caller,
    // we confirm that msg.sender is same as `_canceller` supplied when creating the message)
    underlying.transfer(msg.sender, _amount);
    return entryKey;
  }
}
