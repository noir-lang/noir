import {Honk, P, Q} from "./HonkTypes.sol";
import {Fr, FrLib} from "./Fr.sol";

import "forge-std/console.sol";
import "forge-std/console2.sol";

function bytes32ToString(bytes32 value) pure returns (string memory) {
    bytes memory alphabet = "0123456789abcdef";

    bytes memory str = new bytes(66);
    str[0] = "0";
    str[1] = "x";
    for (uint256 i = 0; i < 32; i++) {
        str[2 + i * 2] = alphabet[uint8(value[i] >> 4)];
        str[3 + i * 2] = alphabet[uint8(value[i] & 0x0f)];
    }
    return string(str);
}

function logG1(string memory name, Honk.G1ProofPoint memory point) pure {
    // TODO: convert both to hex before printing to line up with cpp
    string memory x_0 = bytes32ToString(bytes32(point.x_0));
    string memory x_1 = bytes32ToString(bytes32(point.x_1));
    string memory y_0 = bytes32ToString(bytes32(point.y_0));
    string memory y_1 = bytes32ToString(bytes32(point.y_1));

    string memory message = string(abi.encodePacked(name, " x: ", x_0, x_1, " y: ", y_0, y_1));
    console2.log(message);
}

function logG(string memory name, Honk.G1Point memory point) pure {
    // TODO: convert both to hex before printing to line up with cpp
    string memory x = bytes32ToString(bytes32(point.x));
    string memory y = bytes32ToString(bytes32(point.y));

    string memory message = string(abi.encodePacked(name, " x: ", x, " y: ", y));
    console2.log(message);
}

function logG(string memory name, uint256 i, Honk.G1Point memory point) pure {
    // TODO: convert both to hex before printing to line up with cpp
    string memory x = bytes32ToString(bytes32(point.x));
    string memory y = bytes32ToString(bytes32(point.y));

    string memory message = string(abi.encodePacked(name, " ", i, " x: ", x, " y: ", y));
    console2.log(message);
}

function logUint(string memory name, uint256 value) pure {
    string memory as_hex = bytes32ToString(bytes32(value));
    console2.log(name, as_hex);
}

function logFr(string memory name, Fr value) pure {
    string memory as_hex = bytes32ToString(bytes32(Fr.unwrap(value)));
    console2.log(name, as_hex);
}

function logFr(string memory name, uint256 i, Fr value) pure {
    string memory as_hex = bytes32ToString(bytes32(Fr.unwrap(value)));
    console2.log(name, i, as_hex);
}

// EC Point utilities

function convertProofPoint(Honk.G1ProofPoint memory input) pure returns (Honk.G1Point memory) {
    return Honk.G1Point({x: input.x_0 | (input.x_1 << 136), y: input.y_0 | (input.y_1 << 136)});
}

function ecMul(Honk.G1Point memory point, Fr scalar) view returns (Honk.G1Point memory) {
    bytes memory input = abi.encodePacked(point.x, point.y, Fr.unwrap(scalar));
    (bool success, bytes memory result) = address(0x07).staticcall(input);
    require(success, "ecMul failed");

    (uint256 x, uint256 y) = abi.decode(result, (uint256, uint256));
    return Honk.G1Point({x: x, y: y});
}

function ecAdd(Honk.G1Point memory point0, Honk.G1Point memory point1) view returns (Honk.G1Point memory) {
    bytes memory input = abi.encodePacked(point0.x, point0.y, point1.x, point1.y);
    (bool success, bytes memory result) = address(0x06).staticcall(input);
    require(success, "ecAdd failed");

    (uint256 x, uint256 y) = abi.decode(result, (uint256, uint256));
    return Honk.G1Point({x: x, y: y});
}

function ecSub(Honk.G1Point memory point0, Honk.G1Point memory point1) view returns (Honk.G1Point memory) {
    // We negate the second point
    uint256 negativePoint1Y = (Q - point1.y) % Q;
    bytes memory input = abi.encodePacked(point0.x, point0.y, point1.x, negativePoint1Y);
    (bool success, bytes memory result) = address(0x06).staticcall(input);
    require(success, "ecAdd failed");

    (uint256 x, uint256 y) = abi.decode(result, (uint256, uint256));
    return Honk.G1Point({x: x, y: y});
}

function negateInplace(Honk.G1Point memory point) pure returns (Honk.G1Point memory) {
    point.y = (Q - point.y) % Q;
    return point;
}
