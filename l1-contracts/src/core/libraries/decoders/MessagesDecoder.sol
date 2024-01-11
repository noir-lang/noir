// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

// Libraries
import {Constants} from "../ConstantsGen.sol";
import {Hash} from "../Hash.sol";

/**
 * @title Messages Decoder Library
 * @author Aztec Labs
 * @notice Decoding a L2 block body and returns cross-chain messages + (in/out)Hash.
 * Concerned with readability and velocity of development not giving a damn about gas costs.
 * @dev Assumes the input trees to be padded.
 *
 * -------------------
 * You can use https://gist.github.com/LHerskind/724a7e362c97e8ac2902c6b961d36830 to generate the below outline.
 * -------------------
 * L2 Body Data specification
 * -------------------
 *
 *  | byte start                                                                     | num bytes    | name
 *  | ---                                                                            | ---          | ---
 *  | 0x00                                                                           | 0x04         | len(newCommitments) (denoted a)
 *  | 0x04                                                                           | a * 0x20     | newCommitments
 *  | 0x04 + a * 0x20                                                                | 0x04         | len(newNullifiers) (denoted b)
 *  | 0x08 + a * 0x20                                                                | b * 0x20     | newNullifiers
 *  | 0x08 + a * 0x20 + b * 0x20                                                     | 0x04         | len(newPublicDataWrites) (denoted c)
 *  | 0x0c + a * 0x20 + b * 0x20                                                     | c * 0x40     | newPublicDataWrites
 *  | 0x0c + a * 0x20 + b * 0x20 + c * 0x40                                          | 0x04         | len(newL2ToL1Msgs) (denoted d)
 *  | 0x10 + a * 0x20 + b * 0x20 + c * 0x40                                          | d * 0x20     | newL2ToL1Msgs
 *  | 0x10 + a * 0x20 + b * 0x20 + c * 0x40 + d * 0x20                               | 0x04         | len(contracts) (denoted e)
 *  | 0x14 + a * 0x20 + b * 0x20 + c * 0x40 + d * 0x20                               | e * 0x20     | newContracts
 *  | 0x14 + a * 0x20 + b * 0x20 + c * 0x40 + d * 0x20 + e * 0x20                    | e * 0x34     | newContractsData
 *  | 0x14 + a * 0x20 + b * 0x20 + c * 0x40 + d * 0x20 + e * 0x54                    | 0x04         | len(newL1ToL2Msgs) (denoted f)
 *  | 0x18 + a * 0x20 + b * 0x20 + c * 0x40 + d * 0x20 + e * 0x54                    | f * 0x20     | newL1ToL2Msgs
 *  | 0x18 + a * 0x20 + b * 0x20 + c * 0x40 + d * 0x20 + e * 0x54 + f * 0x20         | 0x04         | byteLen(newEncryptedLogs) (denoted g)
 *  | 0x1c + a * 0x20 + b * 0x20 + c * 0x40 + d * 0x20 + e * 0x54 + f * 0x20         | g            | newEncryptedLogs
 *  | 0x1c + a * 0x20 + b * 0x20 + c * 0x40 + d * 0x20 + e * 0x54 + f * 0x20 + g     | 0x04         | byteLen(newUnencryptedLogs) (denoted h)
 *  | 0x20 + a * 0x20 + b * 0x20 + c * 0x40 + d * 0x20 + e * 0x54 + f * 0x20 + g     | h            | newUnencryptedLogs
 *  | ---                                                                            | ---          | ---
 */
library MessagesDecoder {
  /**
   * @notice Computes consumables for the block
   * @param _body - The L2 block calldata.
   * @return inHash - The hash of the L1 to L2 messages
   * @return outHash - The hash of the L1 to L2 messages
   * @return l1ToL2Msgs - The L1 to L2 messages of the block
   * @return l2ToL1Msgs - The L2 to L1 messages of the block
   */
  function decode(bytes calldata _body)
    internal
    pure
    returns (
      bytes32 inHash,
      bytes32 outHash,
      bytes32[] memory l1ToL2Msgs,
      bytes32[] memory l2ToL1Msgs
    )
  {
    l1ToL2Msgs = new bytes32[](Constants.NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP);

    uint256 offset = 0;

    // Commitments
    uint256 count = read4(_body, offset);
    offset += 0x4 + count * 0x20;

    // Nullifiers
    count = read4(_body, offset);
    offset += 0x4 + count * 0x20;

    // Public data writes
    count = read4(_body, offset);
    offset += 0x4 + count * 0x40;

    // L2 to L1 messages
    count = read4(_body, offset);
    l2ToL1Msgs = new bytes32[](count);
    assembly {
      calldatacopy(add(l2ToL1Msgs, 0x20), add(_body.offset, add(offset, 0x4)), mul(count, 0x20))
    }
    offset += 0x4 + count * 0x20;

    // Contracts
    count = read4(_body, offset);
    offset += 0x4 + count * 0x54;

    // L1 to L2 messages
    count = read4(_body, offset);
    // `l1ToL2Msgs` is fixed size so if `lengths.l1Tol2MsgsCount` < `Constants.NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP` the array
    // will contain some zero values.
    assembly {
      calldatacopy(add(l1ToL2Msgs, 0x20), add(_body.offset, add(offset, 0x04)), mul(count, 0x20))
    }

    inHash = sha256(abi.encodePacked(l1ToL2Msgs));
    outHash = sha256(abi.encodePacked(l2ToL1Msgs));

    return (inHash, outHash, l1ToL2Msgs, l2ToL1Msgs);
  }

  /**
   * @notice Reads 4 bytes from the data
   * @param _data - The data to read from
   * @param _offset - The offset to read from
   * @return The 4 bytes read as a uint256
   */
  function read4(bytes calldata _data, uint256 _offset) internal pure returns (uint256) {
    return uint256(uint32(bytes4(_data[_offset:_offset + 4])));
  }
}
