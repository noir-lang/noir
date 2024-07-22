// TODO: doc
pragma solidity ^0.8.21;

type Fr is uint256;

using {add as +} for Fr global;
using {sub as -} for Fr global;
using {mul as *} for Fr global;

// Yuck using ^ for exp  - todo maybe make it manual
using {exp as ^} for Fr global;
using {notEqual as !=} for Fr global;
using {equal as ==} for Fr global;

uint256 constant MODULUS = 21888242871839275222246405745257275088548364400416034343698204186575808495617; // Prime field order

// Instantiation
library FrLib {
    function from(uint256 value) internal pure returns (Fr) {
        return Fr.wrap(value % MODULUS);
    }

    function fromBytes32(bytes32 value) internal pure returns (Fr) {
        return Fr.wrap(uint256(value) % MODULUS);
    }

    function toBytes32(Fr value) internal pure returns (bytes32) {
        return bytes32(Fr.unwrap(value));
    }

    function invert(Fr value) internal view returns (Fr) {
        uint256 v = Fr.unwrap(value);
        uint256 result;

        // Call the modexp precompile to invert in the field
        assembly {
            let free := mload(0x40)
            mstore(free, 0x20)
            mstore(add(free, 0x20), 0x20)
            mstore(add(free, 0x40), 0x20)
            mstore(add(free, 0x60), v)
            mstore(add(free, 0x80), sub(MODULUS, 2)) // TODO: check --via-ir will compiler inline
            mstore(add(free, 0xa0), MODULUS)
            let success := staticcall(gas(), 0x05, free, 0xc0, 0x00, 0x20)
            if iszero(success) {
                // TODO: meaningful error
                revert(0, 0)
            }
            result := mload(0x00)
        }

        return Fr.wrap(result);
    }

    // TODO: edit other pow, it only works for powers of two
    function pow(Fr base, uint256 v) internal view returns (Fr) {
        uint256 b = Fr.unwrap(base);
        uint256 result;

        // Call the modexp precompile to invert in the field
        assembly {
            let free := mload(0x40)
            mstore(free, 0x20)
            mstore(add(free, 0x20), 0x20)
            mstore(add(free, 0x40), 0x20)
            mstore(add(free, 0x60), b)
            mstore(add(free, 0x80), v) // TODO: check --via-ir will compiler inline
            mstore(add(free, 0xa0), MODULUS)
            let success := staticcall(gas(), 0x05, free, 0xc0, 0x00, 0x20)
            if iszero(success) {
                // TODO: meaningful error
                revert(0, 0)
            }
            result := mload(0x00)
        }

        return Fr.wrap(result);
    }

    // TODO: Montgomery's batch inversion trick
    function div(Fr numerator, Fr denominator) internal view returns (Fr) {
        Fr inversion = invert(denominator);
        return numerator * invert(denominator);
    }
}

// Free functions
function add(Fr a, Fr b) pure returns (Fr) {
    return Fr.wrap(addmod(Fr.unwrap(a), Fr.unwrap(b), MODULUS));
}

function mul(Fr a, Fr b) pure returns (Fr) {
    return Fr.wrap(mulmod(Fr.unwrap(a), Fr.unwrap(b), MODULUS));
}

function sub(Fr a, Fr b) pure returns (Fr) {
    return Fr.wrap(addmod(Fr.unwrap(a), MODULUS - Fr.unwrap(b), MODULUS));
}

// TODO: double check this !
function exp(Fr base, Fr exponent) pure returns (Fr) {
    if (Fr.unwrap(exponent) == 0) return Fr.wrap(1);
    // Implement exponent with a loop as we will overflow otherwise
    for (uint256 i = 1; i < Fr.unwrap(exponent); i += i) {
        base = base * base;
    }
    return base;
}

function notEqual(Fr a, Fr b) pure returns (bool) {
    return Fr.unwrap(a) != Fr.unwrap(b);
}

function equal(Fr a, Fr b) pure returns (bool) {
    return Fr.unwrap(a) == Fr.unwrap(b);
}
