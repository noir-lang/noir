pragma solidity >=0.8.18;

import {IERC20} from "@oz/token/ERC20/IERC20.sol";
import {SafeERC20} from "@oz/token/ERC20/utils/SafeERC20.sol";

// Messaging
import {IRegistry} from "@aztec/core/interfaces/messagebridge/IRegistry.sol";
import {IInbox} from "@aztec/core/interfaces/messagebridge/IInbox.sol";
import {DataStructures} from "@aztec/core/libraries/DataStructures.sol";
import {Hash} from "@aztec/core/libraries/Hash.sol";

contract TokenPortal {
  using SafeERC20 for IERC20;

  IRegistry public registry;
  IERC20 public underlying;
  bytes32 public l2TokenAddress;

  function initialize(address _registry, address _underlying, bytes32 _l2TokenAddress) external {
    registry = IRegistry(_registry);
    underlying = IERC20(_underlying);
    l2TokenAddress = _l2TokenAddress;
  }

  /**
   * @notice Deposit funds into the portal and adds an L2 message.
   * @param _to - The aztec address of the recipient
   * @param _amount - The amount to deposit
   * @param _deadline - The timestamp after which the entry can be cancelled
   * @param _secretHash - The hash of the secret consumable message
   * @return The key of the entry in the Inbox
   */
  function depositToAztec(bytes32 _to, uint256 _amount, uint32 _deadline, bytes32 _secretHash)
    external
    payable
    returns (bytes32)
  {
    // Preamble
    // @todo: (issue #624) handle different versions
    IInbox inbox = registry.getInbox();
    DataStructures.L2Actor memory actor = DataStructures.L2Actor(l2TokenAddress, 1);

    // Hash the message content to be reconstructed in the receiving contract
    bytes32 contentHash =
      Hash.sha256ToField(abi.encodeWithSignature("mint(uint256,bytes32)", _amount, _to));

    // Hold the tokens in the portal
    underlying.safeTransferFrom(msg.sender, address(this), _amount);

    // Send message to rollup
    return inbox.sendL2Message{value: msg.value}(actor, _deadline, contentHash, _secretHash);
  }
}
