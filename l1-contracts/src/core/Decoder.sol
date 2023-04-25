// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.18;

/**
 * @title Decoder
 * @author LHerskind
 * @notice Decoding a L2 block, concerned with readability and velocity of development
 * not giving a damn about gas costs.
 * @dev there is currently no padding of the elements, so we are for now assuming nice trees as inputs.
 * Furthermore, if no contract etc are deployed, we expect there to be address(0) for input.
 *
 * -------------------
 * L2 Block Data specification
 * -------------------
 *
 *  | byte start               | num bytes  | name
 *  | ---                      | ---        | ---
 *  | 0x00                     | 0x04       | L2 block number
 *  | 0x04                     | 0x20       | startPrivateDataTreeSnapshot.root
 *  | 0x24                     | 0x04       | startPrivateDataTreeSnapshot.nextAvailableLeafIndex
 *  | 0x28                     | 0x20       | startNullifierTreeSnapshot.root
 *  | 0x48                     | 0x04       | startNullifierTreeSnapshot.nextAvailableLeafIndex
 *  | 0x4c                     | 0x20       | startContractTreeSnapshot.root
 *  | 0x6c                     | 0x04       | startContractTreeSnapshot.nextAvailableLeafIndex
 *  | 0x70                     | 0x20       | startTreeOfHistoricPrivateDataTreeRootsSnapshot.root
 *  | 0x90                     | 0x04       | startTreeOfHistoricPrivateDataTreeRootsSnapshot.nextAvailableLeafIndex
 *  | 0x94                     | 0x20       | startTreeOfHistoricContractTreeRootsSnapshot.root
 *  | 0xb4                     | 0x04       | startTreeOfHistoricContractTreeRootsSnapshot.nextAvailableLeafIndex
 *  | 0xb8                     | 0x20       | endPrivateDataTreeSnapshot.root
 *  | 0xd8                     | 0x04       | endPrivateDataTreeSnapshot.nextAvailableLeafIndex
 *  | 0xdc                     | 0x20       | endNullifierTreeSnapshot.root
 *  | 0xfc                     | 0x04       | endNullifierTreeSnapshot.nextAvailableLeafIndex
 *  | 0x100                    | 0x20       | endContractTreeSnapshot.root
 *  | 0x120                    | 0x04       | endContractTreeSnapshot.nextAvailableLeafIndex
 *  | 0x124                    | 0x20       | endTreeOfHistoricPrivateDataTreeRootsSnapshot.root
 *  | 0x144                    | 0x04       | endTreeOfHistoricPrivateDataTreeRootsSnapshot.nextAvailableLeafIndex
 *  | 0x148                    | 0x20       | endTreeOfHistoricContractTreeRootsSnapshot.root
 *  | 0x168                    | 0x04       | endTreeOfHistoricContractTreeRootsSnapshot.nextAvailableLeafIndex
 *  | 0x16c                    | 0x04       | len(newCommitments) denoted x
 *  | 0x170                    | x          | newCommits
 *  | 0x170 + x                | 0x04       | len(newNullifiers) denoted y
 *  | 0x174 + x                | y          | newNullifiers
 *  | 0x174 + x + y            | 0x04       | len(newContracts) denoted z
 *  | 0x178 + x + y            | z          | newContracts
 *  | 0x178 + x + y + z        | z          | newContractData
 *  |---                       |---         | ---
 *
 */
contract Decoder {
  uint256 internal constant COMMITMENTS_PER_KERNEL = 4;
  uint256 internal constant NULLIFIERS_PER_KERNEL = 4;

  // Prime field order
  uint256 internal constant P =
    21888242871839275222246405745257275088548364400416034343698204186575808495617;

  /**
   * @notice Decodes the inputs and computes values to check state against
   * @param _l2Block - The L2 block calldata.
   * @return l2BlockNumber  - The L2 block number.
   * @return oldStateHash - The state hash expected prior the execution.
   * @return newStateHash - The state hash expected after the execution.
   * @return publicInputHash - The hash of the public inputs
   */
  function _decode(bytes calldata _l2Block)
    internal
    pure
    returns (
      uint256 l2BlockNumber,
      bytes32 oldStateHash,
      bytes32 newStateHash,
      bytes32 publicInputHash
    )
  {
    l2BlockNumber = _getL2BlockNumber(_l2Block);
    // Note, for oldStateHash to match the storage, the l2 block number must be new - 1.
    // Only jumping 1 block at a time.
    oldStateHash = _computeStateHash(l2BlockNumber - 1, 0x4, _l2Block);
    newStateHash = _computeStateHash(l2BlockNumber, 0xb8, _l2Block);
    publicInputHash = _computePublicInputsHash(_l2Block);
  }

  /**
   * Computes a hash of the public inputs from the calldata
   * @param _l2Block - The L2 block calldata.
   * @return sha256(header[0x4:0x16c], diffRoot)
   */
  function _computePublicInputsHash(bytes calldata _l2Block) internal pure returns (bytes32) {
    // Compute the public inputs hash
    // header size - block number + one value for the diffRoot
    uint256 size = 0x16c - 0x04 + 0x20;
    bytes memory temp = new bytes(size);
    assembly {
      calldatacopy(add(temp, 0x20), add(_l2Block.offset, 0x04), size)
    }

    bytes32 diffRoot = _computeDiffRoot(_l2Block);
    assembly {
      mstore(add(temp, add(0x20, sub(0x16c, 0x04))), diffRoot)
    }

    return bytes32(uint256(sha256(temp)) % P);
  }

  /**
   * @notice Extract the L2 block number from the block
   * @param _l2Block - The L2 block calldata
   * @return l2BlockNumber - The L2 block number
   */
  function _getL2BlockNumber(bytes calldata _l2Block) internal pure returns (uint256 l2BlockNumber) {
    assembly {
      l2BlockNumber := and(shr(224, calldataload(_l2Block.offset)), 0xffffffff)
    }
  }

  /**
   * @notice Computes a state hash
   * @param _l2BlockNumber - The L2 block number
   * @param _offset - The offset into the data, 0x04 for old, 0xb8 for next
   * @param _l2Block - The L2 block calldata.
   * @return The state hash
   */
  function _computeStateHash(uint256 _l2BlockNumber, uint256 _offset, bytes calldata _l2Block)
    internal
    pure
    returns (bytes32)
  {
    bytes memory temp = new bytes(0xb8);

    assembly {
      mstore8(add(temp, 0x20), shr(24, _l2BlockNumber))
      mstore8(add(temp, 0x21), shr(16, _l2BlockNumber))
      mstore8(add(temp, 0x22), shr(8, _l2BlockNumber))
      mstore8(add(temp, 0x23), _l2BlockNumber)
    }
    assembly {
      calldatacopy(add(temp, 0x24), add(_l2Block.offset, _offset), 0xb4)
    }

    return sha256(temp);
  }

  struct Vars {
    uint256 commitmentCount;
    uint256 kernelCount;
    uint256 contractCount;
  }

  /**
   * @notice Creates a "diff" tree and compute its root
   * @param _l2Block - The L2 block calldata.
   * @return The root of the "diff" tree
   */
  function _computeDiffRoot(bytes calldata _l2Block) internal pure returns (bytes32) {
    Vars memory vars;
    {
      uint256 commitmentCount;
      assembly {
        commitmentCount := and(shr(224, calldataload(add(_l2Block.offset, 0x16c))), 0xffffffff)
      }
      vars.commitmentCount = commitmentCount;
      vars.kernelCount = commitmentCount / COMMITMENTS_PER_KERNEL;
      uint256 contractCountOffset =
        vars.kernelCount * (COMMITMENTS_PER_KERNEL + NULLIFIERS_PER_KERNEL) * 0x20;

      uint256 newContractCount;
      assembly {
        newContractCount :=
          and(
            shr(224, calldataload(add(_l2Block.offset, add(0x174, contractCountOffset)))), 0xffffffff
          )
      }
      vars.contractCount = newContractCount;
    }

    bytes32[] memory baseLeafs = new bytes32[](vars.kernelCount / 2);

    uint256 dstCommitmentOffset = COMMITMENTS_PER_KERNEL * 0x20 * 0x2;
    uint256 dstContractOffset = dstCommitmentOffset + NULLIFIERS_PER_KERNEL * 0x20 * 0x2;

    uint256 srcCommitmentOffset = 0x170;
    uint256 srcNullifierOffset = 0x174 + vars.commitmentCount * 0x20;
    uint256 srcContractOffset =
      0x178 + (baseLeafs.length * 2 * (NULLIFIERS_PER_KERNEL + COMMITMENTS_PER_KERNEL) * 0x20);
    uint256 srcContractDataOffset = srcContractOffset + vars.contractCount * 0x20;

    for (uint256 i = 0; i < baseLeafs.length; i++) {
      /**
       * Compute the leaf to insert.
       * Leaf_i = (
       *    newNullifiersKernel1,
       *    newNullifiersKernel2,
       *    newCommitmentsKernel1,
       *    newCommitmentsKernel2,
       *    newContractLeafKernel1,
       *    newContractLeafKernel2,
       *    newContractDataKernel1.aztecAddress,
       *    newContractDataKernel1.ethAddress (padded to 32 bytes)
       *    newContractDataKernel2.aztecAddress,
       *    newContractDataKernel2.ethAddress (padded to 32 bytes)
       * );
       * Note that we always read data, the l2Block (atm) must therefore include dummy or zero-notes for
       * Zero values.
       */
      bytes memory baseLeaf = new bytes(0x2c0);

      assembly {
        // Adding new nullifiers
        calldatacopy(add(baseLeaf, 0x20), add(_l2Block.offset, srcNullifierOffset), mul(0x08, 0x20))

        // Adding new commitments
        calldatacopy(
          add(baseLeaf, add(0x20, dstCommitmentOffset)),
          add(_l2Block.offset, srcCommitmentOffset),
          mul(0x08, 0x20)
        )

        // Adding Contract Leafs
        calldatacopy(
          add(baseLeaf, add(0x20, dstContractOffset)),
          add(_l2Block.offset, srcContractOffset),
          mul(2, 0x20)
        )

        // Kernel1.contract.aztecaddress
        calldatacopy(
          add(baseLeaf, add(0x20, add(dstContractOffset, 0x40))),
          add(_l2Block.offset, srcContractDataOffset),
          0x20
        )
        // Kernel1.contract.ethAddress padded to 32 bytes
        calldatacopy(
          add(baseLeaf, add(0x20, add(dstContractOffset, 0x6c))),
          add(_l2Block.offset, add(srcContractDataOffset, 0x20)),
          0x14
        )
        // Kernel2.contract.aztecaddress
        calldatacopy(
          add(baseLeaf, add(0x20, add(dstContractOffset, 0x80))),
          add(_l2Block.offset, add(srcContractDataOffset, 0x34)),
          0x20
        )
        // Kernel2.contract.ethAddress padded to 32 bytes
        calldatacopy(
          add(baseLeaf, add(0x20, add(dstContractOffset, 0xac))),
          add(_l2Block.offset, add(srcContractDataOffset, 0x54)),
          0x14
        )
      }

      srcCommitmentOffset += 2 * COMMITMENTS_PER_KERNEL * 0x20;
      srcNullifierOffset += 2 * NULLIFIERS_PER_KERNEL * 0x20;
      srcContractOffset += 2 * 0x20;
      srcContractDataOffset += 2 * 0x34;

      baseLeafs[i] = sha256(baseLeaf);
    }

    return _computeRoot(baseLeafs);
  }

  /**
   * @notice Computes the root for a binary Merkle-tree given the leafs.
   * @dev Uses sha256.
   * @param _leafs - The 32 bytes leafs to build the tree of.
   * @return The root of the Merkle tree.
   */
  function _computeRoot(bytes32[] memory _leafs) internal pure returns (bytes32) {
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
