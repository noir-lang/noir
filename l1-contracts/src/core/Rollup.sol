// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.18;

import {MockVerifier} from "@aztec3/mock/MockVerifier.sol";
import {Decoder} from "./Decoder.sol";

/**
 * @title Rollup
 * @author LHerskind
 * @notice Rollup contract that are concerned about readability and velocity of development
 * not giving a damn about gas costs.
 *
 * Work in progress
 */
contract Rollup is Decoder {
  error InvalidStateHash(bytes32 expected, bytes32 actual);
  error InvalidProof();

  event L2BlockProcessed(uint256 indexed blockNum);

  MockVerifier public immutable VERIFIER;
  bytes32 public rollupStateHash;

  constructor() {
    VERIFIER = new MockVerifier();
  }

  /**
   * @notice Process an incoming L2Block and progress the state
   * @param _proof - The proof of correct execution
   * @param _l2Block - The L2Block data, formatted as outlined in `Decoder.sol`
   */
  function process(bytes memory _proof, bytes calldata _l2Block) external {
    (uint256 l2BlockNumber, bytes32 oldStateHash, bytes32 newStateHash, bytes32 publicInputHash) =
      _decode(_l2Block);

    // @todo Proper genesis state. If the state is empty, we allow anything for now.
    if (rollupStateHash != bytes32(0) && rollupStateHash != oldStateHash) {
      revert InvalidStateHash(rollupStateHash, oldStateHash);
    }

    bytes32[] memory publicInputs = new bytes32[](1);
    publicInputs[0] = publicInputHash;

    if (!VERIFIER.verify(_proof, publicInputs)) {
      revert InvalidProof();
    }

    rollupStateHash = newStateHash;

    emit L2BlockProcessed(l2BlockNumber);
  }
}
