// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec.
pragma solidity >=0.8.4;

import {Test} from "forge-std/Test.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";

contract TestBase is Test {
    using Strings for uint256;

    function readProofData(string memory path) internal view returns (bytes memory) {
        // format [4 byte length][data]
        // Reads the raw bytes
        bytes memory rawBytes = vm.readFileBinary(path);

        // Extract the [data], contains inputs and proof
        bytes memory proofData = new bytes(rawBytes.length - 4); //
        assembly {
            let length := shr(224, mload(add(rawBytes, 0x20)))

            let wLoc := add(proofData, 0x20)
            let rLoc := add(rawBytes, 0x24)
            let end := add(rLoc, length)

            for {} lt(rLoc, end) {
                wLoc := add(wLoc, 0x20)
                rLoc := add(rLoc, 0x20)
            } { mstore(wLoc, mload(rLoc)) }
        }
        return proofData;
    }

    function splitProof(bytes memory _proofData, uint256 _numberOfPublicInputs)
        internal
        view
        returns (bytes32[] memory publicInputs, bytes memory proof)
    {
        publicInputs = new bytes32[](_numberOfPublicInputs);
        for (uint256 i = 0; i < _numberOfPublicInputs; i++) {
            // The proofs spit out by barretenberg have the public inputs at the beginning
            publicInputs[i] = readWordByIndex(_proofData, i);
        }

        proof = new bytes(_proofData.length - (_numberOfPublicInputs * 0x20));
        uint256 len = proof.length;
        assembly {
            pop(
                staticcall(
                    gas(), 0x4, add(_proofData, add(0x20, mul(0x20, _numberOfPublicInputs))), len, add(proof, 0x20), len
                )
            )
        }
    }

    function splitProofHonk(bytes memory _proofData, uint256 _numberOfPublicInputs)
        internal
        view
        returns (bytes32[] memory publicInputs, bytes memory proof)
    {
        publicInputs = new bytes32[](_numberOfPublicInputs);
        for (uint256 i = 0; i < _numberOfPublicInputs; i++) {
            // The proofs spit out by barretenberg have the public inputs at the beginning
            publicInputs[i] = readWordByIndex(_proofData, i + 3); // TODO(md): Plus 3 as circuit_size, number of pub, offset preceed
        }

        proof = new bytes(_proofData.length - (_numberOfPublicInputs * 0x20));
        uint256 len = proof.length;

        // Copy first 3 words from proofData to proof
        assembly {
            mstore(add(proof, 0x20), mload(add(_proofData, 0x20)))
            mstore(add(proof, 0x40), mload(add(_proofData, 0x40)))
            mstore(add(proof, 0x60), mload(add(_proofData, 0x60)))
        }

        // Copy the rest of the proof
        assembly {
            pop(
                staticcall(
                    gas(),
                    0x4,
                    add(
                        _proofData,
                        add(
                            0x20,
                            mul(
                                0x20,
                                add(_numberOfPublicInputs, 3) // Then skip public inputs & 3 words already added
                            )
                        )
                    ),
                    len,
                    add(
                        proof,
                        add(0x20, 0x60) // skip the first 3 words we added above
                    ),
                    len
                )
            )
        }
    }

    function printBytes(bytes memory _data, uint256 _offset) internal {
        uint256 length = _data.length - _offset;
        for (uint256 i = 0; i < length / 0x20; i++) {
            bytes32 val;
            assembly {
                val := mload(add(_offset, add(_data, mul(0x20, add(1, i)))))
            }
            emit log_named_bytes32(toHexString(bytes32(i * 0x20)), val);
        }
    }

    function printList(bytes32[] memory _data, uint256 _offset) internal {
        for (uint256 i = _offset; i < _data.length; i++) {
            emit log_named_bytes32(i.toString(), _data[i]);
        }
    }

    function printList(uint256[] memory _data, uint256 _offset) internal {
        for (uint256 i = _offset; i < _data.length; i++) {
            emit log_named_bytes32(i.toString(), bytes32(_data[i]));
        }
    }

    function readWordByIndex(bytes memory _data, uint256 index) internal pure returns (bytes32 result) {
        assembly {
            result := mload(add(_data, add(0x20, mul(0x20, index))))
        }
    }

    /**
     * Convert a bytes32 into an ASCII encoded hex string
     * @param input bytes32 variable
     * @return result hex-encoded string
     */
    function toHexString(bytes32 input) public pure returns (string memory result) {
        if (uint256(input) == 0x00) {
            assembly {
                result := mload(0x40)
                mstore(result, 0x40)
                mstore(add(result, 0x20), 0x3030303030303030303030303030303030303030303030303030303030303030)
                mstore(add(result, 0x40), 0x3030303030303030303030303030303030303030303030303030303030303030)
                mstore(0x40, add(result, 0x60))
            }
            return result;
        }
        assembly {
            result := mload(0x40)
            let table := add(result, 0x60)

            // Store lookup table that maps an integer from 0 to 99 into a 2-byte ASCII equivalent
            // Store lookup table that maps an integer from 0 to ff into a 2-byte ASCII equivalent
            mstore(add(table, 0x1e), 0x3030303130323033303430353036303730383039306130623063306430653066)
            mstore(add(table, 0x3e), 0x3130313131323133313431353136313731383139316131623163316431653166)
            mstore(add(table, 0x5e), 0x3230323132323233323432353236323732383239326132623263326432653266)
            mstore(add(table, 0x7e), 0x3330333133323333333433353336333733383339336133623363336433653366)
            mstore(add(table, 0x9e), 0x3430343134323433343434353436343734383439346134623463346434653466)
            mstore(add(table, 0xbe), 0x3530353135323533353435353536353735383539356135623563356435653566)
            mstore(add(table, 0xde), 0x3630363136323633363436353636363736383639366136623663366436653666)
            mstore(add(table, 0xfe), 0x3730373137323733373437353736373737383739376137623763376437653766)
            mstore(add(table, 0x11e), 0x3830383138323833383438353836383738383839386138623863386438653866)
            mstore(add(table, 0x13e), 0x3930393139323933393439353936393739383939396139623963396439653966)
            mstore(add(table, 0x15e), 0x6130613161326133613461356136613761386139616161626163616461656166)
            mstore(add(table, 0x17e), 0x6230623162326233623462356236623762386239626162626263626462656266)
            mstore(add(table, 0x19e), 0x6330633163326333633463356336633763386339636163626363636463656366)
            mstore(add(table, 0x1be), 0x6430643164326433643464356436643764386439646164626463646464656466)
            mstore(add(table, 0x1de), 0x6530653165326533653465356536653765386539656165626563656465656566)
            mstore(add(table, 0x1fe), 0x6630663166326633663466356636663766386639666166626663666466656666)
            /**
             * Convert `input` into ASCII.
             *
             * Slice 2 base-10  digits off of the input, use to index the ASCII lookup table.
             *
             * We start from the least significant digits, write results into mem backwards,
             * this prevents us from overwriting memory despite the fact that each mload
             * only contains 2 byteso f useful data.
             *
             */
            let base := input
            function slice(v, tableptr) {
                mstore(0x1e, mload(add(tableptr, shl(1, and(v, 0xff)))))
                mstore(0x1c, mload(add(tableptr, shl(1, and(shr(8, v), 0xff)))))
                mstore(0x1a, mload(add(tableptr, shl(1, and(shr(16, v), 0xff)))))
                mstore(0x18, mload(add(tableptr, shl(1, and(shr(24, v), 0xff)))))
                mstore(0x16, mload(add(tableptr, shl(1, and(shr(32, v), 0xff)))))
                mstore(0x14, mload(add(tableptr, shl(1, and(shr(40, v), 0xff)))))
                mstore(0x12, mload(add(tableptr, shl(1, and(shr(48, v), 0xff)))))
                mstore(0x10, mload(add(tableptr, shl(1, and(shr(56, v), 0xff)))))
                mstore(0x0e, mload(add(tableptr, shl(1, and(shr(64, v), 0xff)))))
                mstore(0x0c, mload(add(tableptr, shl(1, and(shr(72, v), 0xff)))))
                mstore(0x0a, mload(add(tableptr, shl(1, and(shr(80, v), 0xff)))))
                mstore(0x08, mload(add(tableptr, shl(1, and(shr(88, v), 0xff)))))
                mstore(0x06, mload(add(tableptr, shl(1, and(shr(96, v), 0xff)))))
                mstore(0x04, mload(add(tableptr, shl(1, and(shr(104, v), 0xff)))))
                mstore(0x02, mload(add(tableptr, shl(1, and(shr(112, v), 0xff)))))
                mstore(0x00, mload(add(tableptr, shl(1, and(shr(120, v), 0xff)))))
            }

            mstore(result, 0x40)
            slice(base, table)
            mstore(add(result, 0x40), mload(0x1e))
            base := shr(128, base)
            slice(base, table)
            mstore(add(result, 0x20), mload(0x1e))
            mstore(0x40, add(result, 0x60))
        }
    }
}
