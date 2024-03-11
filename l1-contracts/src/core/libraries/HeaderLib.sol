// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

// Libraries
import {Errors} from "./Errors.sol";
import {Constants} from "./ConstantsGen.sol";
import {Hash} from "./Hash.sol";

/**
 * @title Header Library
 * @author Aztec Labs
 * @notice Decoding and validating an L2 block header
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
 *  |                                                                                  |              | Header {
 *  | 0x0000                                                                           | 0x20         |   lastArchive.root
 *  | 0x0020                                                                           | 0x04         |   lastArchive.nextAvailableLeafIndex
 *  |                                                                                  |              |   ContentCommitment {
 *  | 0x0024                                                                           | 0x20         |     txTreeHeight
 *  | 0x0044                                                                           | 0x20         |     txsEffectsHash
 *  | 0x0064                                                                           | 0x20         |     inHash
 *  | 0x0084                                                                           | 0x20         |     outHash
 *  |                                                                                  |              |   StateReference {
 *  | 0x00a4                                                                           | 0x20         |     l1ToL2MessageTree.root
 *  | 0x00c4                                                                           | 0x04         |     l1ToL2MessageTree.nextAvailableLeafIndex
 *  |                                                                                  |              |     PartialStateReference {
 *  | 0x00c8                                                                           | 0x20         |       noteHashTree.root
 *  | 0x00e8                                                                           | 0x04         |       noteHashTree.nextAvailableLeafIndex
 *  | 0x00ec                                                                           | 0x20         |       nullifierTree.root
 *  | 0x010c                                                                           | 0x04         |       nullifierTree.nextAvailableLeafIndex
 *  | 0x0110                                                                           | 0x20         |       publicDataTree.root
 *  | 0x0130                                                                           | 0x04         |       publicDataTree.nextAvailableLeafIndex
 *  |                                                                                  |              |     }
 *  |                                                                                  |              |   }
 *  |                                                                                  |              |   GlobalVariables {
 *  | 0x0134                                                                           | 0x20         |     chainId
 *  | 0x0154                                                                           | 0x20         |     version
 *  | 0x0174                                                                           | 0x20         |     blockNumber
 *  | 0x0194                                                                           | 0x20         |     timestamp
 *  | 0x01b4                                                                           | 0x14         |     coinbase
 *  | 0x01c8                                                                           | 0x20         |     feeRecipient
 *  |                                                                                  |              |   }
 *  |                                                                                  |              | }
 *  | ---                                                                              | ---          | ---
 */
library HeaderLib {
  struct AppendOnlyTreeSnapshot {
    bytes32 root;
    uint32 nextAvailableLeafIndex;
  }

  struct PartialStateReference {
    AppendOnlyTreeSnapshot noteHashTree;
    AppendOnlyTreeSnapshot nullifierTree;
    AppendOnlyTreeSnapshot contractTree;
    AppendOnlyTreeSnapshot publicDataTree;
  }

  struct StateReference {
    AppendOnlyTreeSnapshot l1ToL2MessageTree;
    // Note: Can't use "partial" name here as in yellow paper because it is a reserved solidity keyword
    PartialStateReference partialStateReference;
  }

  struct GlobalVariables {
    uint256 chainId;
    uint256 version;
    uint256 blockNumber;
    uint256 timestamp;
    address coinbase;
    bytes32 feeRecipient;
  }

  struct ContentCommitment {
    uint256 txTreeHeight;
    bytes32 txsEffectsHash;
    bytes32 inHash;
    bytes32 outHash;
  }

  struct Header {
    AppendOnlyTreeSnapshot lastArchive;
    ContentCommitment contentCommitment;
    StateReference stateReference;
    GlobalVariables globalVariables;
  }

  uint256 private constant HEADER_LENGTH = 0x1e8; // Header byte length

  /**
   * @notice Validates the header
   * @param _header - The decoded header
   * @param _version - The expected version
   * @param _lastBlockTs - The timestamp of the last block
   * @param _archive - The expected archive root
   */
  function validate(Header memory _header, uint256 _version, uint256 _lastBlockTs, bytes32 _archive)
    internal
    view
  {
    if (block.chainid != _header.globalVariables.chainId) {
      revert Errors.Rollup__InvalidChainId(_header.globalVariables.chainId, block.chainid);
    }

    if (_header.globalVariables.version != _version) {
      revert Errors.Rollup__InvalidVersion(_header.globalVariables.version, _version);
    }

    // block number already constrained by archive root check

    if (_header.globalVariables.timestamp > block.timestamp) {
      revert Errors.Rollup__TimestampInFuture();
    }

    // @todo @LHerskind consider if this is too strict
    // This will make multiple l2 blocks in the same l1 block impractical.
    // e.g., the first block will update timestamp which will make the second fail.
    // Could possibly allow multiple blocks if in same l1 block
    if (_header.globalVariables.timestamp < _lastBlockTs) {
      revert Errors.Rollup__TimestampTooOld();
    }

    // TODO(#4148) Proper genesis state. If the state is empty, we allow anything for now.
    if (_archive != bytes32(0) && _archive != _header.lastArchive.root) {
      revert Errors.Rollup__InvalidArchive(_archive, _header.lastArchive.root);
    }
  }

  /**
   * @notice Decodes the header
   * @param _header - The header calldata
   * @return The decoded header
   */
  function decode(bytes calldata _header) internal pure returns (Header memory) {
    if (_header.length != HEADER_LENGTH) {
      revert Errors.HeaderLib__InvalidHeaderSize(HEADER_LENGTH, _header.length);
    }

    Header memory header;

    // Reading lastArchive
    header.lastArchive = AppendOnlyTreeSnapshot(
      bytes32(_header[0x0000:0x0020]), uint32(bytes4(_header[0x0020:0x0024]))
    );

    // Reading ContentCommitment
    header.contentCommitment.txTreeHeight = uint256(bytes32(_header[0x0024:0x0044]));
    header.contentCommitment.txsEffectsHash = bytes32(_header[0x0044:0x0064]);
    header.contentCommitment.inHash = bytes32(_header[0x0064:0x0084]);
    header.contentCommitment.outHash = bytes32(_header[0x0084:0x00a4]);

    // Reading StateReference
    header.stateReference.l1ToL2MessageTree = AppendOnlyTreeSnapshot(
      bytes32(_header[0x00a4:0x00c4]), uint32(bytes4(_header[0x00c4:0x00c8]))
    );
    header.stateReference.partialStateReference.noteHashTree = AppendOnlyTreeSnapshot(
      bytes32(_header[0x00c8:0x00e8]), uint32(bytes4(_header[0x00e8:0x00ec]))
    );
    header.stateReference.partialStateReference.nullifierTree = AppendOnlyTreeSnapshot(
      bytes32(_header[0x00ec:0x010c]), uint32(bytes4(_header[0x010c:0x0110]))
    );
    header.stateReference.partialStateReference.publicDataTree = AppendOnlyTreeSnapshot(
      bytes32(_header[0x0110:0x0130]), uint32(bytes4(_header[0x0130:0x0134]))
    );

    // Reading GlobalVariables
    header.globalVariables.chainId = uint256(bytes32(_header[0x0134:0x0154]));
    header.globalVariables.version = uint256(bytes32(_header[0x0154:0x0174]));
    header.globalVariables.blockNumber = uint256(bytes32(_header[0x0174:0x0194]));
    header.globalVariables.timestamp = uint256(bytes32(_header[0x0194:0x01b4]));
    header.globalVariables.coinbase = address(bytes20(_header[0x01b4:0x01c8]));
    header.globalVariables.feeRecipient = bytes32(_header[0x01c8:HEADER_LENGTH]);

    return header;
  }
}
