// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Test} from "forge-std/Test.sol";

import {SampleLib} from "../../src/core/sequencer_selection/SampleLib.sol";

// Adding a contract to get some gas-numbers out.
contract Sampler {
  function computeShuffledIndex(uint256 _index, uint256 _indexCount, uint256 _seed)
    public
    pure
    returns (uint256)
  {
    return SampleLib.computeShuffledIndex(_index, _indexCount, _seed);
  }

  function computeOriginalIndex(uint256 _index, uint256 _indexCount, uint256 _seed)
    public
    pure
    returns (uint256)
  {
    return SampleLib.computeOriginalIndex(_index, _indexCount, _seed);
  }

  function computeCommitteeStupid(uint256 _committeeSize, uint256 _indexCount, uint256 _seed)
    public
    pure
    returns (uint256[] memory)
  {
    return SampleLib.computeCommitteeStupid(_committeeSize, _indexCount, _seed);
  }

  function computeCommitteeClever(uint256 _committeeSize, uint256 _indexCount, uint256 _seed)
    public
    pure
    returns (uint256[] memory)
  {
    return SampleLib.computeCommitteeClever(_committeeSize, _indexCount, _seed);
  }
}

contract SamplingTest is Test {
  Sampler sampler = new Sampler();

  function testShuffle() public {
    // Sizes pulled out of thin air
    uint256 setSize = 1024;
    uint256 commiteeSize = 32;

    uint256[] memory indices = new uint256[](setSize);
    for (uint256 i = 0; i < setSize; i++) {
      indices[i] = i;
    }

    uint256[] memory shuffledIndices = new uint256[](setSize);
    uint256 seed = uint256(keccak256(abi.encodePacked("seed1")));

    for (uint256 i = 0; i < setSize; i++) {
      shuffledIndices[i] = sampler.computeShuffledIndex(indices[i], setSize, seed);
      uint256 recoveredIndex = sampler.computeOriginalIndex(shuffledIndices[i], setSize, seed);
      assertEq(recoveredIndex, indices[i], "Invalid index");
    }

    uint256[] memory committee = sampler.computeCommitteeStupid(commiteeSize, setSize, seed);
    uint256[] memory committeeClever = sampler.computeCommitteeClever(commiteeSize, setSize, seed);

    for (uint256 i = 0; i < commiteeSize; i++) {
      assertEq(committee[i], committeeClever[i], "Invalid index");

      uint256 recoveredIndex = sampler.computeOriginalIndex(committee[i], setSize, seed);
      uint256 shuffledIndex = sampler.computeShuffledIndex(recoveredIndex, setSize, seed);

      assertEq(committee[i], shuffledIndex, "Invalid shuffled index");
    }
  }
}
