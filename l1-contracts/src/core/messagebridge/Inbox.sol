// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

// Interfaces
import {IInbox} from "@aztec/core/interfaces/messagebridge/IInbox.sol";
import {IRegistry} from "@aztec/core/interfaces/messagebridge/IRegistry.sol";

// Libraries
import {Constants} from "@aztec/core/libraries/ConstantsGen.sol";
import {DataStructures} from "@aztec/core/libraries/DataStructures.sol";
import {Errors} from "@aztec/core/libraries/Errors.sol";
import {Hash} from "@aztec/core/libraries/Hash.sol";
import {MessageBox} from "@aztec/core/libraries/MessageBox.sol";

/**
 * @title Inbox
 * @author Aztec Labs
 * @notice Lives on L1 and is used to pass messages into the rollup, e.g., L1 -> L2 messages.
 */
contract Inbox is IInbox {
  using MessageBox for mapping(bytes32 entryKey => DataStructures.Entry entry);
  using Hash for DataStructures.L1ToL2Msg;

  IRegistry public immutable REGISTRY;

  mapping(bytes32 entryKey => DataStructures.Entry entry) internal entries;
  mapping(address account => uint256 balance) public feesAccrued;

  constructor(address _registry) {
    REGISTRY = IRegistry(_registry);
  }

  /**
   * @notice Inserts an entry into the Inbox
   * @dev Will emit `MessageAdded` with data for easy access by the sequencer
   * @dev msg.value - The fee provided to sequencer for including the entry
   * @param _recipient - The recipient of the entry
   * @param _deadline - The deadline to consume a message. Only after it, can a message be cancelled.
   * it is uint32 to for slot packing of the Entry struct. Should work until Feb 2106.
   * @param _content - The content of the entry (application specific)
   * @param _secretHash - The secret hash of the entry (make it possible to hide when a specific entry is consumed on L2)
   * @return The key of the entry in the set
   */
  function sendL2Message(
    DataStructures.L2Actor memory _recipient,
    uint32 _deadline,
    bytes32 _content,
    bytes32 _secretHash
  ) external payable override(IInbox) returns (bytes32) {
    if (uint256(_recipient.actor) > Constants.MAX_FIELD_VALUE) {
      revert Errors.Inbox__ActorTooLarge(_recipient.actor);
    }
    if (_deadline <= block.timestamp) revert Errors.Inbox__DeadlineBeforeNow();
    if (uint256(_content) > Constants.MAX_FIELD_VALUE) {
      revert Errors.Inbox__ContentTooLarge(_content);
    }
    if (uint256(_secretHash) > Constants.MAX_FIELD_VALUE) {
      revert Errors.Inbox__SecretHashTooLarge(_secretHash);
    }
    // `fee` is uint64 for slot packing of the Entry struct. uint64 caps at ~18.4 ETH which should be enough.
    // we revert here to safely cast msg.value into uint64.
    if (msg.value > type(uint64).max) revert Errors.Inbox__FeeTooHigh();
    uint64 fee = uint64(msg.value);
    DataStructures.L1ToL2Msg memory message = DataStructures.L1ToL2Msg({
      sender: DataStructures.L1Actor(msg.sender, block.chainid),
      recipient: _recipient,
      content: _content,
      secretHash: _secretHash,
      deadline: _deadline,
      fee: fee
    });

    bytes32 key = computeEntryKey(message);
    // Unsafe cast to uint32, but as we increment by 1 for versions to lookup the snapshots, we should be fine.
    entries.insert(key, fee, uint32(_recipient.version), _deadline, _errIncompatibleEntryArguments);

    emit MessageAdded(
      key,
      message.sender.actor,
      message.recipient.actor,
      message.sender.chainId,
      message.recipient.version,
      message.deadline,
      message.fee,
      message.content,
      message.secretHash
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
    override(IInbox)
    returns (bytes32 entryKey)
  {
    if (msg.sender != _message.sender.actor) revert Errors.Inbox__Unauthorized();
    if (block.timestamp <= _message.deadline) revert Errors.Inbox__NotPastDeadline();
    entryKey = computeEntryKey(_message);
    entries.consume(entryKey, _errNothingToConsume);
    feesAccrued[_feeCollector] += _message.fee;
    emit L1ToL2MessageCancelled(entryKey);
  }

  /**
   * @notice Batch consumes entries from the Inbox
   * @dev Only callable by the rollup contract
   * @dev Will revert if the message is already past deadline
   * @param _entryKeys - Array of entry keys (hash of the messages)
   * @param _feeCollector - The address to receive the "fee"
   */
  function batchConsume(bytes32[] memory _entryKeys, address _feeCollector)
    external
    override(IInbox)
  {
    uint256 totalFee = 0;
    // This MUST revert if not called by a listed rollup contract
    uint32 expectedVersion = uint32(REGISTRY.getVersionFor(msg.sender));
    for (uint256 i = 0; i < _entryKeys.length; i++) {
      if (_entryKeys[i] == bytes32(0)) continue;
      DataStructures.Entry memory entry = get(_entryKeys[i]);
      if (entry.version != expectedVersion) {
        revert Errors.Inbox__InvalidVersion(entry.version, expectedVersion);
      }
      // cant consume if we are already past deadline.
      if (block.timestamp > entry.deadline) revert Errors.Inbox__PastDeadline();
      entries.consume(_entryKeys[i], _errNothingToConsume);
      totalFee += entry.fee;
    }
    if (totalFee > 0) {
      feesAccrued[_feeCollector] += totalFee;
    }
  }

  /**
   * @notice Withdraws fees accrued by the sequencer
   */
  function withdrawFees() external override(IInbox) {
    uint256 balance = feesAccrued[msg.sender];
    feesAccrued[msg.sender] = 0;
    (bool success,) = msg.sender.call{value: balance}("");
    if (!success) revert Errors.Inbox__FailedToWithdrawFees();
  }

  /**
   * @notice Fetch an entry
   * @param _entryKey - The key to lookup
   * @return The entry matching the provided key
   */
  function get(bytes32 _entryKey)
    public
    view
    override(IInbox)
    returns (DataStructures.Entry memory)
  {
    return entries.get(_entryKey, _errNothingToConsume);
  }

  /**
   * @notice Check if entry exists
   * @param _entryKey - The key to lookup
   * @return True if entry exists, false otherwise
   */
  function contains(bytes32 _entryKey) public view override(IInbox) returns (bool) {
    return entries.contains(_entryKey);
  }

  /**
   * @notice Given a message, computes an entry key for the Inbox
   * @param _message - The L1 to L2 message
   * @return The hash of the message (used as the key of the entry in the set)
   */
  function computeEntryKey(DataStructures.L1ToL2Msg memory _message)
    public
    pure
    override(IInbox)
    returns (bytes32)
  {
    return _message.sha256ToField();
  }

  /**
   * @notice Error function passed in cases where there might be nothing to consume
   * @dev Used to have message box library throw `Inbox__` prefixed errors
   * @param _entryKey - The key to lookup
   */
  function _errNothingToConsume(bytes32 _entryKey) internal pure {
    revert Errors.Inbox__NothingToConsume(_entryKey);
  }

  /**
   * @notice Error function passed in cases where insertions can fail
   * @dev Used to have message box library throw `Inbox__` prefixed errors
   * @param _entryKey - The key to lookup
   * @param _storedFee - The fee stored in the entry
   * @param _feePassed - The fee passed into the insertion
   * @param _storedVersion - The version stored in the entry
   * @param _versionPassed - The version passed into the insertion
   * @param _storedDeadline - The deadline stored in the entry
   * @param _deadlinePassed - The deadline passed into the insertion
   */
  function _errIncompatibleEntryArguments(
    bytes32 _entryKey,
    uint64 _storedFee,
    uint64 _feePassed,
    uint32 _storedVersion,
    uint32 _versionPassed,
    uint32 _storedDeadline,
    uint32 _deadlinePassed
  ) internal pure {
    revert Errors.Inbox__IncompatibleEntryArguments(
      _entryKey,
      _storedFee,
      _feePassed,
      _storedVersion,
      _versionPassed,
      _storedDeadline,
      _deadlinePassed
    );
  }
}
