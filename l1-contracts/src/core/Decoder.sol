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
 *  | 0xb8                     | 0x20       | startPublicDataTreeRoot
 *  | 0xd8                     | 0x20       | endPrivateDataTreeSnapshot.root
 *  | 0xf8                     | 0x04       | endPrivateDataTreeSnapshot.nextAvailableLeafIndex
 *  | 0xfc                     | 0x20       | endNullifierTreeSnapshot.root
 *  | 0x11c                    | 0x04       | endNullifierTreeSnapshot.nextAvailableLeafIndex
 *  | 0x120                    | 0x20       | endContractTreeSnapshot.root
 *  | 0x140                    | 0x04       | endContractTreeSnapshot.nextAvailableLeafIndex
 *  | 0x144                    | 0x20       | endTreeOfHistoricPrivateDataTreeRootsSnapshot.root
 *  | 0x164                    | 0x04       | endTreeOfHistoricPrivateDataTreeRootsSnapshot.nextAvailableLeafIndex
 *  | 0x168                    | 0x20       | endTreeOfHistoricContractTreeRootsSnapshot.root
 *  | 0x188                    | 0x04       | endTreeOfHistoricContractTreeRootsSnapshot.nextAvailableLeafIndex
 *  | 0x18c                    | 0x20       | endPublicDataTreeRoot
 *  | 0x1ac                    | 0x04       | len(newCommitments) denoted a
 *  | 0x1b0                    | a          | newCommitments (each element 32 bytes)
 *  | 0x1b0 + a                | 0x04       | len(newNullifiers) denoted b
 *  | 0x1b4 + a                | b          | newNullifiers (each element 32 bytes)
 *  | 0x1b4 + a + b            | 0x04       | len(newPublicDataWrites) denoted c
 *  | 0x1b8 + a + b            | c          | newPublicDataWrites (each element 64 bytes)
 *  | 0x1b8 + a + b + c        | 0x04       | len(newContracts) denoted d
 *  | 0x1bc + a + b + c        | v          | newContracts (each element 32 bytes)
 *  | 0x1bc + a + b + c + d    | v          | newContractData (each element 52 bytes)
 *  |---                       |---         | ---
 * TODO: a,b,c,d are number of elements and not bytes, need to be multiplied by the length of the elements.
 */
contract Decoder {
  uint256 internal constant COMMITMENTS_PER_KERNEL = 4;
  uint256 internal constant NULLIFIERS_PER_KERNEL = 4;
  uint256 internal constant PUBLIC_DATA_WRITES_PER_KERNEL = 4;
  uint256 internal constant CONTRACTS_PER_KERNEL = 1;

  // Prime field order
  uint256 internal constant P =
    21888242871839275222246405745257275088548364400416034343698204186575808495617;

  /**
   * @notice Decodes the inputs and computes values to check state against
   * @param _l2Block - The L2 block calldata.
   * @return l2BlockNumber  - The L2 block number.
   * @return startStateHash - The state hash expected prior the execution.
   * @return endStateHash - The state hash expected after the execution.
   * @return publicInputHash - The hash of the public inputs
   */
  function _decode(bytes calldata _l2Block)
    internal
    pure
    returns (
      uint256 l2BlockNumber,
      bytes32 startStateHash,
      bytes32 endStateHash,
      bytes32 publicInputHash
    )
  {
    l2BlockNumber = _getL2BlockNumber(_l2Block);
    // Note, for startStateHash to match the storage, the l2 block number must be new - 1.
    // Only jumping 1 block at a time.
    startStateHash = _computeStateHash(l2BlockNumber - 1, 0x4, _l2Block);
    endStateHash = _computeStateHash(l2BlockNumber, 0xd8, _l2Block);
    publicInputHash = _computePublicInputsHash(_l2Block);
  }

  /**
   * Computes a hash of the public inputs from the calldata
   * @param _l2Block - The L2 block calldata.
   * @return sha256(header[0x4: 0x1ac], diffRoot)
   */
  function _computePublicInputsHash(bytes calldata _l2Block) internal pure returns (bytes32) {
    // header size - block number size + one value for the diffRoot
    uint256 size = 0x1ac - 0x04 + 0x20;
    bytes memory temp = new bytes(size);
    assembly {
      calldatacopy(add(temp, 0x20), add(_l2Block.offset, 0x04), size)
    }

    bytes32 diffRoot = _computeDiffRoot(_l2Block);
    assembly {
      mstore(add(temp, add(0x20, sub(0x1ac, 0x04))), diffRoot)
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
   * @param _offset - The offset into the data, 0x04 for start, 0xd8 for end
   * @param _l2Block - The L2 block calldata.
   * @return The state hash
   */
  function _computeStateHash(uint256 _l2BlockNumber, uint256 _offset, bytes calldata _l2Block)
    internal
    pure
    returns (bytes32)
  {
    bytes memory temp = new bytes(0xd8);

    assembly {
      mstore8(add(temp, 0x20), shr(24, _l2BlockNumber))
      mstore8(add(temp, 0x21), shr(16, _l2BlockNumber))
      mstore8(add(temp, 0x22), shr(8, _l2BlockNumber))
      mstore8(add(temp, 0x23), _l2BlockNumber)
    }
    assembly {
      // Copy header elements (not including block number) for start or end (size 0xd4)
      calldatacopy(add(temp, 0x24), add(_l2Block.offset, _offset), 0xd4)
    }

    return sha256(temp);
  }

  struct Vars {
    uint256 commitmentCount;
    uint256 nullifierCount;
    uint256 dataWritesCount;
    uint256 contractCount;
  }

  /**
   * @notice Creates a "diff" tree and compute its root
   * @param _l2Block - The L2 block calldata.
   * @return The root of the "diff" tree
   */
  function _computeDiffRoot(bytes calldata _l2Block) internal pure returns (bytes32) {
    // Find the lengths of the different inputs
    Vars memory vars;
    {
      assembly {
        let offset := add(_l2Block.offset, 0x1ac)
        let commitmentCount := and(shr(224, calldataload(offset)), 0xffffffff)
        offset := add(add(offset, 0x4), mul(commitmentCount, 0x20))
        let nullifierCount := and(shr(224, calldataload(offset)), 0xffffffff)
        offset := add(add(offset, 0x4), mul(nullifierCount, 0x20))
        let dataWritesCount := and(shr(224, calldataload(offset)), 0xffffffff)
        offset := add(add(offset, 0x4), mul(nullifierCount, 0x40))
        let contractCount := and(shr(224, calldataload(offset)), 0xffffffff)

        // Store it in vars
        mstore(vars, commitmentCount)
        mstore(add(vars, 0x20), nullifierCount)
        mstore(add(vars, 0x40), dataWritesCount)
        mstore(add(vars, 0x60), contractCount)
      }
    }

    bytes32[] memory baseLeafs = new bytes32[](
            vars.commitmentCount / (COMMITMENTS_PER_KERNEL * 2)
        );

    // Data starts after header. Look at L2 Block Data specification at the top of this file.
    uint256 srcCommitmentOffset = 0x1b0;
    uint256 srcNullifierOffset = srcCommitmentOffset + 0x4 + vars.commitmentCount * 0x20;
    uint256 srcDataOffset = srcNullifierOffset + 0x4 + vars.nullifierCount * 0x20;
    uint256 srcContractOffset = srcDataOffset + 0x4 + vars.dataWritesCount * 0x40;
    uint256 srcContractDataOffset = srcContractOffset + vars.contractCount * 0x20;

    for (uint256 i = 0; i < baseLeafs.length; i++) {
      /**
       * Compute the leaf to insert.
       * Leaf_i = (
       *    newCommitmentsKernel1,
       *    newCommitmentsKernel2,
       *    newNullifiersKernel1,
       *    newNullifiersKernel2,
       *    newPublicDataWritesKernel1,
       *    newPublicDataWritesKernel2,
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
      // Create the leaf to contain commitments (8 * 0x20) + nullifiers (8 * 0x20)
      // + new public data writes (8 * 0x40) + contract deployments (2 * 0x60)
      bytes memory baseLeaf = new bytes(0x4c0);

      assembly {
        let dstOffset := 0x20
        // Adding new commitments
        calldatacopy(
          add(baseLeaf, dstOffset), add(_l2Block.offset, srcCommitmentOffset), mul(0x08, 0x20)
        )
        dstOffset := add(dstOffset, mul(0x08, 0x20))

        // Adding new nullifiers
        calldatacopy(
          add(baseLeaf, dstOffset), add(_l2Block.offset, srcNullifierOffset), mul(0x08, 0x20)
        )
        dstOffset := add(dstOffset, mul(0x08, 0x20))

        // Adding new public data writes
        calldatacopy(add(baseLeaf, dstOffset), add(_l2Block.offset, srcDataOffset), mul(0x08, 0x40))
        dstOffset := add(dstOffset, mul(0x08, 0x40))

        // Adding Contract Leafs
        calldatacopy(
          add(baseLeaf, dstOffset), add(_l2Block.offset, srcContractOffset), mul(2, 0x20)
        )
        dstOffset := add(dstOffset, mul(2, 0x20))

        // Kernel1.contract.aztecAddress
        calldatacopy(add(baseLeaf, dstOffset), add(_l2Block.offset, srcContractDataOffset), 0x20)
        dstOffset := add(dstOffset, 0x20)

        // Kernel1.contract.ethAddress padded to 32 bytes
        // Add 12 (0xc) bytes of padding to the ethAddress
        dstOffset := add(dstOffset, 0xc)
        calldatacopy(
          add(baseLeaf, dstOffset), add(_l2Block.offset, add(srcContractDataOffset, 0x20)), 0x14
        )
        dstOffset := add(dstOffset, 0x20)

        // Kernel2.contract.aztecAddress
        calldatacopy(
          add(baseLeaf, dstOffset), add(_l2Block.offset, add(srcContractDataOffset, 0x34)), 0x20
        )
        dstOffset := add(dstOffset, 0x20)

        // Kernel2.contract.ethAddress padded to 32 bytes
        // Add 12 (0xc) bytes of padding to the ethAddress
        dstOffset := add(dstOffset, 0xc)
        calldatacopy(
          add(baseLeaf, dstOffset), add(_l2Block.offset, add(srcContractDataOffset, 0x54)), 0x14
        )
      }

      srcCommitmentOffset += 2 * COMMITMENTS_PER_KERNEL * 0x20;
      srcNullifierOffset += 2 * NULLIFIERS_PER_KERNEL * 0x20;
      srcDataOffset += 2 * PUBLIC_DATA_WRITES_PER_KERNEL * 0x40;
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
