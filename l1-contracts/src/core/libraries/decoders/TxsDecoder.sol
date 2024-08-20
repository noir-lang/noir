// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

// Libraries
import {Errors} from "../Errors.sol";
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
 * L2 Body Data Specification
 * -------------------
 *  | byte start                                                                                          | num bytes  | name
 *  | ---                                                                                                 | ---        | ---
 *  | 0x0                                                                                                 | 0x4        | len(numTxs) (denoted t)
 *  |                                                                                                     |            | TxEffect 0 {
 *  | 0x4                                                                                                 | 0x1        |   revertCode
 *  | 0x5                                                                                                 | 0x20       |   transactionFee
 *  | 0x25                                                                                                | 0x1        |   len(noteHashes) (denoted b)
 *  | 0x25 + 0x1                                                                                          | b * 0x20   |   noteHashes
 *  | 0x25 + 0x1 + b * 0x20                                                                               | 0x1        |   len(nullifiers) (denoted c)
 *  | 0x25 + 0x1 + b * 0x20 + 0x1                                                                         | c * 0x20   |   nullifiers
 *  | 0x25 + 0x1 + b * 0x20 + 0x1 + c * 0x20                                                              | 0x1        |   len(l2ToL1Msgs) (denoted d)
 *  | 0x25 + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1                                                        | d * 0x20   |   l2ToL1Msgs
 *  | 0x25 + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1 + d * 0x20                                             | 0x1        |   len(publicDataUpdateRequests) (denoted e)
 *  | 0x25 + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1 + d * 0x20 + 0x01                                      | e * 0x40   |   publicDataUpdateRequests
 *  | 0x25 + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1 + d * 0x20 + 0x01 + e * 0x40                           | 0x04       |   byteLen(noteEncryptedLogs) (denoted f)
 *  | 0x25 + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1 + d * 0x20 + 0x01 + e * 0x40 + 0x4                     | f          |   noteEncryptedLogs
 *  | 0x25 + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1 + d * 0x20 + 0x01 + e * 0x40 + 0x4 + f                 | 0x04       |   byteLen(encryptedLogs) (denoted g)
 *  | 0x25 + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1 + d * 0x20 + 0x01 + e * 0x40 + 0x4 + f + 0x4           | g          |   encryptedLogs
 *  | 0x25 + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1 + d * 0x20 + 0x01 + e * 0x40 + 0x4 + f + 0x4 + g       | 0x04       |   byteLen(unencryptedLogs) (denoted h)
 *  | 0x25 + 0x1 + b * 0x20 + 0x1 + c * 0x20 + 0x1 + d * 0x20 + 0x01 + e * 0x40 + 0x4 + f + 0x4 + g + 0x04| h          |   unencryptedLogs
 *  |                                                                                                     |            | },
 *  |                                                                                                     |            | TxEffect 1 {
 *  |                                                                                                     |            |   ...
 *  |                                                                                                     |            | },
 *  |                                                                                                     |            | ...
 *  |                                                                                                     |            | TxEffect (t - 1) {
 *  |                                                                                                     |            |   ...
 *  |                                                                                                     |            | },
 */
library TxsDecoder {
  struct ArrayOffsets {
    uint256 revertCode;
    uint256 transactionFee;
    uint256 noteHash;
    uint256 nullifier;
    uint256 l2ToL1Msgs;
    uint256 publicData;
    uint256 noteEncryptedLogsLength;
    uint256 encryptedLogsLength;
    uint256 unencryptedLogsLength;
  }

  struct Counts {
    uint256 noteHash;
    uint256 nullifier;
    uint256 l2ToL1Msgs;
    uint256 publicData;
  }

  // Note: Used in `computeConsumables` to get around stack too deep errors.
  struct ConsumablesVars {
    bytes32[] baseLeaves;
    bytes baseLeaf;
    uint256 kernelNoteEncryptedLogsLength;
    uint256 kernelEncryptedLogsLength;
    uint256 kernelUnencryptedLogsLength;
    bytes32 noteEncryptedLogsHash;
    bytes32 encryptedLogsHash;
    bytes32 unencryptedLogsHash;
    bytes32 txOutHash;
  }

  /**
   * @notice Computes txs effects hash
   * @param _body - The L2 block body calldata.
   * @return The txs effects hash.
   */
  function decode(bytes calldata _body) internal pure returns (bytes32) {
    ArrayOffsets memory offsets;
    Counts memory counts;
    ConsumablesVars memory vars;
    uint256 offset = 0;

    uint32 numTxEffects = uint32(read4(_body, offset));
    uint256 numTxEffectsToPad = computeNumTxEffectsToPad(numTxEffects);

    offset += 0x4;
    vars.baseLeaves = new bytes32[](numTxEffects + numTxEffectsToPad);

    // Data starts after header. Look at L2 Block Data specification at the top of this file.
    {
      for (uint256 i = 0; i < numTxEffects; i++) {
        /*
         * Compute the leaf to insert.
         * Leaf_i = (
         *    revertCode,
         *    transactionFee,
         *    noteHashesKernel,
         *    nullifiersKernel,
         *    txOutHash,                                           |=> Computed below from l2tol1msgs
         *    publicDataUpdateRequestsKernel,
         *    noteEncryptedLogsLength,
         *    encryptedLogsLength,
         *    unencryptedLogsLength,
         *    noteEncryptedLogsHash,                               |
         *    encryptedLogsHash,                                   |
         *    unencryptedLogsHash,                             ____|=> Computed below from logs' preimages.
         * );
         * Note that we always read data, the l2Block (atm) must therefore include dummy or zero-notes for
         * Zero values.
         */

        // REVERT CODE
        offsets.revertCode = offset;
        offset += 0x1;

        // TX FEE
        offsets.transactionFee = offset;
        offset += 0x20;

        // NOTE HASHES
        uint256 count = read1(_body, offset);
        offset += 0x1;
        counts.noteHash = count;
        offsets.noteHash = offset;
        offset += count * 0x20; // each note hash is 0x20 bytes long

        // NULLIFIERS
        count = read1(_body, offset);
        offset += 0x1;
        counts.nullifier = count;
        offsets.nullifier = offset;
        offset += count * 0x20; // each nullifier is 0x20 bytes long

        // L2 TO L1 MESSAGES
        count = read1(_body, offset);
        offset += 0x1;
        counts.l2ToL1Msgs = count;
        offsets.l2ToL1Msgs = offset;
        offset += count * 0x20; // each l2 to l1 message is 0x20 bytes long

        // PUBLIC DATA UPDATE REQUESTS
        count = read1(_body, offset);
        offset += 0x1;
        counts.publicData = count;
        offsets.publicData = offset;
        offset += count * 0x40; // each public data update request is 0x40 bytes long

        // NOTE ENCRYPTED LOGS LENGTH
        offsets.noteEncryptedLogsLength = offset;
        offset += 0x20;

        // ENCRYPTED LOGS LENGTH
        offsets.encryptedLogsLength = offset;
        offset += 0x20;

        // UNENCRYPTED LOGS LENGTH
        offsets.unencryptedLogsLength = offset;
        offset += 0x20;

        /**
         * Compute note, encrypted and unencrypted logs hashes corresponding to the current leaf.
         * Note: will advance offsets by the number of bytes processed.
         */
        // NOTE ENCRYPTED LOGS HASH
        (vars.noteEncryptedLogsHash, offset, vars.kernelNoteEncryptedLogsLength) =
          computeKernelNoteEncryptedLogsHash(offset, _body);
        // ENCRYPTED LOGS HASH
        (vars.encryptedLogsHash, offset, vars.kernelEncryptedLogsLength) =
          computeKernelEncryptedLogsHash(offset, _body);
        // UNENCRYPTED LOGS HASH
        (vars.unencryptedLogsHash, offset, vars.kernelUnencryptedLogsLength) =
          computeKernelUnencryptedLogsHash(offset, _body);
        // TX LEVEL OUT HASH
        (vars.txOutHash) = computeTxOutHash(offsets.l2ToL1Msgs, _body);

        // We throw to ensure that the byte len we charge for DA gas in the kernels matches the actual chargable log byte len
        // Without this check, the user may provide the kernels with a lower log len than reality
        if (
          uint256(bytes32(slice(_body, offsets.noteEncryptedLogsLength, 0x20)))
            != vars.kernelNoteEncryptedLogsLength
        ) {
          revert Errors.TxsDecoder__InvalidLogsLength(
            uint256(bytes32(slice(_body, offsets.noteEncryptedLogsLength, 0x20))),
            vars.kernelNoteEncryptedLogsLength
          );
        }
        if (
          uint256(bytes32(slice(_body, offsets.encryptedLogsLength, 0x20)))
            != vars.kernelEncryptedLogsLength
        ) {
          revert Errors.TxsDecoder__InvalidLogsLength(
            uint256(bytes32(slice(_body, offsets.encryptedLogsLength, 0x20))),
            vars.kernelEncryptedLogsLength
          );
        }
        if (
          uint256(bytes32(slice(_body, offsets.unencryptedLogsLength, 0x20)))
            != vars.kernelUnencryptedLogsLength
        ) {
          revert Errors.TxsDecoder__InvalidLogsLength(
            uint256(bytes32(slice(_body, offsets.unencryptedLogsLength, 0x20))),
            vars.kernelUnencryptedLogsLength
          );
        }

        // Insertions are split into multiple `bytes.concat` to work around stack too deep.
        vars.baseLeaf = bytes.concat(
          // pad the revert code to 32 bytes to match the hash preimage
          bytes.concat(
            sliceAndPadLeft(_body, offsets.revertCode, 0x1, 0x20),
            slice(_body, offsets.transactionFee, 0x20)
          ),
          bytes.concat(
            sliceAndPadRight(
              _body,
              offsets.noteHash,
              counts.noteHash * 0x20,
              Constants.NOTE_HASHES_NUM_BYTES_PER_BASE_ROLLUP
            ),
            sliceAndPadRight(
              _body,
              offsets.nullifier,
              counts.nullifier * 0x20,
              Constants.NULLIFIERS_NUM_BYTES_PER_BASE_ROLLUP
            ),
            vars.txOutHash,
            sliceAndPadRight(
              _body,
              offsets.publicData,
              counts.publicData * 0x40,
              Constants.PUBLIC_DATA_WRITES_NUM_BYTES_PER_BASE_ROLLUP
            )
          ),
          bytes.concat(
            slice(_body, offsets.noteEncryptedLogsLength, 0x20),
            slice(_body, offsets.encryptedLogsLength, 0x20),
            slice(_body, offsets.unencryptedLogsLength, 0x20)
          ),
          bytes.concat(vars.noteEncryptedLogsHash, vars.encryptedLogsHash, vars.unencryptedLogsHash)
        );

        vars.baseLeaves[i] = Hash.sha256ToField(vars.baseLeaf);
      }

      // We pad base leaves with hashes of empty tx effect.
      for (uint256 i = numTxEffects; i < vars.baseLeaves.length; i++) {
        // Value taken from tx_effect.test.ts "hash of empty tx effect matches snapshot" test case
        vars.baseLeaves[i] = hex"00f0aa51fc81f8242316fcf2cb3b28196241ed3fa26dd320a959bce6c529b270";
      }
    }

    return computeUnbalancedRoot(vars.baseLeaves);
  }

  /**
   * @notice Computes logs hash as is done in the kernel and app circuits.
   * @param _offsetInBlock - The offset of kernel's logs in a block.
   * @param _body - The L2 block calldata.
   * @return The hash of the logs and offset in a block after processing the logs.
   * @dev We have logs preimages on the input and we need to perform the same hashing process as is done in the app
   *      circuit (hashing the logs) and in the kernel circuit (accumulating the logs hashes). The tail kernel
   *      circuit flat hashes all the app log hashes.
   *
   *      E.g. for resulting logs hash of a kernel with 3 iterations would be computed as:
   *
   *        kernelPublicInputsLogsHash = sha256((sha256(I1_LOGS), sha256(I2_LOGS)), sha256(I3_LOGS))
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
   * @dev The circuit outputs a total logs len based on the byte length that the user pays DA gas for.
   *      In terms of the encoding above, this is the raw log length (i, j, or k) + 4 for each log.
   *      For the example above, kernelLogsLength = (i + 4) + (j + 4) + (k + 4). Since we already track
   *      the total remainingLogsLength, we just remove the bytes holding function logs length.
   *
   * @dev Link to a relevant discussion:
   *      https://discourse.aztec.network/t/proposal-forcing-the-sequencer-to-actually-submit-data-to-l1/426/9
   */
  function computeKernelNoteEncryptedLogsHash(uint256 _offsetInBlock, bytes calldata _body)
    internal
    pure
    returns (bytes32, uint256, uint256)
  {
    uint256 offset = _offsetInBlock;
    uint256 remainingLogsLength = read4(_body, offset);
    uint256 kernelLogsLength = remainingLogsLength;
    offset += 0x4;

    bytes memory flattenedLogHashes; // The hash input

    // Iterate until all the logs were processed
    while (remainingLogsLength > 0) {
      // The length of the logs emitted by Aztec.nr from the function call corresponding to this kernel iteration
      uint256 privateCircuitPublicInputLogsLength = read4(_body, offset);
      offset += 0x4;

      // Decrease remaining logs length by this privateCircuitPublicInputsLogs's length (len(I?_LOGS)) and 4 bytes for I?_LOGS_LEN
      remainingLogsLength -= (privateCircuitPublicInputLogsLength + 0x4);

      kernelLogsLength -= 0x4;

      while (privateCircuitPublicInputLogsLength > 0) {
        uint256 singleCallLogsLength = read4(_body, offset);
        offset += 0x4;

        bytes32 singleLogHash = Hash.sha256ToField(slice(_body, offset, singleCallLogsLength));
        offset += singleCallLogsLength;

        flattenedLogHashes = bytes.concat(flattenedLogHashes, singleLogHash);

        privateCircuitPublicInputLogsLength -= (singleCallLogsLength + 0x4);
      }
    }

    // Not having a 0 value hash for empty logs causes issues with empty txs used for padding.
    if (flattenedLogHashes.length == 0) {
      return (0, offset, 0);
    }

    // padded to MAX_LOGS * 32 bytes
    flattenedLogHashes = bytes.concat(
      flattenedLogHashes,
      new bytes(Constants.MAX_NOTE_ENCRYPTED_LOGS_PER_TX * 32 - flattenedLogHashes.length)
    );

    bytes32 kernelPublicInputsLogsHash = Hash.sha256ToField(flattenedLogHashes);

    return (kernelPublicInputsLogsHash, offset, kernelLogsLength);
  }

  /**
   * @notice Computes encrypted logs hash as is done in the kernel circuits.
   * @param _offsetInBlock - The offset of kernel's logs in a block.
   * @param _body - The L2 block calldata.
   * @return The hash of the logs and offset in a block after processing the logs.
   * @dev See above for full details. Non-note encrypted logs hashes are siloed with
   * their (hidden) contract address:
   * singleLogsHash = sha256ToField(encryptedBuffer)
   * siloedLogsHash = sha256ToField(maskedContractAddress, singleLogsHash)
   * where maskedContractAddress = pedersen(contract_address, randomness) is provided as part
   * of the block bytes, prepended to each encrypted log.
   * We don't currently count the maskedContractAddress as part of the
   * chargable DA length of the log.
   */
  function computeKernelEncryptedLogsHash(uint256 _offsetInBlock, bytes calldata _body)
    internal
    pure
    returns (bytes32, uint256, uint256)
  {
    uint256 offset = _offsetInBlock;
    uint256 remainingLogsLength = read4(_body, offset);
    uint256 kernelLogsLength = remainingLogsLength;
    offset += 0x4;

    bytes memory flattenedLogHashes; // The hash input

    // Iterate until all the logs were processed
    while (remainingLogsLength > 0) {
      // The length of the logs emitted by Aztec.nr from the function call corresponding to this kernel iteration
      uint256 privateCircuitPublicInputLogsLength = read4(_body, offset);
      offset += 0x4;

      // Decrease remaining logs length by this privateCircuitPublicInputsLogs's length (len(I?_LOGS)) and 4 bytes for I?_LOGS_LEN
      remainingLogsLength -= (privateCircuitPublicInputLogsLength + 0x4);

      kernelLogsLength -= 0x4;

      while (privateCircuitPublicInputLogsLength > 0) {
        uint256 singleCallLogsLengthWithMaskedAddress = read4(_body, offset);
        offset += 0x4;
        // The first 32 bytes of the provided encrypted log are its masked address (see EncryptedL2Log.toBuffer())
        bytes32 maskedContractAddress = bytes32(slice(_body, offset, 0x20));
        offset += 0x20;
        // We don't currently include the masked contract address as part of the DA length
        kernelLogsLength -= 0x20;
        uint256 singleCallLogsLength = singleCallLogsLengthWithMaskedAddress - 0x20;

        bytes32 singleLogHash = Hash.sha256ToField(slice(_body, offset, singleCallLogsLength));

        bytes32 siloedLogHash =
          Hash.sha256ToField(bytes.concat(maskedContractAddress, singleLogHash));
        offset += singleCallLogsLength;

        flattenedLogHashes = bytes.concat(flattenedLogHashes, siloedLogHash);

        privateCircuitPublicInputLogsLength -= (singleCallLogsLengthWithMaskedAddress + 0x4);
      }
    }

    // Not having a 0 value hash for empty logs causes issues with empty txs used for padding.
    if (flattenedLogHashes.length == 0) {
      return (0, offset, 0);
    }

    // padded to MAX_LOGS * 32 bytes
    flattenedLogHashes = bytes.concat(
      flattenedLogHashes,
      new bytes(Constants.MAX_ENCRYPTED_LOGS_PER_TX * 32 - flattenedLogHashes.length)
    );

    bytes32 kernelPublicInputsLogsHash = Hash.sha256ToField(flattenedLogHashes);

    return (kernelPublicInputsLogsHash, offset, kernelLogsLength);
  }

  /**
   * @notice Computes unencrypted logs hash as is done in the kernel circuits.
   * @param _offsetInBlock - The offset of kernel's logs in a block.
   * @param _body - The L2 block calldata.
   * @return The hash of the logs and offset in a block after processing the logs.
   * @dev See above for details. The only difference here is that unencrypted logs are
   * siloed with their contract address.
   */
  function computeKernelUnencryptedLogsHash(uint256 _offsetInBlock, bytes calldata _body)
    internal
    pure
    returns (bytes32, uint256, uint256)
  {
    uint256 offset = _offsetInBlock;
    uint256 remainingLogsLength = read4(_body, offset);
    uint256 kernelLogsLength = remainingLogsLength;
    offset += 0x4;

    bytes memory flattenedLogHashes; // The hash input

    // Iterate until all the logs were processed
    while (remainingLogsLength > 0) {
      // The length of the logs emitted by Aztec.nr from the function call corresponding to this kernel iteration
      uint256 privateCircuitPublicInputLogsLength = read4(_body, offset);
      offset += 0x4;

      // Decrease remaining logs length by this privateCircuitPublicInputsLogs's length (len(I?_LOGS)) and 4 bytes for I?_LOGS_LEN
      remainingLogsLength -= (privateCircuitPublicInputLogsLength + 0x4);

      kernelLogsLength -= 0x4;

      while (privateCircuitPublicInputLogsLength > 0) {
        uint256 singleCallLogsLength = read4(_body, offset);
        offset += 0x4;

        bytes32 singleLogHash = Hash.sha256ToField(slice(_body, offset, singleCallLogsLength));
        // The first 32 bytes of an unencrypted log buffer are its address (see UnencryptedL2Log.toBuffer())
        bytes32 siloedLogHash =
          Hash.sha256ToField(bytes.concat(slice(_body, offset, 0x20), singleLogHash));
        offset += singleCallLogsLength;

        flattenedLogHashes = bytes.concat(flattenedLogHashes, siloedLogHash);

        privateCircuitPublicInputLogsLength -= (singleCallLogsLength + 0x4);
      }
    }

    // Not having a 0 value hash for empty logs causes issues with empty txs used for padding.
    if (flattenedLogHashes.length == 0) {
      return (0, offset, 0);
    }

    // padded to MAX_LOGS * 32 bytes
    flattenedLogHashes = bytes.concat(
      flattenedLogHashes,
      new bytes(Constants.MAX_UNENCRYPTED_LOGS_PER_TX * 32 - flattenedLogHashes.length)
    );

    bytes32 kernelPublicInputsLogsHash = Hash.sha256ToField(flattenedLogHashes);

    return (kernelPublicInputsLogsHash, offset, kernelLogsLength);
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
        _leafs[j / 2] = Hash.sha256ToField(bytes.concat(_leafs[j], _leafs[j + 1]));
      }
      treeSize /= 2;
    }

    return _leafs[0];
  }

  /**
   * @notice Computes the root for a binary unbalanced Merkle-tree given the leaves.
   * @dev Filled in greedily with subtrees. Useful for txsEffectsHash and outHash tree.
   * @param _leaves - The 32 bytes leafs to build the tree of.
   * @return The root of the Merkle tree.
   */
  function computeUnbalancedRoot(bytes32[] memory _leaves) internal pure returns (bytes32) {
    // e.g. an unbalanced tree of 7 txs will contain subtrees of 4, 2, and 1 tx(s) = 111
    // e.g. an unbalanced tree of 9 txs will contain subtrees of 8 and 1 tx(s) = 1001
    // We collect the roots of each subtree
    bytes32 root;
    uint256 currentSubtreeSize = 1;
    uint256 numTxs = _leaves.length;
    // We must calculate the smaller rightmost subtrees first, hence starting at 1
    while (numTxs != 0) {
      // If size & txs == 0, the subtree doesn't exist for this number of txs
      if (currentSubtreeSize & numTxs == 0) {
        currentSubtreeSize <<= 1;
        continue;
      }
      bytes32[] memory leavesInSubtree = new bytes32[](currentSubtreeSize);
      uint256 start = numTxs - currentSubtreeSize;
      for (uint256 i = start; i < numTxs; i++) {
        leavesInSubtree[i - start] = _leaves[i];
      }
      bytes32 subtreeRoot = computeRoot(leavesInSubtree);
      root =
        numTxs == _leaves.length ? subtreeRoot : Hash.sha256ToField(bytes.concat(subtreeRoot, root));
      numTxs -= currentSubtreeSize;
      currentSubtreeSize <<= 1;
    }
    return root;
  }

  /**
   * @notice Computes the root for the binary variable height Merkle-tree made of one tx's L2 to L1 msgs.
   * @dev Mimics compute_kernel_out_hash in base_rollup.
   * TODO(#7218): Revert to fixed height tree for outbox
   * @param _data - The blob of data containing l2 to l1 msgs.
   * @return The root of the Merkle tree.
   */
  function computeTxOutHash(uint256 _start, bytes calldata _data) internal pure returns (bytes32) {
    uint256 offset = _start;
    // The stored offsets.l2ToL1Msgs does not include the single byte storing the num of msgs, hence -1
    uint32 numMsgs = uint32(read1(_data, _start - 1));
    uint256 numMsgsToPad = computeNumMsgsToPad(uint32(numMsgs));
    bytes32[] memory leavesInMsgTree = new bytes32[](numMsgs + numMsgsToPad);
    for (uint256 i = 0; i < numMsgs; i++) {
      leavesInMsgTree[i] = bytes32(slice(_data, offset, 0x20));
      offset += 0x20;
    }
    bytes32 outHash = computeRoot(leavesInMsgTree);
    return outHash;
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
  function sliceAndPadLeft(
    bytes calldata _data,
    uint256 _start,
    uint256 _length,
    uint256 _targetLength
  ) internal pure returns (bytes memory) {
    return bytes.concat(new bytes(_targetLength - _length), _data[_start:_start + _length]);
  }

  /**
   * @notice Wrapper around the slicing and padding to avoid some stack too deep
   * @param _data - The data to slice
   * @param _start - The start of the slice
   * @param _length - The length of the slice
   * @param _targetLength - The length of the padded array
   * @return The slice
   */
  function sliceAndPadRight(
    bytes calldata _data,
    uint256 _start,
    uint256 _length,
    uint256 _targetLength
  ) internal pure returns (bytes memory) {
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

  /**
   * @notice Pads L2 to L1 messages to the next power of 2 - simple algo as we only have a max of 8 msgs
   * @param _numL2toL1Msgs - num of msgs (currently between 0 and MAX_L2_TO_L1_MSGS_PER_TX = 8)
   * @return Num msgs to pad
   */
  function computeNumMsgsToPad(uint32 _numL2toL1Msgs) internal pure returns (uint32) {
    // We handle the edge case of 0 msgs by padding by 1 - this ensures computeRoot() gives a result of 0
    if (_numL2toL1Msgs == 0 || _numL2toL1Msgs == 1) {
      return 1;
    }

    uint32 v = _numL2toL1Msgs;

    // the following rounds _numL2toL1Msgs up to the next power of 2 (works only for 4 bytes value!)
    v--;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    // We dont expect to have MAX_L2_TO_L1_MSGS_PER_TX greater than 8 bits, so commented out the below to save gas
    // v |= v >> 8;
    // v |= v >> 16;
    v++;

    return v - _numL2toL1Msgs;
  }

  function computeNumTxEffectsToPad(uint32 _numTxEffects) internal pure returns (uint32) {
    // 2 is the minimum number of tx effects so we have to handle the following 2 cases separately
    if (_numTxEffects == 0) {
      return 2;
    } else if (_numTxEffects == 1) {
      return 1;
    }
    return 0;
  }
}
