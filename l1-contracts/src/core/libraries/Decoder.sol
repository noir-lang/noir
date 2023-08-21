// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

// Libraries
import {Constants} from "@aztec/core/libraries/ConstantsGen.sol";
import {Hash} from "@aztec/core/libraries/Hash.sol";

/**
 * @title Decoder Library
 * @author Aztec Labs
 * @notice Decoding a L2 block, concerned with readability and velocity of development
 * not giving a damn about gas costs.
 * @dev there is currently no padding of the elements, so we are for now assuming nice trees as inputs.
 * Furthermore, if no contract etc are deployed, we expect there to be address(0) for input.
 *
 * -------------------
 * You can use https://gist.github.com/LHerskind/724a7e362c97e8ac2902c6b961d36830 to generate the below outline.
 * -------------------
 * L2 Block Data specification
 * -------------------
 *
 *  | byte start                                                                       | num bytes    | name
 *  | ---                                                                              | ---          | ---
 *  | 0x0000                                                                           | 0x20         | chain-id
 *  | 0x0020                                                                           | 0x20         | version
 *  | 0x0040                                                                           | 0x20         | L2 block number
 *  | 0x0060                                                                           | 0x20         | L2 timestamp
 *  | 0x0080                                                                           | 0x20         | startPrivateDataTreeSnapshot.root
 *  | 0x00a0                                                                           | 0x04         | startPrivateDataTreeSnapshot.nextAvailableLeafIndex
 *  | 0x00a4                                                                           | 0x20         | startNullifierTreeSnapshot.root
 *  | 0x00c4                                                                           | 0x04         | startNullifierTreeSnapshot.nextAvailableLeafIndex
 *  | 0x00c8                                                                           | 0x20         | startContractTreeSnapshot.root
 *  | 0x00e8                                                                           | 0x04         | startContractTreeSnapshot.nextAvailableLeafIndex
 *  | 0x00ec                                                                           | 0x20         | startPublicDataTreeRoot
 *  | 0x010c                                                                           | 0x20         | startL1ToL2MessageTreeSnapshot.root
 *  | 0x012c                                                                           | 0x04         | startL1ToL2MessageTreeSnapshot.nextAvailableLeafIndex
 *  | 0x0130                                                                           | 0x20         | startHistoricBlocksTreeSnapshot.root
 *  | 0x0150                                                                           | 0x04         | startHistoricBlocksTreeSnapshot.nextAvailableLeafIndex
 *  | 0x0154                                                                           | 0x20         | endPrivateDataTreeSnapshot.root
 *  | 0x0174                                                                           | 0x04         | endPrivateDataTreeSnapshot.nextAvailableLeafIndex
 *  | 0x0178                                                                           | 0x20         | endNullifierTreeSnapshot.root
 *  | 0x0198                                                                           | 0x04         | endNullifierTreeSnapshot.nextAvailableLeafIndex
 *  | 0x019c                                                                           | 0x20         | endContractTreeSnapshot.root
 *  | 0x01bc                                                                           | 0x04         | endContractTreeSnapshot.nextAvailableLeafIndex
 *  | 0x01c0                                                                           | 0x20         | endPublicDataTreeRoot
 *  | 0x01e0                                                                           | 0x20         | endL1ToL2MessageTreeSnapshot.root
 *  | 0x0200                                                                           | 0x04         | endL1ToL2MessageTreeSnapshot.nextAvailableLeafIndex
 *  | 0x0204                                                                           | 0x20         | endHistoricBlocksTreeSnapshot.root
 *  | 0x0224                                                                           | 0x04         | endHistoricBlocksTreeSnapshot.nextAvailableLeafIndex
 *  | 0x0228                                                                           | 0x04         | len(newCommitments) (denoted a)
 *  | 0x022c                                                                           | a * 0x20     | newCommitments
 *  | 0x022c + a * 0x20                                                                | 0x04         | len(newNullifiers) (denoted b)
 *  | 0x0230 + a * 0x20                                                                | b * 0x20     | newNullifiers
 *  | 0x0230 + a * 0x20 + b * 0x20                                                     | 0x04         | len(newPublicDataWrites) (denoted c)
 *  | 0x0234 + a * 0x20 + b * 0x20                                                     | c * 0x40     | newPublicDataWrites
 *  | 0x0234 + a * 0x20 + b * 0x20 + c * 0x40                                          | 0x04         | len(newL2ToL1Msgs) (denoted d)
 *  | 0x0238 + a * 0x20 + b * 0x20 + c * 0x40                                          | d * 0x20     | newL2ToL1Msgs
 *  | 0x0238 + a * 0x20 + b * 0x20 + c * 0x40 + d * 0x20                               | 0x04         | len(contracts) (denoted e)
 *  | 0x023c + a * 0x20 + b * 0x20 + c * 0x40 + d * 0x20                               | e * 0x20     | newContracts
 *  | 0x023c + a * 0x20 + b * 0x20 + c * 0x40 + d * 0x20 + e * 0x20                    | e * 0x34     | newContractsData
 *  | 0x023c + a * 0x20 + b * 0x20 + c * 0x40 + d * 0x20 + e * 0x54                    | 0x04         | len(newL1ToL2Msgs) (denoted f)
 *  | 0x0240 + a * 0x20 + b * 0x20 + c * 0x40 + d * 0x20 + e * 0x54                    | f * 0x20     | newL1ToL2Msgs
 *  | 0x0240 + a * 0x20 + b * 0x20 + c * 0x40 + d * 0x20 + e * 0x54 + f * 0x20         | 0x04         | byteLen(newEncryptedLogs) (denoted g)
 *  | 0x0244 + a * 0x20 + b * 0x20 + c * 0x40 + d * 0x20 + e * 0x54 + f * 0x20         | g            | newEncryptedLogs
 *  | 0x0244 + a * 0x20 + b * 0x20 + c * 0x40 + d * 0x20 + e * 0x54 + f * 0x20 + g     | 0x04         | byteLen(newUnencryptedLogs) (denoted h)
 *  | 0x0248 + a * 0x20 + b * 0x20 + c * 0x40 + d * 0x20 + e * 0x54 + f * 0x20 + g     | h            | newUnencryptedLogs
 *  | ---                                                                              | ---          | ---
 */
library Decoder {
  struct ArrayOffsets {
    uint256 commitment;
    uint256 nullifier;
    uint256 publicData;
    uint256 l2ToL1Msgs;
    uint256 contracts;
    uint256 contractData;
    uint256 l1ToL2Msgs;
    uint256 encryptedLogs;
    uint256 unencryptedLogs;
  }

  // Note: Used in `computeConsumables` to get around stack too deep errors.
  struct ConsumablesVars {
    bytes32[] baseLeaves;
    bytes32[] l2ToL1Msgs;
    bytes baseLeaf;
    bytes32 encryptedLogsHashKernel1;
    bytes32 encryptedLogsHashKernel2;
    bytes32 unencryptedLogsHashKernel1;
    bytes32 unencryptedLogsHashKernel2;
    uint256 l1Tol2MsgsCount;
  }

  // DECODING OFFSET CONSTANTS
  // Where the start of trees metadata begins in the block
  uint256 private constant START_TREES_BLOCK_HEADER_OFFSET = 0x80;

  // The size of the block header elements
  uint256 private constant TREES_BLOCK_HEADER_SIZE = 0xd4;

  // Where the end of trees metadata begns in the block
  uint256 private constant END_TREES_BLOCK_HEADER_OFFSET =
    START_TREES_BLOCK_HEADER_OFFSET + TREES_BLOCK_HEADER_SIZE;

  // Where the metadata ends and the block data begins.
  uint256 private constant BLOCK_HEADER_OFFSET =
    START_TREES_BLOCK_HEADER_OFFSET + 2 * TREES_BLOCK_HEADER_SIZE;

  /**
   * @notice Decodes the inputs and computes values to check state against
   * @param _l2Block - The L2 block calldata.
   * @return l2BlockNumber  - The L2 block number.
   * @return startStateHash - The state hash expected prior the execution.
   * @return endStateHash - The state hash expected after the execution.
   * @return publicInputHash - The hash of the public inputs
   * @return l2ToL1Msgs - The L2 to L1 messages
   * @return l1ToL2Msgs - The L1 to L2 messages
   */
  function decode(bytes calldata _l2Block)
    internal
    pure
    returns (
      uint256 l2BlockNumber,
      bytes32 startStateHash,
      bytes32 endStateHash,
      bytes32 publicInputHash,
      bytes32[] memory l2ToL1Msgs,
      bytes32[] memory l1ToL2Msgs
    )
  {
    l2BlockNumber = getL2BlockNumber(_l2Block);
    // Note, for startStateHash to match the storage, the l2 block number must be new - 1.
    // Only jumping 1 block at a time.
    startStateHash = computeStateHash(l2BlockNumber - 1, START_TREES_BLOCK_HEADER_OFFSET, _l2Block);
    endStateHash = computeStateHash(l2BlockNumber, END_TREES_BLOCK_HEADER_OFFSET, _l2Block);

    bytes32 diffRoot;
    bytes32 l1ToL2MsgsHash;
    (diffRoot, l1ToL2MsgsHash, l2ToL1Msgs, l1ToL2Msgs) = computeConsumables(_l2Block);
    publicInputHash = computePublicInputHash(_l2Block, diffRoot, l1ToL2MsgsHash);
  }

  /**
   * @notice Computes the public input hash
   * @dev Uses sha256 to field
   * @param _l2Block - The L2 block calldata.
   * @param _diffRoot - The root of the diff merkle tree
   * @param _l1ToL2MsgsHash - The hash of the L1 to L2 messages
   * @return publicInputHash - The hash of the public inputs (sha256 to field)
   */
  function computePublicInputHash(
    bytes calldata _l2Block,
    bytes32 _diffRoot,
    bytes32 _l1ToL2MsgsHash
  ) internal pure returns (bytes32) {
    return
      Hash.sha256ToField(bytes.concat(_l2Block[:BLOCK_HEADER_OFFSET], _diffRoot, _l1ToL2MsgsHash));
  }

  /**
   * @notice Extract the L2 block number from the block
   * @param _l2Block - The L2 block calldata
   * @return l2BlockNumber - The L2 block number
   */
  function getL2BlockNumber(bytes calldata _l2Block) internal pure returns (uint256 l2BlockNumber) {
    return uint256(bytes32(_l2Block[0x40:0x60]));
  }

  /**
   * @notice Computes a state hash
   * @param _l2BlockNumber - The L2 block number
   * @param _offset - The offset into the data, 0x80 for start, 0x019c for end
   * @param _l2Block - The L2 block calldata.
   * @return The state hash
   * @dev The state hash is sha256 hash of block's header elements. For each block the header elements are
   *      the block number, snapshots of all the trees and the root of the public data tree. This function
   *      copies all of these to memory and then hashes them.
   */
  function computeStateHash(uint256 _l2BlockNumber, uint256 _offset, bytes calldata _l2Block)
    internal
    pure
    returns (bytes32)
  {
    return sha256(
      bytes.concat(bytes32(_l2BlockNumber), slice(_l2Block, _offset, TREES_BLOCK_HEADER_SIZE))
    );
  }

  /**
   * @notice Computes consumables for the block
   * @param _l2Block - The L2 block calldata.
   * @return diffRoot - The root of the diff tree (new commitments, nullifiers etc)
   * @return l1ToL2MsgsHash - The hash of the L1 to L2 messages
   * @return l2ToL1Msgs - The L2 to L1 messages of the block
   * @return l1ToL2Msgs - The L1 to L2 messages of the block
   */
  function computeConsumables(bytes calldata _l2Block)
    internal
    pure
    returns (bytes32, bytes32, bytes32[] memory, bytes32[] memory)
  {
    ArrayOffsets memory offsets;
    ConsumablesVars memory vars;

    {
      uint256 offset = BLOCK_HEADER_OFFSET;

      // Commitments
      uint256 count = read4(_l2Block, offset);
      vars.baseLeaves = new bytes32[](count / (Constants.MAX_NEW_COMMITMENTS_PER_TX * 2));
      offsets.commitment = BLOCK_HEADER_OFFSET + 0x4;
      offset += 0x4 + count * 0x20;
      offsets.nullifier = offset + 0x4; // + 0x4 to offset by next read4

      // Nullifiers
      count = read4(_l2Block, offset);
      offset += 0x4 + count * 0x20;
      offsets.publicData = offset + 0x4; // + 0x4 to offset by next read4

      // Public data writes
      count = read4(_l2Block, offset);
      offset += 0x4 + count * 0x40;
      offsets.l2ToL1Msgs = offset + 0x4; // + 0x4 to offset by next read4

      // L2 to L1 messages
      count = read4(_l2Block, offset);
      vars.l2ToL1Msgs = new bytes32[](count);
      assembly {
        // load the l2 to l1 msgs (done here as offset will be altered in loop)
        let l2ToL1Msgs := mload(add(vars, 0x20))
        calldatacopy(
          add(l2ToL1Msgs, 0x20), add(_l2Block.offset, mload(add(offsets, 0x60))), mul(count, 0x20)
        )
      }
      offset += 0x4 + count * 0x20;
      offsets.contracts = offset + 0x4; // + 0x4 to offset by next read4

      // Contracts
      count = read4(_l2Block, offset);
      offsets.contractData = offsets.contracts + count * 0x20;
      offset += 0x4 + count * 0x54;
      offsets.l1ToL2Msgs = offset + 0x4; // + 0x4 to offset by next read4

      // L1 to L2 messages
      count = read4(_l2Block, offset);
      vars.l1Tol2MsgsCount = count;
      offset += 0x4 + count * 0x20;
      offsets.encryptedLogs = offset + 0x4; // + 0x4 to offset by next read4

      // Used as length in bytes down here
      uint256 length = read4(_l2Block, offset);
      offsets.unencryptedLogs = offsets.encryptedLogs + 0x4 + length;
    }

    // Data starts after header. Look at L2 Block Data specification at the top of this file.
    {
      for (uint256 i = 0; i < vars.baseLeaves.length; i++) {
        /*
         * Compute the leaf to insert.
         * Leaf_i = (
         *    newCommitmentsKernel1,
         *    newCommitmentsKernel2,
         *    newNullifiersKernel1,
         *    newNullifiersKernel2,
         *    newPublicDataWritesKernel1,
         *    newPublicDataWritesKernel2,
         *    newL2ToL1MsgsKernel1,
         *    newL2ToL1MsgsKernel2,
         *    newContractLeafKernel1,
         *    newContractLeafKernel2,
         *    newContractDataKernel1.aztecAddress,
         *    newContractDataKernel1.ethAddress (padded to 32 bytes),
         *    newContractDataKernel2.aztecAddress,
         *    newContractDataKernel2.ethAddress (padded to 32 bytes), ____
         *    encryptedLogsHashKernel1,                                   |
         *    encryptedLogsHashKernel2,                                   |=> Computed below from logs' preimages.
         *    unencryptedLogsHashKernel1,                                |
         *    unencryptedLogsHashKernel2                              ___|
         * );
         * Note that we always read data, the l2Block (atm) must therefore include dummy or zero-notes for
         * Zero values.
         */

        /**
         * Compute encrypted and unencrypted logs hashes corresponding to the current leaf.
         * Note: will advance offsets by the number of bytes processed.
         */
        (vars.encryptedLogsHashKernel1, offsets.encryptedLogs) =
          computeKernelLogsHash(offsets.encryptedLogs, _l2Block);
        (vars.encryptedLogsHashKernel2, offsets.encryptedLogs) =
          computeKernelLogsHash(offsets.encryptedLogs, _l2Block);

        (vars.unencryptedLogsHashKernel1, offsets.unencryptedLogs) =
          computeKernelLogsHash(offsets.unencryptedLogs, _l2Block);
        (vars.unencryptedLogsHashKernel2, offsets.unencryptedLogs) =
          computeKernelLogsHash(offsets.unencryptedLogs, _l2Block);

        // Insertions are split into multiple `bytes.concat` to work around stack too deep.
        vars.baseLeaf = bytes.concat(
          bytes.concat(
            slice(_l2Block, offsets.commitment, Constants.COMMITMENTS_NUM_BYTES_PER_BASE_ROLLUP),
            slice(_l2Block, offsets.nullifier, Constants.NULLIFIERS_NUM_BYTES_PER_BASE_ROLLUP),
            slice(
              _l2Block, offsets.publicData, Constants.PUBLIC_DATA_WRITES_NUM_BYTES_PER_BASE_ROLLUP
            ),
            slice(_l2Block, offsets.l2ToL1Msgs, Constants.L2_TO_L1_MSGS_NUM_BYTES_PER_BASE_ROLLUP),
            slice(_l2Block, offsets.contracts, Constants.CONTRACTS_NUM_BYTES_PER_BASE_ROLLUP)
          ),
          bytes.concat(
            slice(_l2Block, offsets.contractData, 0x20), // newContractDataKernel1.aztecAddress
            bytes12(0),
            slice(_l2Block, offsets.contractData + 0x20, 0x14), // newContractDataKernel1.ethAddress
            slice(_l2Block, offsets.contractData + 0x34, 0x20), // newContractDataKernel2.aztecAddress
            bytes12(0),
            slice(_l2Block, offsets.contractData + 0x54, 0x14) // newContractDataKernel2.ethAddress
          ),
          bytes.concat(
            vars.encryptedLogsHashKernel1,
            vars.encryptedLogsHashKernel2,
            vars.unencryptedLogsHashKernel1,
            vars.unencryptedLogsHashKernel2
          )
        );

        offsets.commitment += Constants.COMMITMENTS_NUM_BYTES_PER_BASE_ROLLUP;
        offsets.nullifier += Constants.NULLIFIERS_NUM_BYTES_PER_BASE_ROLLUP;
        offsets.publicData += Constants.PUBLIC_DATA_WRITES_NUM_BYTES_PER_BASE_ROLLUP;
        offsets.l2ToL1Msgs += Constants.L2_TO_L1_MSGS_NUM_BYTES_PER_BASE_ROLLUP;
        offsets.contracts += Constants.CONTRACTS_NUM_BYTES_PER_BASE_ROLLUP;
        offsets.contractData += Constants.CONTRACT_DATA_NUM_BYTES_PER_BASE_ROLLUP_UNPADDED;

        vars.baseLeaves[i] = sha256(vars.baseLeaf);
      }
    }

    bytes32 diffRoot = computeRoot(vars.baseLeaves);
    bytes32[] memory l1ToL2Msgs;
    bytes32 l1ToL2MsgsHash;
    {
      // `l1ToL2Msgs` is fixed size so if `lengths.l1Tol2MsgsCount` < `Constants.NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP` the array
      // will contain some zero values.
      uint256 l1ToL2MsgsHashPreimageSize = 0x20 * vars.l1Tol2MsgsCount;
      l1ToL2Msgs = new bytes32[](Constants.NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP);
      assembly {
        calldatacopy(
          add(l1ToL2Msgs, 0x20),
          add(_l2Block.offset, mload(add(offsets, 0xc0))),
          l1ToL2MsgsHashPreimageSize
        )
      }

      l1ToL2MsgsHash = sha256(abi.encodePacked(l1ToL2Msgs));
    }

    return (diffRoot, l1ToL2MsgsHash, vars.l2ToL1Msgs, l1ToL2Msgs);
  }

  /**
   * @notice Computes logs hash as is done in the kernel and app circuits.
   * @param _offsetInBlock - The offset of kernel's logs in a block.
   * @param _l2Block - The L2 block calldata.
   * @return The hash of the logs and offset in a block after processing the logs.
   * @dev We have logs preimages on the input and we need to perform the same hashing process as is done in the app
   *      circuit (hashing the logs) and in the kernel circuit (accumulating the logs hashes). In each iteration of
   *      kernel, the kernel computes a hash of the previous iteration's logs hash (the hash in the previous kernel's
   *      public inputs) and the the current iteration private circuit public inputs logs hash.
   *
   *      E.g. for resulting logs hash of a kernel with 3 iterations would be computed as:
   *
   *        kernelPublicInputsLogsHash = sha256(sha256(sha256(I1_LOGS), sha256(I2_LOGS)), sha256(I3_LOGS))
   *
   *      where I1_LOGS, I2_LOGS and I3_LOGS are logs emitted in the first, second and third function call.
   *
   *      Note that `sha256(I1_LOGS)`, `sha256(I2_LOGS)` and `sha256(I3_LOGS)` are computed in the app circuit and not
   *      in the kernel circuit. The kernel circuit only accumulates the hashes.
   *
   * @dev For the example above, the logs are encoded in the following way:
   *
   *        || K_LOGS_LEN | I1_LOGS_LEN | I1_LOGS | I2_LOGS_LEN | I2_LOGS | I3_LOGS_LEN | I3_LOGS ||
   *           4 bytes      4 bytes       i bytes   4 bytes       j bytes     4 bytes     k bytes
   *
   *        K_LOGS_LEN is the total length of the logs in the kernel.
   *        I1_LOGS_LEN (i) is the length of the logs in the first iteration.
   *        I1_LOGS are all the logs emitted in the first iteration.
   *        I2_LOGS_LEN (j) ...
   *
   * @dev Link to a relevant discussion:
   *      https://discourse.aztec.network/t/proposal-forcing-the-sequencer-to-actually-submit-data-to-l1/426/9
   */
  function computeKernelLogsHash(uint256 _offsetInBlock, bytes calldata _l2Block)
    internal
    pure
    returns (bytes32, uint256)
  {
    uint256 offset = _offsetInBlock;
    uint256 remainingLogsLength = read4(_l2Block, offset);
    offset += 0x4;

    bytes32 kernelPublicInputsLogsHash; // The hash on the output of kernel iteration

    // Iterate until all the logs were processed
    while (remainingLogsLength > 0) {
      // The length of the logs emitted by Noir from the function call corresponding to this kernel iteration
      uint256 privateCircuitPublicInputLogsLength = read4(_l2Block, offset);
      offset += 0x4;

      // Hash the logs of this iteration's function call
      bytes32 privateCircuitPublicInputsLogsHash =
        sha256(slice(_l2Block, offset, privateCircuitPublicInputLogsLength));
      offset += privateCircuitPublicInputLogsLength;

      // Decrease remaining logs length by this privateCircuitPublicInputsLogs's length (len(I?_LOGS)) and 4 bytes for I?_LOGS_LEN
      remainingLogsLength -= (privateCircuitPublicInputLogsLength + 0x4);

      kernelPublicInputsLogsHash =
        sha256(bytes.concat(kernelPublicInputsLogsHash, privateCircuitPublicInputsLogsHash));
    }

    return (kernelPublicInputsLogsHash, offset);
  }

  /**
   * @notice Computes the root for a binary Merkle-tree given the leafs.
   * @dev Uses sha256.
   * @param _leafs - The 32 bytes leafs to build the tree of.
   * @return The root of the Merkle tree.
   */
  function computeRoot(bytes32[] memory _leafs) internal pure returns (bytes32) {
    // @todo Must pad the tree
    uint256 treeDepth = 0;
    while (2 ** treeDepth < _leafs.length) {
      treeDepth++;
    }
    uint256 treeSize = 2 ** treeDepth;
    assembly {
      mstore(_leafs, treeSize)
    }

    for (uint256 i = 0; i < treeDepth; i++) {
      for (uint256 j = 0; j < treeSize; j += 2) {
        _leafs[j / 2] = sha256(bytes.concat(_leafs[j], _leafs[j + 1]));
      }
    }

    return _leafs[0];
  }

  /**
   * @notice Wrapper around the slicing to avoid some stack too deep
   * @param _data - The data to slice
   * @param _start - The start of the slice
   * @param _length - The length of the slice
   * @return The slice
   */
  function slice(bytes calldata _data, uint256 _start, uint256 _length)
    internal
    pure
    returns (bytes memory)
  {
    return _data[_start:_start + _length];
  }

  /**
   * @notice Reads 4 bytes from the data
   * @param _data - The data to read from
   * @param _offset - The offset to read from
   * @return The 4 bytes read as a uint256
   */
  function read4(bytes calldata _data, uint256 _offset) internal pure returns (uint256) {
    return uint256(uint32(bytes4(slice(_data, _offset, 4))));
  }
}
