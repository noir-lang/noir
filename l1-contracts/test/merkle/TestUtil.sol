// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

import {Test} from "forge-std/Test.sol";

contract MerkleTestUtil is Test {
  /*
  * @notice Calculates a tree height from the amount of elements in the tree
  * @param _size - The amount of elements in the tree
  */
  function calculateTreeHeightFromSize(uint256 _size) public pure returns (uint256) {
    /// The code / formula that works below has one edge case at _size = 1, which we handle here
    if (_size == 1) {
      return 1;
    }

    /// We need to store the original numer to check at the end if we are a power of two
    uint256 originalNumber = _size;

    /// We need the height of the tree that will contain all of our leaves,
    /// hence the next highest power of two from the amount of leaves - Math.ceil(Math.log2(x))
    uint256 height = 0;

    /// While size > 1, we divide by two, and count how many times we do this; producing a rudimentary way of calculating Math.Floor(Math.log2(x))
    while (_size > 1) {
      _size >>= 1;
      height++;
    }

    /// @notice - We check if 2 ** height does not equal our original number. If so, this means that our size is not a power of two,
    /// and hence we've rounded down (Math.floor) and have obtained the next lowest power of two instead of rounding up (Math.ceil) to obtain the next highest power of two and therefore we need to increment height before returning it.
    /// If 2 ** height equals our original number, it means that we have a perfect power of two and Math.floor(Math.log2(x)) = Math.ceil(Math.log2(x)) and we can return height as-is
    return (2 ** height) != originalNumber ? ++height : height;
  }

  function testCalculateTreeHeightFromSize() external {
    assertEq(calculateTreeHeightFromSize(0), 1);
    assertEq(calculateTreeHeightFromSize(1), 1);
    assertEq(calculateTreeHeightFromSize(2), 1);
    assertEq(calculateTreeHeightFromSize(3), 2);
    assertEq(calculateTreeHeightFromSize(4), 2);
    assertEq(calculateTreeHeightFromSize(5), 3);
    assertEq(calculateTreeHeightFromSize(6), 3);
    assertEq(calculateTreeHeightFromSize(7), 3);
    assertEq(calculateTreeHeightFromSize(8), 3);
    assertEq(calculateTreeHeightFromSize(9), 4);
  }
}
