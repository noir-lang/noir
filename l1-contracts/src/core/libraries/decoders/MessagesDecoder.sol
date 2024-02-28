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
    // L1 to L2 messages
    uint256 count = read4(_body, offset);
    offset += 0x4;

    // `l1ToL2Msgs` is fixed size so if `lengths.l1Tol2MsgsCount` < `Constants.NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP` the array
    // will contain some zero values.
    assembly {
      calldatacopy(add(l1ToL2Msgs, 0x20), add(_body.offset, offset), mul(count, 0x20))
    }

    offset += count * 0x20;

    uint256 numTxs = read4(_body, offset);
    offset += 0x4;

    l2ToL1Msgs = new bytes32[](numTxs * Constants.MAX_NEW_L2_TO_L1_MSGS_PER_TX);

    // Now we iterate over the tx effects
    for (uint256 i = 0; i < numTxs; i++) {
      // Note hashes
      count = read1(_body, offset);
      offset += 0x1;
      offset += count * 0x20; // each note hash is 0x20 bytes long

      // Nullifiers
      count = read1(_body, offset);
      offset += 0x1;
      offset += count * 0x20; // each nullifier is 0x20 bytes long

      // L2 to L1 messages
      {
        count = read1(_body, offset);
        offset += 0x1;

        uint256 msgsLength = count * 0x20; // each l2 to l1 message is 0x20 bytes long

        // Now we copy the new messages into the array (if there are some)
        if (count > 0) {
          uint256 indexInArray = i * Constants.MAX_NEW_L2_TO_L1_MSGS_PER_TX;
          assembly {
            calldatacopy(
              add(add(l2ToL1Msgs, 0x20), mul(indexInArray, 0x20)),
              add(_body.offset, offset),
              msgsLength
            )
          }
        }

        offset += msgsLength;
      }

      // Public data writes
      count = read1(_body, offset);
      offset += 0x1;
      offset += count * 0x40; // each public data write is 0x40 bytes long

      // Contracts
      count = read1(_body, offset);
      offset += 0x1;
      offset += count * 0x20; // each contract leaf is 0x20 bytes long

      // Contract data
      offset += count * 0x34; // each contract data is 0x34 bytes long

      // Encrypted logs
      uint256 length = read4(_body, offset);
      offset += 0x4 + length;

      // Unencrypted logs
      length = read4(_body, offset);
      offset += 0x4 + length;
    }

    inHash = sha256(abi.encodePacked(l1ToL2Msgs));
    outHash = sha256(abi.encodePacked(l2ToL1Msgs));

    return (inHash, outHash, l1ToL2Msgs, l2ToL1Msgs);
  }

  /**
   * @notice Reads 1 bytes from the data
   * @param _data - The data to read from
   * @param _offset - The offset to read from
   * @return The 1 byte as a uint256
   */
  function read1(bytes calldata _data, uint256 _offset) internal pure returns (uint256) {
    return uint256(uint8(bytes1(_data[_offset:_offset + 1])));
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
