// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

// Libraries
import {Constants} from "../ConstantsGen.sol";
import {Hash} from "../Hash.sol";

/**
 * @title Header Decoder Library
 * @author Aztec Labs
 * @notice Decoding a L2 header
 * Concerned with readability and velocity of development not giving a damn about gas costs.
 *
 * -------------------
 * You can use https://gist.github.com/LHerskind/724a7e362c97e8ac2902c6b961d36830 to generate the below outline.
 * -------------------
 * L2 Block Header specification
 * -------------------
 *
 *  | byte start                                                                       | num bytes    | name
 *  | ---                                                                              | ---          | ---
 *  | 0x0000                                                                           | 0x20         | chain-id
 *  | 0x0020                                                                           | 0x20         | version
 *  | 0x0040                                                                           | 0x20         | L2 block number
 *  | 0x0060                                                                           | 0x20         | L2 timestamp
 *  | 0x0080                                                                           | 0x20         | startNoteHashTreeSnapshot.root
 *  | 0x00a0                                                                           | 0x04         | startNoteHashTreeSnapshot.nextAvailableLeafIndex
 *  | 0x00a4                                                                           | 0x20         | startNullifierTreeSnapshot.root
 *  | 0x00c4                                                                           | 0x04         | startNullifierTreeSnapshot.nextAvailableLeafIndex
 *  | 0x00c8                                                                           | 0x20         | startContractTreeSnapshot.root
 *  | 0x00e8                                                                           | 0x04         | startContractTreeSnapshot.nextAvailableLeafIndex
 *  | 0x00ec                                                                           | 0x20         | startPublicDataTreeSnapshot.root
 *  | 0x010c                                                                           | 0x04         | startPublicDataTreeSnapshot.nextAvailableLeafIndex
 *  | 0x0110                                                                           | 0x20         | startL1ToL2MessageTreeSnapshot.root
 *  | 0x0130                                                                           | 0x04         | startL1ToL2MessageTreeSnapshot.nextAvailableLeafIndex
 *  | 0x0134                                                                           | 0x20         | startArchiveSnapshot.root
 *  | 0x0154                                                                           | 0x04         | startArchiveSnapshot.nextAvailableLeafIndex
 *  | 0x0158                                                                           | 0x20         | endNoteHashTreeSnapshot.root
 *  | 0x0178                                                                           | 0x04         | endNoteHashTreeSnapshot.nextAvailableLeafIndex
 *  | 0x017c                                                                           | 0x20         | endNullifierTreeSnapshot.root
 *  | 0x019c                                                                           | 0x04         | endNullifierTreeSnapshot.nextAvailableLeafIndex
 *  | 0x01a0                                                                           | 0x20         | endContractTreeSnapshot.root
 *  | 0x01c0                                                                           | 0x04         | endContractTreeSnapshot.nextAvailableLeafIndex
 *  | 0x01c4                                                                           | 0x20         | endPublicDataTreeSnapshot.root
 *  | 0x01e4                                                                           | 0x04         | endPublicDataTreeSnapshot.nextAvailableLeafIndex
 *  | 0x01e8                                                                           | 0x20         | endL1ToL2MessageTreeSnapshot.root
 *  | 0x0208                                                                           | 0x04         | endL1ToL2MessageTreeSnapshot.nextAvailableLeafIndex
 *  | 0x020c                                                                           | 0x20         | endArchiveSnapshot.root
 *  | 0x022c                                                                           | 0x04         | endArchiveSnapshot.nextAvailableLeafIndex
 *  | ---                                                                              | ---          | ---
 */
library HeaderDecoder {
  // DECODING OFFSET CONSTANTS
  // Where the start of trees metadata begins in the block
  uint256 private constant START_TREES_BLOCK_HEADER_OFFSET = 0x80;

  // The size of the block header elements
  uint256 private constant TREES_BLOCK_HEADER_SIZE = 0xd8;

  // Where the end of trees metadata begins in the block
  uint256 private constant END_TREES_BLOCK_HEADER_OFFSET =
    START_TREES_BLOCK_HEADER_OFFSET + TREES_BLOCK_HEADER_SIZE;

  // Where the metadata ends and the block data begins.
  uint256 internal constant BLOCK_HEADER_SIZE =
    START_TREES_BLOCK_HEADER_OFFSET + 2 * TREES_BLOCK_HEADER_SIZE;

  /**
   * @notice Decodes the header
   * @param _header - The L2 block calldata.
   * @return l2BlockNumber - The L2 block number
   * @return startStateHash - The start state hash
   * @return endStateHash - The end state hash
   */
  function decode(bytes calldata _header)
    internal
    pure
    returns (uint256 l2BlockNumber, bytes32 startStateHash, bytes32 endStateHash)
  {
    l2BlockNumber = uint256(bytes32(_header[0x40:0x60]));
    // Note, for startStateHash to match the storage, the l2 block number must be new - 1.
    // Only jumping 1 block at a time.
    startStateHash = computeStateHash(l2BlockNumber - 1, START_TREES_BLOCK_HEADER_OFFSET, _header);
    endStateHash = computeStateHash(l2BlockNumber, END_TREES_BLOCK_HEADER_OFFSET, _header);
  }

  /**
   * @notice Computes a state hash
   * @param _l2BlockNumber - The L2 block number
   * @param _offset - The offset into the data, 0x80 for start, 0x019c for end
   * @param _header - The L2 block calldata.
   * @return The state hash
   * @dev The state hash is sha256 hash of block's header elements. For each block the header elements are
   *      the block number, snapshots of all the trees and the root of the public data tree. This function
   *      copies all of these to memory and then hashes them.
   */
  function computeStateHash(uint256 _l2BlockNumber, uint256 _offset, bytes calldata _header)
    internal
    pure
    returns (bytes32)
  {
    return sha256(
      bytes.concat(bytes32(_l2BlockNumber), _header[_offset:_offset + TREES_BLOCK_HEADER_SIZE])
    );
  }
}
