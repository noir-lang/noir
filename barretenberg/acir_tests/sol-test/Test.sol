// THIS FILE WILL NOT COMPILE BY ITSELF
// Compilation is handled in `src/index.js` where solcjs gathers the dependencies

pragma solidity >=0.8.4;

import {Verifier} from "./Verifier.sol";

contract Test {
    Verifier verifier;

    constructor() {
       verifier = new Verifier(); 
    }

    function test(bytes calldata proof, bytes32[] calldata publicInputs) view public returns(bool) {
        return verifier.verify(proof, publicInputs);
    }
}

