// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {IInbox} from "@aztec/core/interfaces/messagebridge/IInbox.sol";
import {DataStructures} from "@aztec/core/libraries/DataStructures.sol";
import {MessageBox} from "./MessageBox.sol";

/**
 * @title Inbox
 * @author Aztec Labs
 * @notice Lives on L1 and is used to pass messages into the rollup, e.g., L1 -> L2 messages.
 */
contract Inbox is MessageBox, IInbox {
  error Inbox__DeadlineBeforeNow();
  error Inbox__NotPastDeadline();
  error Inbox__PastDeadline();
  error Inbox__Unauthorized();
  error Inbox__FailedToWithdrawFees();

  mapping(address account => uint256 balance) public feesAccrued;

  constructor(address _registry) MessageBox(_registry) {}

  /**
   * @notice Given a message, computes an entry key for the Inbox
   * @param message - The L1 to L2 message
   * @return The hash of the message (used as the key of the entry in the set)
   */
  function computeMessageKey(DataStructures.L1ToL2Msg memory message) public pure returns (bytes32) {
    return bytes32(
      uint256(
        sha256(
          abi.encode(
            message.sender,
            message.recipient,
            message.content,
            message.secretHash,
            message.deadline,
            message.fee
          )
        )
      ) % P // TODO: Replace mod P later on when we have a better idea of how to handle Fields.
    );
  }

  /**
   * @notice Inserts an entry into the Inbox
   * @dev Will emit `MessageAdded` with data for easy access by the sequencer
   * @dev msg.value - The fee provided to sequencer for including the entry
   * @param _recipient - The recipient of the entry
   * @param _deadline - The deadline to consume a message. Only after it, can a message be cancalled.
   * @param _content - The content of the entry (application specific)
   * @param _secretHash - The secret hash of the entry (make it possible to hide when a specific entry is consumed on L2)
   * @return The key of the entry in the set
   */
  function sendL2Message(
    DataStructures.L2Actor memory _recipient,
    uint32 _deadline,
    bytes32 _content,
    bytes32 _secretHash
  ) external payable returns (bytes32) {
    if (_deadline <= block.timestamp) revert Inbox__DeadlineBeforeNow();
    uint64 fee = uint64(msg.value);
    DataStructures.L1ToL2Msg memory message = DataStructures.L1ToL2Msg({
      sender: DataStructures.L1Actor(msg.sender, block.chainid),
      recipient: _recipient,
      content: _content,
      secretHash: _secretHash,
      deadline: _deadline,
      fee: fee
    });

    bytes32 key = computeMessageKey(message);
    _insert(key, fee, _deadline);

    emit MessageAdded(
      key,
      message.sender.actor,
      message.recipient.actor,
      message.sender.chainId,
      message.recipient.version,
      message.deadline,
      message.fee,
      message.content
    );

    return key;
  }

  /**
   * @notice Cancel a pending L2 message
   * @dev Will revert if the deadline have not been crossed - message only cancellable past the deadline
   *  so it cannot be yanked away while the sequencer is building a block including it
   * @dev Must be called by portal that inserted the entry
   * @param _message - The content of the entry (application specific)
   * @param _feeCollector - The address to receive the "fee"
   * @return entryKey - The key of the entry removed
   */
  function cancelL2Message(DataStructures.L1ToL2Msg memory _message, address _feeCollector)
    external
    returns (bytes32 entryKey)
  {
    if (msg.sender != _message.sender.actor) revert Inbox__Unauthorized();
    if (_message.deadline <= block.timestamp) revert Inbox__NotPastDeadline();
    entryKey = computeMessageKey(_message);
    _consume(entryKey);
    feesAccrued[_feeCollector] += _message.fee;
    emit L1ToL2MessageCancelled(entryKey);
  }

  /**
   * @notice Batch consumes entries from the Inbox
   * @dev Only callable by the rollup contract
   * @dev Will revert if the message is already past deadline
   * @param entryKeys - Array of entry keys (hash of the messages)
   * @param _feeCollector - The address to receive the "fee"
   */
  function batchConsume(bytes32[] memory entryKeys, address _feeCollector) external onlyRollup {
    uint256 totalFee = 0;
    for (uint256 i = 0; i < entryKeys.length; i++) {
      // TODO: Combine these to optimise for gas.
      DataStructures.Entry memory entry = get(entryKeys[i]);
      if (entry.deadline > block.timestamp) revert Inbox__PastDeadline();
      _consume(entryKeys[i]);
      totalFee += entry.fee;
    }
    feesAccrued[_feeCollector] += totalFee;
  }

  /**
   * @notice Withdraws fees accrued by the sequencer
   */
  function withdrawFees() external {
    uint256 balance = feesAccrued[msg.sender];
    feesAccrued[msg.sender] = 0;
    (bool success,) = msg.sender.call{value: balance}("");
    if (!success) revert Inbox__FailedToWithdrawFees();
  }
}
