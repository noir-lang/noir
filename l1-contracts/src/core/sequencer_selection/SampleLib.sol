// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

import {Errors} from "../libraries/Errors.sol";

/**
 * @title   SampleLib
 * @author  Anaxandridas II
 * @notice  A tiny library to shuffle indices using the swap-or-not algorithm and then
 *          draw a committee from the shuffled indices.
 *
 * @dev     Using the `swap-or-not` alogirthm that is used by Ethereum consensus client.
 *          We are using this algorithm, since it can compute a shuffle of individual indices,
 *          which will be very useful for EVENTUALLY reducing the cost of committee selection.
 *
 *          Currently the library is maximally simple, and will simply do "dumb" sampling to select
 *          a committee, but re-use parts of computation to improve efficiency.
 *
 *          https://eth2book.info/capella/part2/building_blocks/shuffling/
 *          https://link.springer.com/content/pdf/10.1007%2F978-3-642-32009-5_1.pdf
 */
library SampleLib {
  /**
   * @notice  Computes the shuffled index
   *
   * @param _index - The index to shuffle
   * @param _indexCount - The total number of indices
   * @param _seed - The seed to use for shuffling
   */
  function computeShuffledIndex(uint256 _index, uint256 _indexCount, uint256 _seed)
    internal
    pure
    returns (uint256)
  {
    if (_index >= _indexCount) {
      revert Errors.SampleLib__IndexOutOfBounds(_index, _indexCount);
    }
    uint256 rounds = computeShuffleRounds(_indexCount);

    uint256 index = _index;

    for (uint256 currentRound = 0; currentRound < rounds; currentRound++) {
      uint256 pivot = computePivot(_seed, currentRound, _indexCount);
      index = computeInner(_seed, pivot, index, currentRound, _indexCount);
    }

    return index;
  }

  /**
   * @notice  Computes the original index from a shuffled index
   *
   * @dev     Notice, that we are using same logic as `computeShuffledIndex` just looping in reverse.
   *
   * @param _shuffledIndex - The shuffled index
   * @param _indexCount - The total number of indices
   * @param _seed - The seed to use for shuffling
   *
   * @return index - The original index
   */
  function computeOriginalIndex(uint256 _shuffledIndex, uint256 _indexCount, uint256 _seed)
    internal
    pure
    returns (uint256)
  {
    if (_shuffledIndex >= _indexCount) {
      revert Errors.SampleLib__IndexOutOfBounds(_shuffledIndex, _indexCount);
    }

    uint256 rounds = computeShuffleRounds(_indexCount);

    uint256 index = _shuffledIndex;

    for (uint256 currentRound = rounds; currentRound > 0; currentRound--) {
      uint256 pivot = computePivot(_seed, currentRound - 1, _indexCount);
      index = computeInner(_seed, pivot, index, currentRound - 1, _indexCount);
    }

    return index;
  }

  /**
   * @notice  Computing a committee the most direct way.
   *          This is horribly inefficient as we are throwing plenty of things away, but it is useful
   *          for testing and just showcasing the simplest case.
   *
   * @param _committeeSize - The size of the committee
   * @param _indexCount - The total number of indices
   * @param _seed - The seed to use for shuffling
   *
   * @return indices - The indices of the committee
   */
  function computeCommitteeStupid(uint256 _committeeSize, uint256 _indexCount, uint256 _seed)
    internal
    pure
    returns (uint256[] memory)
  {
    uint256[] memory indices = new uint256[](_committeeSize);

    for (uint256 index = 0; index < _indexCount; index++) {
      uint256 sampledIndex = computeShuffledIndex(index, _indexCount, _seed);
      if (sampledIndex < _committeeSize) {
        indices[sampledIndex] = index;
      }
    }

    return indices;
  }

  /**
   * @notice  Computing a committee slightly more cleverly.
   *          Only computes for the committee size, and does not sample the full set.
   *          This is more efficient than the stupid way, but still not optimal.
   *          To be more clever, we can compute the `shuffeRounds` and `pivots` separately
   *          such that they get shared accross multiple indices.
   *
   * @param _committeeSize - The size of the committee
   * @param _indexCount - The total number of indices
   * @param _seed - The seed to use for shuffling
   *
   * @return indices - The indices of the committee
   */
  function computeCommitteeClever(uint256 _committeeSize, uint256 _indexCount, uint256 _seed)
    internal
    pure
    returns (uint256[] memory)
  {
    uint256[] memory indices = new uint256[](_committeeSize);

    for (uint256 index = 0; index < _committeeSize; index++) {
      uint256 originalIndex = computeOriginalIndex(index, _indexCount, _seed);
      indices[index] = originalIndex;
    }

    return indices;
  }

  /**
   * @notice  Compute the number of shuffle rounds
   *
   * @dev     A safe number of rounds should be 4 * log_2 N where N is the number of indices
   *
   * @param _count - The number of indices
   *
   * @return rounds - The number of rounds to shuffle
   */
  function computeShuffleRounds(uint256 _count) private pure returns (uint256) {
    return log2(_count) * 4;
  }

  /**
   * @notice  Computes the pivot for a given round
   *
   * @param _seed - The seed to use for shuffling
   * @param _currentRound - The current round of shuffling
   * @param _indexCount - The total number of indices
   *
   * @return pivot - The pivot for the round
   */
  function computePivot(uint256 _seed, uint256 _currentRound, uint256 _indexCount)
    private
    pure
    returns (uint256)
  {
    return uint256(keccak256(abi.encodePacked(_seed, uint8(_currentRound)))) % _indexCount;
  }

  /**
   * @notice Computes the inner loop (one round) of a shuffle
   *
   * @param _seed - The seed to use for shuffling
   * @param _pivot - The pivot to use for shuffling
   * @param _index - The index to shuffle
   * @param _currentRound - The current round of shuffling
   * @param _indexCount - The total number of indices
   *
   * @return index - The shuffled index
   */
  function computeInner(
    uint256 _seed,
    uint256 _pivot,
    uint256 _index,
    uint256 _currentRound,
    uint256 _indexCount
  ) private pure returns (uint256) {
    uint256 flip = (_pivot + _indexCount - _index) % _indexCount;
    uint256 position = _index > flip ? _index : flip;
    bytes32 source =
      keccak256(abi.encodePacked(_seed, uint8(_currentRound), uint32(position / 256)));
    uint8 byte_ = uint8(source[(position % 256) / 8]);
    uint8 bit_ = (byte_ >> (position % 8)) % 2;

    return bit_ == 1 ? flip : _index;
  }

  /**
   * @notice  Computes the log2 of a uint256 number
   *
   * @param x - The number to compute the log2 of
   *
   * @return y - The log2 of the number
   */
  function log2(uint256 x) private pure returns (uint256 y) {
    // https://graphics.stanford.edu/~seander/bithacks.html#IntegerLogDeBruijn
    assembly {
      let arg := x
      x := sub(x, 1)
      x := or(x, div(x, 0x02))
      x := or(x, div(x, 0x04))
      x := or(x, div(x, 0x10))
      x := or(x, div(x, 0x100))
      x := or(x, div(x, 0x10000))
      x := or(x, div(x, 0x100000000))
      x := or(x, div(x, 0x10000000000000000))
      x := or(x, div(x, 0x100000000000000000000000000000000))
      x := add(x, 1)
      let m := mload(0x40)
      mstore(m, 0xf8f9cbfae6cc78fbefe7cdc3a1793dfcf4f0e8bbd8cec470b6a28a7a5a3e1efd)
      mstore(add(m, 0x20), 0xf5ecf1b3e9debc68e1d9cfabc5997135bfb7a7a3938b7b606b5b4b3f2f1f0ffe)
      mstore(add(m, 0x40), 0xf6e4ed9ff2d6b458eadcdf97bd91692de2d4da8fd2d0ac50c6ae9a8272523616)
      mstore(add(m, 0x60), 0xc8c0b887b0a8a4489c948c7f847c6125746c645c544c444038302820181008ff)
      mstore(add(m, 0x80), 0xf7cae577eec2a03cf3bad76fb589591debb2dd67e0aa9834bea6925f6a4a2e0e)
      mstore(add(m, 0xa0), 0xe39ed557db96902cd38ed14fad815115c786af479b7e83247363534337271707)
      mstore(add(m, 0xc0), 0xc976c13bb96e881cb166a933a55e490d9d56952b8d4e801485467d2362422606)
      mstore(add(m, 0xe0), 0x753a6d1b65325d0c552a4d1345224105391a310b29122104190a110309020100)
      mstore(0x40, add(m, 0x100))
      let magic := 0x818283848586878898a8b8c8d8e8f929395969799a9b9d9e9faaeb6bedeeff
      let shift := 0x100000000000000000000000000000000000000000000000000000000000000
      let a := div(mul(x, magic), shift)
      y := div(mload(add(m, sub(255, a))), shift)
      y :=
        add(y, mul(256, gt(arg, 0x8000000000000000000000000000000000000000000000000000000000000000)))
    }
  }
}
