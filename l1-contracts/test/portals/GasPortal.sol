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
   * @param _secretHash - The hash of the secret consumable message. The hash should be 254 bits (so it can fit in a Field element)
   * @return - The key of the entry in the Inbox
   */
  function depositToAztecPublic(bytes32 _to, uint256 _amount, bytes32 _secretHash)
    external
    returns (bytes32)
  {
    // Preamble
    IInbox inbox = registry.getInbox();
    DataStructures.L2Actor memory actor = DataStructures.L2Actor(l2TokenAddress, 1);

    // Hash the message content to be reconstructed in the receiving contract
    bytes32 contentHash =
      Hash.sha256ToField(abi.encodeWithSignature("mint_public(bytes32,uint256)", _to, _amount));

    // Hold the tokens in the portal
    underlying.safeTransferFrom(msg.sender, address(this), _amount);

    // Send message to rollup
    return inbox.sendL2Message(actor, contentHash, _secretHash);
  }
}
