// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {IRegistry} from "@aztec/core/interfaces/messagebridge/IRegistry.sol";
import {IInbox} from "@aztec/core/interfaces/messagebridge/IInbox.sol";
import {IOutbox} from "@aztec/core/interfaces/messagebridge/IOutbox.sol";
import {Errors} from "@aztec/core/libraries/Errors.sol";

import {MockVerifier} from "@aztec/mock/MockVerifier.sol";
import {Decoder} from "./Decoder.sol";

/**
 * @title Rollup
 * @author Aztec Labs
 * @notice Rollup contract that are concerned about readability and velocity of development
 * not giving a damn about gas costs.
 *
 * Work in progress
 */
contract Rollup is Decoder {
  event L2BlockProcessed(uint256 indexed blockNum);

  MockVerifier public immutable VERIFIER;
  IRegistry public immutable REGISTRY;

  bytes32 public rollupStateHash;

  constructor(IRegistry _registry) {
    VERIFIER = new MockVerifier();
    REGISTRY = _registry;
  }

  /**
   * @notice Process an incoming L2Block and progress the state
   * @param _proof - The proof of correct execution
   * @param _l2Block - The L2Block data, formatted as outlined in `Decoder.sol`
   */
  function process(bytes memory _proof, bytes calldata _l2Block) external {
    (
      uint256 l2BlockNumber,
      bytes32 oldStateHash,
      bytes32 newStateHash,
      bytes32 publicInputHash,
      bytes32[] memory l2ToL1Msgs,
      bytes32[] memory l1ToL2Msgs
    ) = _decode(_l2Block);

    // @todo @LHerskind Proper genesis state. If the state is empty, we allow anything for now.
    if (rollupStateHash != bytes32(0) && rollupStateHash != oldStateHash) {
      revert Errors.Rollup__InvalidStateHash(rollupStateHash, oldStateHash);
    }

    bytes32[] memory publicInputs = new bytes32[](1);
    publicInputs[0] = publicInputHash;

    if (!VERIFIER.verify(_proof, publicInputs)) {
      revert Errors.Rollup__InvalidProof();
    }

    rollupStateHash = newStateHash;

    // @todo (issue #605) handle fee collector
    IInbox inbox = REGISTRY.getInbox();
    inbox.batchConsume(l1ToL2Msgs, msg.sender);

    IOutbox outbox = REGISTRY.getOutbox();
    outbox.sendL1Messages(l2ToL1Msgs);

    emit L2BlockProcessed(l2BlockNumber);
  }
}
