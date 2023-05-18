pragma solidity ^0.8.18;

import "forge-std/Test.sol";
import {PortalERC20} from "./PortalERC20.sol";

contract PortalERC20Test is Test {
  PortalERC20 portalERC20;

  function setUp() public {
    portalERC20 = new PortalERC20();
  }

  function test_mint() public {
    portalERC20.mint(address(this), 100);
    assertEq(portalERC20.balanceOf(address(this)), 100);
  }
}
