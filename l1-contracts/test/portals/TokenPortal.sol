pragma solidity >=0.8.18;

import {IERC20} from "@oz/token/ERC20/IERC20.sol";
import {SafeERC20} from "@oz/token/ERC20/utils/SafeERC20.sol";

// Messaging
import {IRegistry} from "../../src/core/interfaces/messagebridge/IRegistry.sol";
import {IInbox} from "../../src/core/interfaces/messagebridge/IInbox.sol";
import {IOutbox} from "../../src/core/interfaces/messagebridge/IOutbox.sol";
import {DataStructures} from "../../src/core/libraries/DataStructures.sol";
// docs:start:content_hash_sol_import
import {Hash} from "../../src/core/libraries/Hash.sol";
// docs:end:content_hash_sol_import

// docs:start:init
contract TokenPortal {
  using SafeERC20 for IERC20;

  IRegistry public registry;
  IERC20 public underlying;
  bytes32 public l2Bridge;

  function initialize(address _registry, address _underlying, bytes32 _l2Bridge) external {
    registry = IRegistry(_registry);
    underlying = IERC20(_underlying);
    l2Bridge = _l2Bridge;
  }
  // docs:end:init

  // docs:start:deposit_public
  /**
   * @notice Deposit funds into the portal and adds an L2 message which can only be consumed publicly on Aztec
   * @param _to - The aztec address of the recipient
   * @param _amount - The amount to deposit
   * @param _secretHash - The hash of the secret consumable message. The hash should be 254 bits (so it can fit in a Field element)
   * @return The key of the entry in the Inbox
   */
  function depositToAztecPublic(bytes32 _to, uint256 _amount, bytes32 _secretHash)
    external
    returns (bytes32)
  {
    // Preamble
    IInbox inbox = registry.getInbox();
    DataStructures.L2Actor memory actor = DataStructures.L2Actor(l2Bridge, 1);

    // Hash the message content to be reconstructed in the receiving contract
    bytes32 contentHash =
      Hash.sha256ToField(abi.encodeWithSignature("mint_public(bytes32,uint256)", _to, _amount));

    // Hold the tokens in the portal
    underlying.safeTransferFrom(msg.sender, address(this), _amount);

    // Send message to rollup
    return inbox.sendL2Message(actor, contentHash, _secretHash);
  }
  // docs:end:deposit_public

  // docs:start:deposit_private
  /**
   * @notice Deposit funds into the portal and adds an L2 message which can only be consumed privately on Aztec
   * @param _secretHashForRedeemingMintedNotes - The hash of the secret to redeem minted notes privately on Aztec. The hash should be 254 bits (so it can fit in a Field element)
   * @param _amount - The amount to deposit
   * @param _secretHashForL2MessageConsumption - The hash of the secret consumable L1 to L2 message. The hash should be 254 bits (so it can fit in a Field element)
   * @return The key of the entry in the Inbox
   */
  function depositToAztecPrivate(
    bytes32 _secretHashForRedeemingMintedNotes,
    uint256 _amount,
    bytes32 _secretHashForL2MessageConsumption
  ) external returns (bytes32) {
    // Preamble
    IInbox inbox = registry.getInbox();
    DataStructures.L2Actor memory actor = DataStructures.L2Actor(l2Bridge, 1);

    // Hash the message content to be reconstructed in the receiving contract
    bytes32 contentHash = Hash.sha256ToField(
      abi.encodeWithSignature(
        "mint_private(bytes32,uint256)", _secretHashForRedeemingMintedNotes, _amount
      )
    );

    // Hold the tokens in the portal
    underlying.safeTransferFrom(msg.sender, address(this), _amount);

    // Send message to rollup
    return inbox.sendL2Message(actor, contentHash, _secretHashForL2MessageConsumption);
  }
  // docs:end:deposit_private

  // docs:start:token_portal_withdraw
  /**
   * @notice Withdraw funds from the portal
   * @dev Second part of withdraw, must be initiated from L2 first as it will consume a message from outbox
   * @param _recipient - The address to send the funds to
   * @param _amount - The amount to withdraw
   * @param _withCaller - Flag to use `msg.sender` as caller, otherwise address(0)
   * @param _l2BlockNumber - The address to send the funds to
   * @param _leafIndex - The amount to withdraw
   * @param _path - Flag to use `msg.sender` as caller, otherwise address(0)
   * Must match the caller of the message (specified from L2) to consume it.
   */
  function withdraw(
    address _recipient,
    uint256 _amount,
    bool _withCaller,
    uint256 _l2BlockNumber,
    uint256 _leafIndex,
    bytes32[] calldata _path
  ) external {
    DataStructures.L2ToL1Msg memory message = DataStructures.L2ToL1Msg({
      sender: DataStructures.L2Actor(l2Bridge, 1),
      recipient: DataStructures.L1Actor(address(this), block.chainid),
      content: Hash.sha256ToField(
        abi.encodeWithSignature(
          "withdraw(address,uint256,address)",
          _recipient,
          _amount,
          _withCaller ? msg.sender : address(0)
        )
        )
    });

    IOutbox outbox = registry.getOutbox();

    outbox.consume(message, _l2BlockNumber, _leafIndex, _path);

    underlying.transfer(_recipient, _amount);
  }
  // docs:end:token_portal_withdraw
}
