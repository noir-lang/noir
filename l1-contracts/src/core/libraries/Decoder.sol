// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

// Libraries
import {Constants} from "@aztec/core/libraries/Constants.sol";
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
 * L2 Block Data specification
 * -------------------
 *
 *  | byte start                                           | num bytes  | name
 *  | ---                                                  | ---        | ---
 *  | 0x00                                                 | 0x04       | L2 block number
 *  | 0x04                                                 | 0x20       | startPrivateDataTreeSnapshot.root
 *  | 0x24                                                 | 0x04       | startPrivateDataTreeSnapshot.nextAvailableLeafIndex
 *  | 0x28                                                 | 0x20       | startNullifierTreeSnapshot.root
 *  | 0x48                                                 | 0x04       | startNullifierTreeSnapshot.nextAvailableLeafIndex
 *  | 0x4c                                                 | 0x20       | startContractTreeSnapshot.root
 *  | 0x6c                                                 | 0x04       | startContractTreeSnapshot.nextAvailableLeafIndex
 *  | 0x70                                                 | 0x20       | startTreeOfHistoricPrivateDataTreeRootsSnapshot.root
 *  | 0x90                                                 | 0x04       | startTreeOfHistoricPrivateDataTreeRootsSnapshot.nextAvailableLeafIndex
 *  | 0x94                                                 | 0x20       | startTreeOfHistoricContractTreeRootsSnapshot.root
 *  | 0xb4                                                 | 0x04       | startTreeOfHistoricContractTreeRootsSnapshot.nextAvailableLeafIndex
 *  | 0xb8                                                 | 0x20       | startPublicDataTreeRoot
 *  | 0xd8                                                 | 0x20       | startL1ToL2MessagesTreeSnapshot.root
 *  | 0xf8                                                 | 0x04       | startL1ToL2MessagesTreeSnapshot.nextAvailableLeafIndex
 *  | 0xfc                                                 | 0x20       | startTreeOfHistoricL1ToL2MessagesTreeRootsSnapshot.root
 *  | 0x11c                                                | 0x04       | startTreeOfHistoricL1ToL2MessagesTreeRootsSnapshot.nextAvailableLeafIndex
 *  | 0x120                                                | 0x20       | endPrivateDataTreeSnapshot.root
 *  | 0x140                                                | 0x04       | endPrivateDataTreeSnapshot.nextAvailableLeafIndex
 *  | 0x144                                                | 0x20       | endNullifierTreeSnapshot.root
 *  | 0x164                                                | 0x04       | endNullifierTreeSnapshot.nextAvailableLeafIndex
 *  | 0x168                                                | 0x20       | endContractTreeSnapshot.root
 *  | 0x188                                                | 0x04       | endContractTreeSnapshot.nextAvailableLeafIndex
 *  | 0x18c                                                | 0x20       | endTreeOfHistoricPrivateDataTreeRootsSnapshot.root
 *  | 0x1ac                                                | 0x04       | endTreeOfHistoricPrivateDataTreeRootsSnapshot.nextAvailableLeafIndex
 *  | 0x1b0                                                | 0x20       | endTreeOfHistoricContractTreeRootsSnapshot.root
 *  | 0x1d0                                                | 0x04       | endTreeOfHistoricContractTreeRootsSnapshot.nextAvailableLeafIndex
 *  | 0x1d4                                                | 0x20       | endPublicDataTreeRoot
 *  | 0x1f4                                                | 0x20       | endL1ToL2MessagesTreeSnapshot.root
 *  | 0x214                                                | 0x04       | endL1ToL2MessagesTreeSnapshot.nextAvailableLeafIndex
 *  | 0x218                                                | 0x20       | endTreeOfHistoricL1ToL2MessagesTreeRootsSnapshot.root
 *  | 0x238                                                | 0x04       | endTreeOfHistoricL1ToL2MessagesTreeRootsSnapshot.nextAvailableLeafIndex
 *  | 0x23c                                                | 0x04       | len(newCommitments) denoted a
 *  | 0x240                                                | a * 0x20   | newCommitments (each element 32 bytes)
 *  | 0x240 + a * 0x20                                     | 0x04       | len(newNullifiers) denoted b
 *  | 0x244 + a * 0x20                                     | b * 0x20   | newNullifiers (each element 32 bytes)
 *  | 0x244 + (a + b) * 0x20                               | 0x04       | len(newPublicDataWrites) denoted c
 *  | 0x248 + (a + b) * 0x20                               | c * 0x40   | newPublicDataWrites (each element 64 bytes)
 *  | 0x248 + (a + b) * 0x20 + c * 0x40                    | 0x04       | len(newL2ToL1msgs) denoted d
 *  | 0x24c + (a + b) * 0x20 + c * 0x40                    | d * 0x20   | newL2ToL1msgs (each element 32 bytes)
 *  | 0x24c + (a + b + d) * 0x20 + c * 0x40                | 0x04       | len(newContracts) denoted e
 *  | 0x250 + (a + b + d) * 0x20 + c * 0x40                | e * 0x20   | newContracts (each element 32 bytes)
 *  | 0x250 + (a + b + d) * 0x20 + c * 0x40 + e * 0x20     | e * 0x34   | newContractData (each element 52 bytes)
 *  | 0x250 + (a + b + d) * 0x20 + c * 0x40 + e * 0x54     | 0x04       | len(l1ToL2Messages) denoted f
 *  | K := 0x254 + (a + b + d) * 0x20 + c * 0x40 + e * 0x54| f * 0x20   | l1ToL2Messages (each element 32 bytes)
 *  | K + f * 0x20                                         | 0x04       | byteLen(newEncryptedLogs) denoted g
 *  | K + f * 0x20 + 0x04                                  | g          | newEncryptedLogs
 *  | K + f * 0x20 + 0x04 + g                              | 0x04       | byteLen(newUnencryptedLogs) denoted h
 *  | K + f * 0x20 + 0x04 + g + 0x04                       | h          | newUnencryptedLogs
 *  |---                                                   |---         | ---
 */
library Decoder {
  struct ArrayLengths {
    uint256 commitmentCount;
    uint256 nullifierCount;
    uint256 dataWritesCount;
    uint256 l2ToL1MsgsCount;
    uint256 contractCount;
    uint256 l1Tol2MsgsCount;
    uint256 encryptedLogsLength; // in bytes
    uint256 unencryptedLogsLength; // in bytes
  }

  struct ArrayOffsets {
    uint256 commitmentOffset;
    uint256 nullifierOffset;
    uint256 publicDataOffset;
    uint256 l2ToL1MsgsOffset;
    uint256 contractOffset;
    uint256 contractDataOffset;
    uint256 l1ToL2MsgsOffset;
    uint256 encryptedLogsOffset;
    uint256 unencryptedLogsOffset;
  }

  // Note: Used in `_computeConsumables` to get around stack too deep errors.
  struct ConsumablesVars {
    bytes32[] baseLeaves;
    bytes32[] l2ToL1Msgs;
    bytes baseLeaf;
    bytes32 encrypedLogsHashKernel1;
    bytes32 encrypedLogsHashKernel2;
    bytes32 unencryptedLogsHashKernel1;
    bytes32 unencryptedLogsHashKernel2;
  }

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
    startStateHash = computeStateHash(l2BlockNumber - 1, 0x4, _l2Block);
    endStateHash = computeStateHash(l2BlockNumber, 0x120, _l2Block);

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
    uint256 size = 0x23c - 0x04 + 0x20 + 0x20;
    bytes memory temp = new bytes(size);
    assembly {
      calldatacopy(add(temp, 0x20), add(_l2Block.offset, 0x04), size)
      let endOfTreesData := sub(0x23c, 0x04)
      mstore(add(temp, add(0x20, endOfTreesData)), _diffRoot)
      mstore(add(temp, add(0x40, endOfTreesData)), _l1ToL2MsgsHash)
    }
    return Hash.sha256ToField(temp);
  }

  /**
   * @notice Extract the L2 block number from the block
   * @param _l2Block - The L2 block calldata
   * @return l2BlockNumber - The L2 block number
   */
  function getL2BlockNumber(bytes calldata _l2Block) internal pure returns (uint256 l2BlockNumber) {
    assembly {
      l2BlockNumber := and(shr(224, calldataload(_l2Block.offset)), 0xffffffff)
    }
  }

  /**
   * @notice Computes a state hash
   * @param _l2BlockNumber - The L2 block number
   * @param _offset - The offset into the data, 0x04 for start, 0xd8 for end
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
    bytes memory temp = new bytes(0x120);

    assembly {
      // Copy the L2 block number byte by byte to memory
      mstore8(add(temp, 0x20), shr(24, _l2BlockNumber))
      mstore8(add(temp, 0x21), shr(16, _l2BlockNumber))
      mstore8(add(temp, 0x22), shr(8, _l2BlockNumber))
      mstore8(add(temp, 0x23), _l2BlockNumber)
    }
    assembly {
      // Copy header elements (not including block number) for start or end (size 0xd4)
      calldatacopy(add(temp, 0x24), add(_l2Block.offset, _offset), 0x11c)
    }

    return sha256(temp);
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
    // Find the lengths of the different inputs
    // TODO: Naming / getting the messages root within this function is a bit weird
    ArrayLengths memory lengths;
    ArrayOffsets memory offsets;
    {
      assembly {
        let offset := add(_l2Block.offset, 0x23c)
        let commitmentCount := and(shr(224, calldataload(offset)), 0xffffffff)
        offset := add(add(offset, 0x4), mul(commitmentCount, 0x20))
        let nullifierCount := and(shr(224, calldataload(offset)), 0xffffffff)
        offset := add(add(offset, 0x4), mul(nullifierCount, 0x20))
        let dataWritesCount := and(shr(224, calldataload(offset)), 0xffffffff)
        offset := add(add(offset, 0x4), mul(nullifierCount, 0x40))
        let l2ToL1MsgsCount := and(shr(224, calldataload(offset)), 0xffffffff)
        offset := add(add(offset, 0x4), mul(l2ToL1MsgsCount, 0x20))
        let contractCount := and(shr(224, calldataload(offset)), 0xffffffff)
        offset := add(add(offset, 0x4), mul(contractCount, 0x54))
        let l1Tol2MsgsCount := and(shr(224, calldataload(offset)), 0xffffffff)
        offset := add(add(offset, 0x4), mul(l1Tol2MsgsCount, 0x20))
        let encryptedLogsLength := and(shr(224, calldataload(offset)), 0xffffffff)
        offset := add(add(offset, 0x4), encryptedLogsLength)
        let unencryptedLogsLength := and(shr(224, calldataload(offset)), 0xffffffff)

        // Store it in lengths
        mstore(lengths, commitmentCount)
        mstore(add(lengths, 0x20), nullifierCount)
        mstore(add(lengths, 0x40), dataWritesCount)
        mstore(add(lengths, 0x60), l2ToL1MsgsCount)
        mstore(add(lengths, 0x80), contractCount)
        mstore(add(lengths, 0xa0), l1Tol2MsgsCount) // currently included to allow optimisation where empty messages are not included in calldata
        mstore(add(lengths, 0xc0), encryptedLogsLength)
        mstore(add(lengths, 0xe0), unencryptedLogsLength)
      }
    }

    ConsumablesVars memory vars;
    vars.baseLeaves = new bytes32[](
            lengths.commitmentCount / (Constants.COMMITMENTS_PER_KERNEL * 2)
        );
    vars.l2ToL1Msgs = new bytes32[](
            lengths.l2ToL1MsgsCount
        );

    // Data starts after header. Look at L2 Block Data specification at the top of this file.
    {
      offsets.commitmentOffset = 0x240;
      offsets.nullifierOffset = offsets.commitmentOffset + 0x4 + lengths.commitmentCount * 0x20;
      offsets.publicDataOffset = offsets.nullifierOffset + 0x4 + lengths.nullifierCount * 0x20;
      offsets.l2ToL1MsgsOffset = offsets.publicDataOffset + 0x4 + lengths.dataWritesCount * 0x40;
      offsets.contractOffset = offsets.l2ToL1MsgsOffset + 0x4 + lengths.l2ToL1MsgsCount * 0x20;
      offsets.contractDataOffset = offsets.contractOffset + lengths.contractCount * 0x20;
      offsets.l1ToL2MsgsOffset = offsets.contractDataOffset + 0x4 + lengths.contractCount * 0x34;
      offsets.encryptedLogsOffset = offsets.l1ToL2MsgsOffset + 0x4 + lengths.l1Tol2MsgsCount * 0x20;
      offsets.unencryptedLogsOffset =
        offsets.encryptedLogsOffset + 0x4 + lengths.encryptedLogsLength;

      // load the l2 to l1 msgs (done here as offset will be altered in loop)
      assembly {
        let l2ToL1Msgs := mload(add(vars, 0x20))
        calldatacopy(
          add(l2ToL1Msgs, 0x20),
          add(_l2Block.offset, mload(add(offsets, 0x60))),
          mul(mload(add(lengths, 0x60)), 0x20)
        )
      }

      // Create the leaf to contain commitments (8 * 0x20) + nullifiers (8 * 0x20)
      // + new public data writes (8 * 0x40) + contract deployments (2 * 0x60) + logs hashes (4 * 0x20)
      // TODO: Replace 0x540 with 0x5C0 once the logs functionality is added in other places
      vars.baseLeaf = new bytes(0x540);

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
         *    encrypedLogsHashKernel1,                                   |
         *    encrypedLogsHashKernel2,                                   |=> Computed bellow from logs' preimages.
         *    unencryptedLogsHashKernel1,                                |
         *    unencryptedLogsHashKernel2                              ___|
         * );
         * Note that we always read data, the l2Block (atm) must therefore include dummy or zero-notes for
         * Zero values.
         */

        // TODO: Uncomment once the logs functionality is added in other places
        // /**
        //  * Compute encrypted and unencrypted logs hashes corresponding to the current leaf.
        //  * Note: `_computeKernelLogsHash` will advance offsets by the number of bytes processed.
        //  */
        // (vars.encrypedLogsHashKernel1, offsets.encryptedLogsOffset) =
        //   _computeKernelLogsHash(offsets.encryptedLogsOffset, _l2Block);
        // (vars.encrypedLogsHashKernel2, offsets.encryptedLogsOffset) =
        //   _computeKernelLogsHash(offsets.encryptedLogsOffset, _l2Block);

        // (vars.unencryptedLogsHashKernel1, offsets.unencryptedLogsOffset) =
        //   _computeKernelLogsHash(offsets.unencryptedLogsOffset, _l2Block);
        // (vars.unencryptedLogsHashKernel2, offsets.unencryptedLogsOffset) =
        //   _computeKernelLogsHash(offsets.unencryptedLogsOffset, _l2Block);

        assembly {
          let baseLeaf := mload(add(vars, 0x40)) // Load the pointer to `vars.baseLeaf`
          let dstPtr := add(baseLeaf, 0x20) // Current position withing `baseLeaf` to write to

          // Adding new commitments
          calldatacopy(dstPtr, add(_l2Block.offset, mload(offsets)), mul(0x08, 0x20))
          dstPtr := add(dstPtr, mul(0x08, 0x20))

          // Adding new nullifiers
          calldatacopy(dstPtr, add(_l2Block.offset, mload(add(offsets, 0x20))), mul(0x08, 0x20))
          dstPtr := add(dstPtr, mul(0x08, 0x20))

          // Adding new public data writes
          calldatacopy(dstPtr, add(_l2Block.offset, mload(add(offsets, 0x40))), mul(0x08, 0x40))
          dstPtr := add(dstPtr, mul(0x08, 0x40))

          // Adding new l2 to l1 msgs
          calldatacopy(dstPtr, add(_l2Block.offset, mload(add(offsets, 0x60))), mul(0x04, 0x20))
          dstPtr := add(dstPtr, mul(0x04, 0x20))

          // Adding Contract Leafs
          calldatacopy(dstPtr, add(_l2Block.offset, mload(add(offsets, 0x80))), mul(0x2, 0x20))
          dstPtr := add(dstPtr, mul(2, 0x20))

          // Kernel1.contract.aztecAddress
          let contractDataOffset := mload(add(offsets, 0xa0))
          calldatacopy(dstPtr, add(_l2Block.offset, contractDataOffset), 0x20)
          dstPtr := add(dstPtr, 0x20)

          // Kernel1.contract.ethAddress padded to 32 bytes
          // Add 12 (0xc) bytes of padding to the ethAddress
          dstPtr := add(dstPtr, 0xc)
          calldatacopy(dstPtr, add(_l2Block.offset, add(contractDataOffset, 0x20)), 0x14)
          dstPtr := add(dstPtr, 0x14)

          // Kernel2.contract.aztecAddress
          calldatacopy(dstPtr, add(_l2Block.offset, add(contractDataOffset, 0x34)), 0x20)
          dstPtr := add(dstPtr, 0x20)

          // Kernel2.contract.ethAddress padded to 32 bytes
          // Add 12 (0xc) bytes of padding to the ethAddress
          dstPtr := add(dstPtr, 0xc)
          calldatacopy(dstPtr, add(_l2Block.offset, add(contractDataOffset, 0x54)), 0x14)

          // TODO: Uncomment once the logs functionality is added in other places
          // // encryptedLogsHashKernel1
          // dstPtr := add(dstPtr, 0x14)
          // mstore(dstPtr, mload(add(vars, 0x60))) // `encryptedLogsHashKernel1` starts at 0x60 in `vars`

          // // encryptedLogsHashKernel2
          // dstPtr := add(dstPtr, 0x20)
          // mstore(dstPtr, mload(add(vars, 0x80))) // `encryptedLogsHashKernel2` starts at 0x80 in `vars`

          // // unencryptedLogsHashKernel1
          // dstPtr := add(dstPtr, 0x20)
          // mstore(dstPtr, mload(add(vars, 0xa0))) // `unencryptedLogsHashKernel1` starts at 0xa0 in `vars`

          // // unencryptedLogsHashKernel2
          // dstPtr := add(dstPtr, 0x20)
          // mstore(dstPtr, mload(add(vars, 0xc0))) // `unencryptedLogsHashKernel2` starts at 0xc0 in `vars`
        }

        offsets.commitmentOffset += 2 * Constants.COMMITMENTS_PER_KERNEL * 0x20;
        offsets.nullifierOffset += 2 * Constants.NULLIFIERS_PER_KERNEL * 0x20;
        offsets.publicDataOffset += 2 * Constants.PUBLIC_DATA_WRITES_PER_KERNEL * 0x40;
        offsets.l2ToL1MsgsOffset += 2 * Constants.L2_TO_L1_MSGS_PER_KERNEL * 0x20;
        offsets.contractOffset += 2 * 0x20;
        offsets.contractDataOffset += 2 * 0x34;

        vars.baseLeaves[i] = sha256(vars.baseLeaf);
      }
    }

    bytes32 diffRoot = computeRoot(vars.baseLeaves);
    bytes32[] memory l1ToL2Msgs;
    bytes32 l1ToL2MsgsHash;
    {
      // `l1ToL2Msgs` is fixed size so if `lengths.l1Tol2MsgsCount` < `Constants.L1_TO_L2_MSGS_PER_ROLLUP` the array
      // will contain some zero values.
      uint256 l1ToL2MsgsHashPreimageSize = 0x20 * lengths.l1Tol2MsgsCount;
      l1ToL2Msgs = new bytes32[](Constants.L1_TO_L2_MSGS_PER_ROLLUP);
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
   * @notice Computes the hash of logs in a kernel.
   * @param _offset - The offset of kernel's logs in calldata.
   * @param - The L2 block calldata.
   * @return The hash of the logs and offset pointing to the end of the logs in calldata.
   * @dev We have logs preimages on the input and we need to perform the same hashing process as is done in the kernel.
   *      In each iteration of kernel, the kernel computes a hash of the previous iteration's logs hash and the current
   *      iteration's logs. E.g. for resulting logs hash of a kernel with 3 iterations would be computed as:
   *
   *        logsHash = sha256(sha256(sha256(I1_LOGS), I2_LOGS), I3_LOGS)
   *
   *      where I1_LOGS, I2_LOGS and I3_LOGS are logs emitted in the first, second and third iterations respectively.
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
  function computeKernelLogsHash(uint256 _offset, bytes calldata /* _l2Block */ )
    internal
    pure
    returns (bytes32, uint256)
  {
    uint256 remainingLogsLength;
    uint256 offset;
    assembly {
      // Set the remaining logs length to the total logs length
      // Loads 32 bytes from calldata, shifts right by 224 bits and masks the result with 0xffffffff
      remainingLogsLength := and(shr(224, calldataload(_offset)), 0xffffffff)
      // Move the calldata offset by the 4 bytes we just read
      offset := add(_offset, 0x4)
    }

    bytes32 logsHash;
    uint256 iterationLogsLength;
    uint256 tempLength;

    // Iterate until all the logs were processed
    while (remainingLogsLength > 0) {
      assembly {
        // Load this iteration's logs length
        iterationLogsLength := and(shr(224, calldataload(offset)), 0xffffffff)
        offset := add(offset, 0x4)
        tempLength := add(0x20, iterationLogsLength) // len(logsHash) + iterationLogsLength
      }

      // TODO: Allocating memory in each iteration is expensive. Should we somehow set it to max length of all the
      //       iterations? (e.g. We could do that by first searching for max length in a loop or by modifying
      //       the encoding and storing max length on a predefined position)
      bytes memory temp = new bytes(tempLength);
      assembly {
        // Copy logsHash from stack to temp
        mstore(add(temp, 0x20), logsHash)

        // Load this iteration's logs to memory
        calldatacopy(add(temp, 0x40), offset, iterationLogsLength)
        offset := add(offset, iterationLogsLength)

        // Decrease remaining logs length by this iteration's logs length (len(I?_LOGS)) and 4 bytes for I?_LOGS_LEN
        remainingLogsLength := sub(remainingLogsLength, add(iterationLogsLength, 0x4))
      }

      // Compute current iteration's logs hash
      logsHash = sha256(temp);
    }

    return (logsHash, offset);
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
        _leafs[j / 2] = sha256(abi.encode(_leafs[j], _leafs[j + 1]));
      }
    }

    return _leafs[0];
  }
}
