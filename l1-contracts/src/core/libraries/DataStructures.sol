// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

/**
 * @title Data Structures Library
 * @author Aztec Labs
 * @notice Library that contains data structures used throughout the Aztec protocol
 */
library DataStructures {
  /**
   * @notice Entry struct - Done as struct to easily support extensions if needed
   * @param fee - The fee provided to sequencer for including in the inbox. 0 if Outbox (as not applicable).
   * @param count - The occurrence of the entry in the dataset
   * @param version - The version of the entry
   * @param deadline - The deadline to consume a message. Only after it, can a message be cancelled.
   */
  struct Entry {
    uint64 fee;
    uint32 count;
    uint32 version;
    uint32 deadline;
  }

  /**
   * @notice Actor on L1.
   * @param actor - The address of the actor
   * @param chainId - The chainId of the actor
   */
  struct L1Actor {
    address actor;
    uint256 chainId;
  }

  /**
   * @notice Actor on L2.
   * @param actor - The aztec address of the actor
   * @param version - Ahe Aztec instance the actor is on
   */
  struct L2Actor {
    bytes32 actor;
    uint256 version;
  }

  /**
   * @notice Struct containing a message from L1 to L2
   * @param sender - The sender of the message
   * @param recipient - The recipient of the message
   * @param content - The content of the message (application specific) padded to bytes32 or hashed if larger.
   * @param secretHash - The secret hash of the message (make it possible to hide when a specific message is consumed on L2)
   * @param deadline - The deadline to consume a message. Only after it, can a message be cancelled.
   * @param fee - The fee provided to sequencer for including the entry
   */
  struct L1ToL2Msg {
    L1Actor sender;
    L2Actor recipient;
    bytes32 content;
    bytes32 secretHash;
    uint32 deadline;
    uint64 fee;
  }

  /**
   * @notice Struct containing a message from L2 to L1
   * @param sender - The sender of the message
   * @param recipient - The recipient of the message
   * @param content - The content of the message (application specific) padded to bytes32 or hashed if larger.
   */
  struct L2ToL1Msg {
    DataStructures.L2Actor sender;
    DataStructures.L1Actor recipient;
    bytes32 content;
  }

  /**
   * @notice Struct for storing address of cross communication components and the block number when it was updated
   * @param rollup - The address of the rollup contract
   * @param inbox - The address of the inbox contract
   * @param outbox - The address of the outbox contract
   * @param blockNumber - The block number of the snapshot
   */
  struct RegistrySnapshot {
    address rollup;
    address inbox;
    address outbox;
    uint256 blockNumber;
  }
}
