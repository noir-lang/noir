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
 *  | byte start                                             | num bytes  | name
 *  | ---                                                    | ---        | ---
 *  | 0x0000                                                 | 0x20       | chain-id
 *  | 0x0020                                                 | 0x20       | version
 *  | 0x0040                                                 | 0x20       | L2 block number
 *  | 0x0060                                                 | 0x20       | L2 timestamp
 *  | 0x0080                                                 | 0x20       | startPrivateDataTreeSnapshot.root
 *  | 0x00a0                                                 | 0x04       | startPrivateDataTreeSnapshot.nextAvailableLeafIndex
 *  | 0x00a4                                                 | 0x20       | startNullifierTreeSnapshot.root
 *  | 0x00c4                                                 | 0x04       | startNullifierTreeSnapshot.nextAvailableLeafIndex
 *  | 0x00c8                                                 | 0x20       | startContractTreeSnapshot.root
 *  | 0x00e8                                                 | 0x04       | startContractTreeSnapshot.nextAvailableLeafIndex
 *  | 0x00ec                                                 | 0x20       | startTreeOfHistoricPrivateDataTreeRootsSnapshot.root
 *  | 0x010c                                                 | 0x04       | startTreeOfHistoricPrivateDataTreeRootsSnapshot.nextAvailableLeafIndex
 *  | 0x0110                                                 | 0x20       | startTreeOfHistoricContractTreeRootsSnapshot.root
 *  | 0x0130                                                 | 0x04       | startTreeOfHistoricContractTreeRootsSnapshot.nextAvailableLeafIndex
 *  | 0x0134                                                 | 0x20       | startPublicDataTreeRoot
 *  | 0x0154                                                 | 0x20       | startL1ToL2MessagesTreeSnapshot.root
 *  | 0x0174                                                 | 0x04       | startL1ToL2MessagesTreeSnapshot.nextAvailableLeafIndex
 *  | 0x0178                                                 | 0x20       | startTreeOfHistoricL1ToL2MessagesTreeRootsSnapshot.root
 *  | 0x0198                                                 | 0x04       | startTreeOfHistoricL1ToL2MessagesTreeRootsSnapshot.nextAvailableLeafIndex
 *  | 0x019c                                                 | 0x20       | startBlocksTreeSnapshot.root
 *  | 0x01bc                                                 | 0x04       | startBlocksTreeSnapshot.nextAvailableLeafIndex
 *  | 0x01c0                                                 | 0x20       | endPrivateDataTreeSnapshot.root
 *  | 0x01e0                                                 | 0x04       | endPrivateDataTreeSnapshot.nextAvailableLeafIndex
 *  | 0x01e4                                                 | 0x20       | endNullifierTreeSnapshot.root
 *  | 0x0204                                                 | 0x04       | endNullifierTreeSnapshot.nextAvailableLeafIndex
 *  | 0x0208                                                 | 0x20       | endContractTreeSnapshot.root
 *  | 0x0228                                                 | 0x04       | endContractTreeSnapshot.nextAvailableLeafIndex
 *  | 0x022c                                                 | 0x20       | endTreeOfHistoricPrivateDataTreeRootsSnapshot.root
 *  | 0x024c                                                 | 0x04       | endTreeOfHistoricPrivateDataTreeRootsSnapshot.nextAvailableLeafIndex
 *  | 0x0250                                                 | 0x20       | endTreeOfHistoricContractTreeRootsSnapshot.root
 *  | 0x0270                                                 | 0x04       | endTreeOfHistoricContractTreeRootsSnapshot.nextAvailableLeafIndex
 *  | 0x0274                                                 | 0x20       | endPublicDataTreeRoot
 *  | 0x0294                                                 | 0x20       | endL1ToL2MessagesTreeSnapshot.root
 *  | 0x02b4                                                 | 0x04       | endL1ToL2MessagesTreeSnapshot.nextAvailableLeafIndex
 *  | 0x0300                                                 | 0x20       | endTreeOfHistoricL1ToL2MessagesTreeRootsSnapshot.root
 *  | 0x02d8                                                 | 0x04       | endTreeOfHistoricL1ToL2MessagesTreeRootsSnapshot.nextAvailableLeafIndex
 *  | 0x02dc                                                 | 0x20       | endBlocksTreeSnapshot.root
 *  | 0x02fc                                                 | 0x04       | endBlocksTreeSnapshot.nextAvailableLeafIndex
 *  | 0x0300                                                 | a * 0x20   | newCommitments (each element 32 bytes)
 *  | 0x0300 + a * 0x20                                      | 0x04       | len(newNullifiers) denoted b
 *  | 0x0304 + a * 0x20                                      | b * 0x20   | newNullifiers (each element 32 bytes)
 *  | 0x0304 + (a + b) * 0x20                                | 0x04       | len(newPublicDataWrites) denoted c
 *  | 0x0308 + (a + b) * 0x20                                | c * 0x40   | newPublicDataWrites (each element 64 bytes)
 *  | 0x0308 + (a + b) * 0x20 + c * 0x40                     | 0x04       | len(newL2ToL1msgs) denoted d
 *  | 0x030c + (a + b) * 0x20 + c * 0x40                     | d * 0x20   | newL2ToL1msgs (each element 32 bytes)
 *  | 0x030c + (a + b + d) * 0x20 + c * 0x40                 | 0x04       | len(newContracts) denoted e
 *  | 0x0310 + (a + b + d) * 0x20 + c * 0x40                 | e * 0x20   | newContracts (each element 32 bytes)
 *  | 0x0310 + (a + b + d) * 0x20 + c * 0x40 + e * 0x20      | e * 0x34   | newContractData (each element 52 bytes)
 *  | 0x0310 + (a + b + d) * 0x20 + c * 0x40 + e * 0x54      | 0x04       | len(l1ToL2Messages) denoted f
 *  | K := 0x0310 + (a + b + d) * 0x20 + c * 0x40 + e * 0x54 | f * 0x20   | l1ToL2Messages (each element 32 bytes)
 *  | K + f * 0x20                                           | 0x04       | byteLen(newEncryptedLogs) denoted g
 *  | K + f * 0x20 + 0x04                                    | g          | newEncryptedLogs
 *  | K + f * 0x20 + 0x04 + g                                | 0x04       | byteLen(newUnencryptedLogs) denoted h
 *  | K + f * 0x20 + 0x04 + g + 0x04                         | h          | newUnencryptedLogs
 *  |---                                                     |---         | ---
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

  // Note: Used in `computeConsumables` to get around stack too deep errors.
  struct ConsumablesVars {
    bytes32[] baseLeaves;
    bytes32[] l2ToL1Msgs;
    bytes baseLeaf;
    bytes32 encrypedLogsHashKernel1;
    bytes32 encrypedLogsHashKernel2;
    bytes32 unencryptedLogsHashKernel1;
    bytes32 unencryptedLogsHashKernel2;
  }

  // DECODING OFFSET CONSTANTS
  // Where the start of trees metadata begins in the block
  uint256 private constant START_TREES_BLOCK_HEADER_OFFSET = 0x80;

  // The size of the block header elements
  uint256 private constant TREES_BLOCK_HEADER_SIZE = 0x140;

  // Where the end of trees metadata begns in the block
  uint256 private constant END_TREES_BLOCK_HEADER_OFFSET =
    START_TREES_BLOCK_HEADER_OFFSET + TREES_BLOCK_HEADER_SIZE;

  // Where the metadata ends and the block data begins.
  // This is really (START_TREES_BLOCK_HEADER_OFFSET + 2 * TREES_BLOCK_HEADER_SIZE) but assembly doesnt allow comptime constant use
  uint256 private constant BLOCK_HEADER_OFFSET = 0x0300;

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
    bytes memory temp = new bytes(BLOCK_HEADER_OFFSET + 0x20 + 0x20);
    assembly {
      calldatacopy(add(temp, 0x20), _l2Block.offset, BLOCK_HEADER_OFFSET)
      mstore(add(temp, add(0x20, BLOCK_HEADER_OFFSET)), _diffRoot)
      mstore(add(temp, add(0x40, BLOCK_HEADER_OFFSET)), _l1ToL2MsgsHash)
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
      l2BlockNumber := calldataload(add(_l2Block.offset, 0x40))
    }
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
    // 0x20 for the block number + TREES_BLOCK_HEADER_SIZE for the header elements
    bytes memory temp = new bytes(0x20 + TREES_BLOCK_HEADER_SIZE);

    assembly {
      // Copy block number
      mstore(add(temp, 0x20), _l2BlockNumber)
      // Copy header elements (not including block number) for start or end
      calldatacopy(add(temp, 0x40), add(_l2Block.offset, _offset), TREES_BLOCK_HEADER_SIZE)
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
        let offset := add(_l2Block.offset, BLOCK_HEADER_OFFSET)
        let commitmentCount := and(shr(224, calldataload(offset)), 0xffffffff)
        offset := add(add(offset, 0x4), mul(commitmentCount, 0x20))
        let nullifierCount := and(shr(224, calldataload(offset)), 0xffffffff)
        offset := add(add(offset, 0x4), mul(nullifierCount, 0x20))
        let dataWritesCount := and(shr(224, calldataload(offset)), 0xffffffff)
        offset := add(add(offset, 0x4), mul(dataWritesCount, 0x40))
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
            lengths.commitmentCount / (Constants.COMMITMENTS_PER_TX * 2)
        );
    vars.l2ToL1Msgs = new bytes32[](
            lengths.l2ToL1MsgsCount
        );

    // Data starts after header. Look at L2 Block Data specification at the top of this file.
    {
      offsets.commitmentOffset = BLOCK_HEADER_OFFSET + 0x4;
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

      // Create the leaf to contain commitments (2 * COMMITMENTS_PER_TX * 020) + nullifiers (2 * NULLIFIERS_PER_TX * 0x20)
      // + new public data writes (8 * 0x40) + contract deployments (2 * 0x60) + logs hashes (2 * 4 * 0x20)
      vars.baseLeaf =
      new bytes(2 * Constants.COMMITMENTS_PER_TX * 0x20 + 2 * Constants.NULLIFIERS_PER_TX * 0x20 + 2 * Constants.PUBLIC_DATA_WRITES_PER_TX * 0x40 + 2 * Constants.CONTRACTS_PER_TX * 0x60 + 2 * 4 * 0x20);

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

        /**
         * Compute encrypted and unencrypted logs hashes corresponding to the current leaf.
         * Note: `computeKernelLogsHash` will advance offsets by the number of bytes processed.
         */
        (vars.encrypedLogsHashKernel1, offsets.encryptedLogsOffset) =
          computeKernelLogsHash(offsets.encryptedLogsOffset, _l2Block);
        (vars.encrypedLogsHashKernel2, offsets.encryptedLogsOffset) =
          computeKernelLogsHash(offsets.encryptedLogsOffset, _l2Block);

        (vars.unencryptedLogsHashKernel1, offsets.unencryptedLogsOffset) =
          computeKernelLogsHash(offsets.unencryptedLogsOffset, _l2Block);
        (vars.unencryptedLogsHashKernel2, offsets.unencryptedLogsOffset) =
          computeKernelLogsHash(offsets.unencryptedLogsOffset, _l2Block);

        uint256 commitmentsPerBase = 2 * Constants.COMMITMENTS_PER_TX;
        uint256 nullifiersPerBase = 2 * Constants.NULLIFIERS_PER_TX;

        assembly {
          let baseLeaf := mload(add(vars, 0x40)) // Load the pointer to `vars.baseLeaf`
          let dstPtr := add(baseLeaf, 0x20) // Current position withing `baseLeaf` to write to

          // Adding new commitments
          calldatacopy(dstPtr, add(_l2Block.offset, mload(offsets)), mul(commitmentsPerBase, 0x20))
          dstPtr := add(dstPtr, mul(commitmentsPerBase, 0x20))

          // Adding new nullifiers
          calldatacopy(
            dstPtr, add(_l2Block.offset, mload(add(offsets, 0x20))), mul(nullifiersPerBase, 0x20)
          )
          dstPtr := add(dstPtr, mul(nullifiersPerBase, 0x20))

          // Adding new public data writes
          calldatacopy(dstPtr, add(_l2Block.offset, mload(add(offsets, 0x40))), mul(0x08, 0x40))
          dstPtr := add(dstPtr, mul(0x08, 0x40))

          // Adding new l2 to l1 msgs
          calldatacopy(dstPtr, add(_l2Block.offset, mload(add(offsets, 0x60))), mul(0x04, 0x20))
          dstPtr := add(dstPtr, mul(0x04, 0x20))

          // Adding Contract Leafs
          calldatacopy(dstPtr, add(_l2Block.offset, mload(add(offsets, 0x80))), mul(0x2, 0x20))
          dstPtr := add(dstPtr, mul(0x2, 0x20))

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

          // encryptedLogsHashKernel1
          dstPtr := add(dstPtr, 0x14)
          mstore(dstPtr, mload(add(vars, 0x60))) // `encryptedLogsHashKernel1` starts at 0x60 in `vars`

          // encryptedLogsHashKernel2
          dstPtr := add(dstPtr, 0x20)
          mstore(dstPtr, mload(add(vars, 0x80))) // `encryptedLogsHashKernel2` starts at 0x80 in `vars`

          // unencryptedLogsHashKernel1
          dstPtr := add(dstPtr, 0x20)
          mstore(dstPtr, mload(add(vars, 0xa0))) // `unencryptedLogsHashKernel1` starts at 0xa0 in `vars`

          // unencryptedLogsHashKernel2
          dstPtr := add(dstPtr, 0x20)
          mstore(dstPtr, mload(add(vars, 0xc0))) // `unencryptedLogsHashKernel2` starts at 0xc0 in `vars`
        }

        offsets.commitmentOffset += 2 * Constants.COMMITMENTS_PER_TX * 0x20;
        offsets.nullifierOffset += 2 * Constants.NULLIFIERS_PER_TX * 0x20;
        offsets.publicDataOffset += 2 * Constants.PUBLIC_DATA_WRITES_PER_TX * 0x40;
        offsets.l2ToL1MsgsOffset += 2 * Constants.L2_TO_L1_MSGS_PER_TX * 0x20;
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
    uint256 remainingLogsLength;
    uint256 offset;
    assembly {
      offset := add(_offsetInBlock, _l2Block.offset)
      // Set the remaining logs length to the total logs length
      // Loads 32 bytes from calldata, shifts right by 224 bits and masks the result with 0xffffffff
      remainingLogsLength := and(shr(224, calldataload(offset)), 0xffffffff)
      // Move the calldata offset by the 4 bytes we just read
      offset := add(offset, 0x4)
    }

    bytes32[2] memory logsHashes; // A memory to which we will write the 2 logs hashes to be accumulated
    bytes32 kernelPublicInputsLogsHash; // The hash on the output of kernel iteration
    // The length of the logs emitted by Noir from the function call corresponding to this kernel iteration
    uint256 privateCircuitPublicInputLogsLength;

    // Iterate until all the logs were processed
    while (remainingLogsLength > 0) {
      assembly {
        // Load this iteration's logs length
        privateCircuitPublicInputLogsLength := and(shr(224, calldataload(offset)), 0xffffffff)
        offset := add(offset, 0x4)
      }

      // TODO: Allocating memory in each iteration is expensive. Should we somehow set it to max length of all the
      //       iterations? (e.g. We could do that by first searching for max length in a loop or by modifying
      //       the encoding and storing max length on a predefined position)
      bytes memory privateCircuitPublicInputLogs = new bytes(privateCircuitPublicInputLogsLength);
      assembly {
        // Load logs corresponding to this iteration's function call to memory
        calldatacopy(
          add(privateCircuitPublicInputLogs, 0x20), offset, privateCircuitPublicInputLogsLength
        )
        offset := add(offset, privateCircuitPublicInputLogsLength)
      }

      // Hash the logs
      bytes32 privateCircuitPublicInputsLogsHash = sha256(privateCircuitPublicInputLogs);

      logsHashes[0] = kernelPublicInputsLogsHash;
      logsHashes[1] = privateCircuitPublicInputsLogsHash;
      // Decrease remaining logs length by this privateCircuitPublicInputsLogs's length (len(I?_LOGS)) and 4 bytes for I?_LOGS_LEN
      remainingLogsLength -= (privateCircuitPublicInputLogsLength + 0x4); // 0x4 is the length of the logs length

      // Hash logs hash from the public inputs of previous kernel iteration and logs hash from private circuit public inputs
      kernelPublicInputsLogsHash = sha256(abi.encodePacked(logsHashes));
    }

    uint256 offsetInBlock;
    assembly {
      offsetInBlock := sub(offset, _l2Block.offset)
    }

    return (kernelPublicInputsLogsHash, offsetInBlock);
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
