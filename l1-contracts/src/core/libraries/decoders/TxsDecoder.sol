// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

// Libraries
import {Constants} from "../ConstantsGen.sol";
import {Hash} from "../Hash.sol";

/**
 * @title Txs Decoder Library
 * @author Aztec Labs
 * @notice Decoding a L2 block body and computing the TxsHash.
 * Concerned with readability and velocity of development not giving a damn about gas costs.
 * @dev Assumes the input trees to be padded.
 *
 * -------------------
 * You can use https://gist.github.com/LHerskind/724a7e362c97e8ac2902c6b961d36830 to generate the below outline.
 * -------------------
 * L2 Body Data Specification
 * -------------------
 *  | byte start                                                                                                                | num bytes  | name
 *  | ---                                                                                                                       | ---        | ---
 *  | 0x0                                                                                                                       | 0x4        | len(newL1ToL2Msgs) (denoted a)
 *  | 0x4                                                                                                                       | a * 0x20   | newL1ToL2Msgs
 *  | 0x4 + a * 0x20 = tx0Start                                                                                                 | 0x4        | len(numTxs) (denoted t)
 *  |                                                                                                                           |            | TxEffect 0 {
 *  | tx0Start                                                                                                                  | 0x1        |   len(newNoteHashes) (denoted b)
 *  | tx0Start + 0x1                                                                                                            | b * 0x20   |   newNoteHashes
 *  | tx0Start + 0x1 + b * 0x20                                                                                                 | 0x1        |   len(newNullifiers) (denoted c)
 *  | tx0Start + 0x1 + b * 0x20 + 0x1                                                                                           | c * 0x20   |   newNullifiers
 *  | tx0Start + 0x1 + b * 0x20 + 0x1 + c * 0x20                                                                                | 0x1        |   len(newL2ToL1Msgs) (denoted d)
 *  | tx0Start + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1                                                                          | d * 0x20   |   newL2ToL1Msgs
 *  | tx0Start + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1 + d * 0x20                                                               | 0x1        |   len(newPublicDataWrites) (denoted e)
 *  | tx0Start + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1 + d * 0x20 + 0x01                                                        | e * 0x40   |   newPublicDataWrites
 *  | tx0Start + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1 + d * 0x20 + 0x01 + e * 0x40                                             | 0x1        |   len(contracts) (denoted f)
 *  | tx0Start + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1 + d * 0x20 + 0x01 + e * 0x40 + 0x1                                       | f * 0x20   |   newContracts
 *  | tx0Start + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1 + d * 0x20 + 0x01 + e * 0x40 + 0x1 + f * 0x20                            | f * 0x34   |   newContractsData
 *  | tx0Start + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1 + d * 0x20 + 0x01 + e * 0x40 + 0x1 + f * 0x20 + f * 0x34                 | 0x04       |   byteLen(newEncryptedLogs) (denoted g)
 *  | tx0Start + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1 + d * 0x20 + 0x01 + e * 0x40 + 0x1 + f * 0x20 + f * 0x34 + 0x4           | g          |   newEncryptedLogs
 *  | tx0Start + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1 + d * 0x20 + 0x01 + e * 0x40 + 0x1 + f * 0x20 + f * 0x34 + 0x4 + g       | 0x04       |   byteLen(newUnencryptedLogs) (denoted h)
 *  | tx0Start + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1 + d * 0x20 + 0x01 + e * 0x40 + 0x1 + f * 0x20 + f * 0x34 + 0x4 + g + 0x4 | h          |   newUnencryptedLogs
 *  |                                                                                                                           |            | },
 *  |                                                                                                                           |            | TxEffect 1 {
 *  |                                                                                                                           |            |   ...
 *  |                                                                                                                           |            | },
 *  |                                                                                                                           |            | ...
 *  |                                                                                                                           |            | TxEffect (t - 1) {
 *  |                                                                                                                           |            |   ...
 *  |                                                                                                                           |            | },
 */
library TxsDecoder {
  struct ArrayOffsets {
    uint256 noteHash;
    uint256 nullifier;
    uint256 publicData;
    uint256 l2ToL1Msgs;
    uint256 contracts;
    uint256 contractData;
    uint256 l1ToL2Msgs;
  }

  struct Counts {
    uint256 noteHash;
    uint256 nullifier;
    uint256 l2ToL1Msgs;
    uint256 publicData;
    uint256 contracts;
    uint256 contractData;
  }

  // Note: Used in `computeConsumables` to get around stack too deep errors.
  struct ConsumablesVars {
    bytes32[] baseLeaves;
    bytes32[] l2ToL1Msgs;
    bytes baseLeaf;
    bytes32 encryptedLogsHash;
    bytes32 unencryptedLogsHash;
    uint256 l1Tol2MsgsCount;
  }

  /**
   * @notice Computes consumables for the block
   * @param _body - The L2 block calldata.
   * @return diffRoot - The root of the diff tree (new note hashes, nullifiers etc)
   */
  function decode(bytes calldata _body) internal pure returns (bytes32) {
    ArrayOffsets memory offsets;
    Counts memory counts;
    ConsumablesVars memory vars;
    uint256 offset = 0;

    {
      // L1 to L2 messages
      uint256 count = read4(_body, offset);
      vars.l1Tol2MsgsCount = count;
      offset += 0x4 + count * 0x20;

      count = read4(_body, offset); // number of tx effects
      offset += 0x4;
      vars.baseLeaves = new bytes32[](count);
    }

    // Data starts after header. Look at L2 Block Data specification at the top of this file.
    {
      for (uint256 i = 0; i < vars.baseLeaves.length; i++) {
        /*
         * Compute the leaf to insert.
         * Leaf_i = (
         *    newNoteHashesKernel,
         *    newNullifiersKernel,
         *    newPublicDataWritesKernel,
         *    newL2ToL1MsgsKernel,
         *    newContractLeafKernel,
         *    newContractDataKernel.aztecAddress,
         *    newContractDataKernel.ethAddress (padded to 32 bytes),
         *    encryptedLogsHash,                                   |
         *    unencryptedLogsHash,                             ____|=> Computed below from logs' preimages.
         * );
         * Note that we always read data, the l2Block (atm) must therefore include dummy or zero-notes for
         * Zero values.
         */

        // Note hashes
        uint256 count = read1(_body, offset);
        offset += 0x1;
        counts.noteHash = count;
        offsets.noteHash = offset;
        offset += count * 0x20; // each note hash is 0x20 bytes long

        // Nullifiers
        count = read1(_body, offset);
        offset += 0x1;
        counts.nullifier = count;
        offsets.nullifier = offset;
        offset += count * 0x20; // each nullifier is 0x20 bytes long

        // L2 to L1 messages
        count = read1(_body, offset);
        offset += 0x1;
        counts.l2ToL1Msgs = count;
        offsets.l2ToL1Msgs = offset;
        offset += count * 0x20; // each l2 to l1 message is 0x20 bytes long

        // Public data writes
        count = read1(_body, offset);
        offset += 0x1;
        counts.publicData = count;
        offsets.publicData = offset;
        offset += count * 0x40; // each public data write is 0x40 bytes long

        // Contracts
        count = read1(_body, offset);
        offset += 0x1;
        counts.contracts = count;
        offsets.contracts = offset;
        offset += count * 0x20; // each contract leaf is 0x20 bytes long

        // Contract data
        counts.contractData = count; // count is the same as contracts
        offsets.contractData = offset;
        offset += count * 0x34; // each contract data is 0x34 bytes long

        bytes memory contractData = new bytes(Constants.CONTRACT_DATA_NUM_BYTES_PER_BASE_ROLLUP);
        if (counts.contracts == 1) {
          contractData = bytes.concat(
            slice(_body, offsets.contractData, 0x20), // newContractDataKernel.aztecAddress
            bytes12(0), // We pad the ethAddress to 32 bytes, we don't use sliceAndPad here because we want to prefix
            slice(_body, offsets.contractData + 0x20, 0x14) // newContractDataKernel.ethAddress
          );
        }

        /**
         * Compute encrypted and unencrypted logs hashes corresponding to the current leaf.
         * Note: will advance offsets by the number of bytes processed.
         */
        (vars.encryptedLogsHash, offset) = computeKernelLogsHash(offset, _body);
        (vars.unencryptedLogsHash, offset) = computeKernelLogsHash(offset, _body);

        // Insertions are split into multiple `bytes.concat` to work around stack too deep.
        vars.baseLeaf = bytes.concat(
          bytes.concat(
            sliceAndPad(
              _body,
              offsets.noteHash,
              counts.noteHash * 0x20,
              Constants.NOTE_HASHES_NUM_BYTES_PER_BASE_ROLLUP
            ),
            sliceAndPad(
              _body,
              offsets.nullifier,
              counts.nullifier * 0x20,
              Constants.NULLIFIERS_NUM_BYTES_PER_BASE_ROLLUP
            ),
            sliceAndPad(
              _body,
              offsets.l2ToL1Msgs,
              counts.l2ToL1Msgs * 0x20,
              Constants.L2_TO_L1_MSGS_NUM_BYTES_PER_BASE_ROLLUP
            ),
            sliceAndPad(
              _body,
              offsets.publicData,
              counts.publicData * 0x40,
              Constants.PUBLIC_DATA_WRITES_NUM_BYTES_PER_BASE_ROLLUP
            ),
            sliceAndPad(
              _body,
              offsets.contracts,
              counts.contracts * 0x20,
              Constants.CONTRACTS_NUM_BYTES_PER_BASE_ROLLUP
            )
          ),
          contractData,
          bytes.concat(vars.encryptedLogsHash, vars.unencryptedLogsHash)
        );

        vars.baseLeaves[i] = sha256(vars.baseLeaf);
      }
    }

    return computeRoot(vars.baseLeaves);
  }

  /**
   * @notice Computes logs hash as is done in the kernel and app circuits.
   * @param _offsetInBlock - The offset of kernel's logs in a block.
   * @param _body - The L2 block calldata.
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
  function computeKernelLogsHash(uint256 _offsetInBlock, bytes calldata _body)
    internal
    pure
    returns (bytes32, uint256)
  {
    uint256 offset = _offsetInBlock;
    uint256 remainingLogsLength = read4(_body, offset);
    offset += 0x4;

    bytes32 kernelPublicInputsLogsHash; // The hash on the output of kernel iteration

    // Iterate until all the logs were processed
    while (remainingLogsLength > 0) {
      // The length of the logs emitted by Aztec.nr from the function call corresponding to this kernel iteration
      uint256 privateCircuitPublicInputLogsLength = read4(_body, offset);
      offset += 0x4;

      // Hash the logs of this iteration's function call
      bytes32 privateCircuitPublicInputsLogsHash =
        sha256(slice(_body, offset, privateCircuitPublicInputLogsLength));
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
      treeSize /= 2;
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
   * @notice Wrapper around the slicing and padding to avoid some stack too deep
   * @param _data - The data to slice
   * @param _start - The start of the slice
   * @param _length - The length of the slice
   * @param _targetLength - The length of the padded array
   * @return The slice
   */
  function sliceAndPad(bytes calldata _data, uint256 _start, uint256 _length, uint256 _targetLength)
    internal
    pure
    returns (bytes memory)
  {
    return bytes.concat(_data[_start:_start + _length], new bytes(_targetLength - _length));
  }

  /**
   * @notice Reads 1 bytes from the data
   * @param _data - The data to read from
   * @param _offset - The offset to read from
   * @return The 1 byte as a uint256
   */
  function read1(bytes calldata _data, uint256 _offset) internal pure returns (uint256) {
    return uint256(uint8(bytes1(slice(_data, _offset, 1))));
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
