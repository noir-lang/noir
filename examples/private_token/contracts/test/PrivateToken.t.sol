// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/UltraVerifier.sol";
import "../src/PrivateToken.sol";

contract PrivateTokenTest is Test {
    PrivateToken public token;
    UltraVerifier public transferVerifier;
    UltraVerifier public mintVerifier;
    
    address public owner = address(this);
    address public user1 = address(0x1);
    address public user2 = address(0x2);
    
    bytes32 public commitment1 = keccak256("commitment1");
    bytes32 public commitment2 = keccak256("commitment2");
    bytes32 public commitment3 = keccak256("commitment3");
    bytes32 public nullifier1 = keccak256("nullifier1");
    
    function setUp() public {
        transferVerifier = new UltraVerifier();
        mintVerifier = new UltraVerifier();
        token = new PrivateToken(address(transferVerifier), address(mintVerifier));
    }
    
    function testMint() public {
        bytes memory proof = hex"1234";
        bytes32[] memory publicInputs = new bytes32[](2);
        publicInputs[0] = commitment1;
        publicInputs[1] = bytes32(uint256(1)); // request ID
        
        vm.prank(user1);
        token.mint(proof, publicInputs);
        
        assertTrue(token.hasCommitment(commitment1));
        assertEq(token.getCommitmentCount(), 1);
    }
    
    function testMintDuplicateCommitmentFails() public {
        bytes memory proof = hex"1234";
        bytes32[] memory publicInputs = new bytes32[](2);
        publicInputs[0] = commitment1;
        publicInputs[1] = bytes32(uint256(1));
        
        token.mint(proof, publicInputs);
        
        vm.expectRevert(PrivateToken.CommitmentAlreadyExists.selector);
        token.mint(proof, publicInputs);
    }
    
    function testTransfer() public {
        // First mint to create input commitment
        bytes memory mintProof = hex"1234";
        bytes32[] memory mintInputs = new bytes32[](2);
        mintInputs[0] = commitment1;
        mintInputs[1] = bytes32(uint256(1));
        token.mint(mintProof, mintInputs);
        
        // Now transfer
        bytes memory transferProof = hex"5678";
        bytes32[] memory transferInputs = new bytes32[](5);
        transferInputs[0] = commitment1; // input commitment
        transferInputs[1] = commitment2; // sender output
        transferInputs[2] = commitment3; // recipient output
        transferInputs[3] = nullifier1;  // nullifier
        transferInputs[4] = bytes32(uint256(2)); // new nonce
        
        token.transfer(transferProof, transferInputs);
        
        assertTrue(token.hasCommitment(commitment2));
        assertTrue(token.hasCommitment(commitment3));
        assertTrue(token.isNullifierUsed(nullifier1));
        assertEq(token.getCommitmentCount(), 3);
    }
    
    function testTransferDoubleSpendFails() public {
        // Mint
        bytes memory mintProof = hex"1234";
        bytes32[] memory mintInputs = new bytes32[](2);
        mintInputs[0] = commitment1;
        mintInputs[1] = bytes32(uint256(1));
        token.mint(mintProof, mintInputs);
        
        // First transfer
        bytes memory transferProof = hex"5678";
        bytes32[] memory transferInputs = new bytes32[](5);
        transferInputs[0] = commitment1;
        transferInputs[1] = commitment2;
        transferInputs[2] = commitment3;
        transferInputs[3] = nullifier1;
        transferInputs[4] = bytes32(uint256(2));
        token.transfer(transferProof, transferInputs);
        
        // Try to use same nullifier again (double spend)
        bytes32 commitment4 = keccak256("commitment4");
        bytes32 commitment5 = keccak256("commitment5");
        transferInputs[0] = commitment1;
        transferInputs[1] = commitment4;
        transferInputs[2] = commitment5;
        transferInputs[3] = nullifier1; // Same nullifier
        transferInputs[4] = bytes32(uint256(3));
        
        vm.expectRevert(PrivateToken.NullifierAlreadyUsed.selector);
        token.transfer(transferProof, transferInputs);
    }
    
    function testTransferNonExistentCommitmentFails() public {
        bytes memory transferProof = hex"5678";
        bytes32[] memory transferInputs = new bytes32[](5);
        transferInputs[0] = commitment1; // Non-existent commitment
        transferInputs[1] = commitment2;
        transferInputs[2] = commitment3;
        transferInputs[3] = nullifier1;
        transferInputs[4] = bytes32(uint256(2));
        
        vm.expectRevert(PrivateToken.CommitmentNotFound.selector);
        token.transfer(transferProof, transferInputs);
    }
    
    function testSetVerifier() public {
        UltraVerifier newVerifier = new UltraVerifier();
        
        token.setTransferVerifier(address(newVerifier));
        assertEq(address(token.transferVerifier()), address(newVerifier));
    }
    
    function testSetVerifierNotOwnerFails() public {
        UltraVerifier newVerifier = new UltraVerifier();
        
        vm.prank(user1);
        vm.expectRevert(PrivateToken.OnlyOwner.selector);
        token.setTransferVerifier(address(newVerifier));
    }
}
