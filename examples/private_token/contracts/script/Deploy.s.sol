// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/UltraVerifier.sol";
import "../src/PrivateToken.sol";

contract DeployScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        
        vm.startBroadcast(deployerPrivateKey);
        
        // Deploy verifier contracts
        // Note: In production, deploy the actual Noir-generated verifiers
        UltraVerifier transferVerifier = new UltraVerifier();
        console.log("Transfer Verifier deployed to:", address(transferVerifier));
        
        UltraVerifier mintVerifier = new UltraVerifier();
        console.log("Mint Verifier deployed to:", address(mintVerifier));
        
        // Deploy PrivateToken
        PrivateToken token = new PrivateToken(
            address(transferVerifier),
            address(mintVerifier)
        );
        console.log("PrivateToken deployed to:", address(token));
        
        vm.stopBroadcast();
        
        // Output deployment summary
        console.log("");
        console.log("=== Deployment Summary ===");
        console.log("Network: Sepolia");
        console.log("Transfer Verifier:", address(transferVerifier));
        console.log("Mint Verifier:", address(mintVerifier));
        console.log("Private Token:", address(token));
    }
}
