#[macro_export]
macro_rules! PAIRINGSBN254_LIBRARY {
    () => { r#"
    /**
     * @title PairingsBn254 library used for the fr, g1 and g2 point types
     * @dev Used to manipulate fr, g1, g2 types, perform modular arithmetic on them and call
     * the precompiles add, scalar mul and pairing
     *
     * Notes on optimisations
     * 1) Perform addmod, mulmod etc. in assembly - removes the check that Solidity performs to confirm that
     * the supplied modulus is not 0. This is safe as the modulus's used (r_mod, q_mod) are hard coded
     * inside the contract and not supplied by the user
     */
    library PairingsBn254 {
        uint256 constant q_mod = 21888242871839275222246405745257275088696311157297823662689037894645226208583;
        uint256 constant r_mod = 21888242871839275222246405745257275088548364400416034343698204186575808495617;
    
        function new_fr(uint256 value) internal pure returns (Types.Fr memory out) {
            assembly {
                mstore(out, mod(value, r_mod))
            }
        }
    
        function copy(Types.Fr memory self)
            internal
            pure
            returns (Types.Fr memory n)
        {
            n.value = self.value;
        }
    
        function assign(Types.Fr memory self, Types.Fr memory other) internal pure {
            self.value = other.value;
        }
    
        function inverse(Types.Fr memory fr)
            internal
            view
            returns (Types.Fr memory)
        {
            assert(fr.value != 0);
            return pow(fr, r_mod - 2);
        }
    
        function add_assign(Types.Fr memory self, Types.Fr memory other)
            internal
            pure
        {
            assembly {
                mstore(self, addmod(mload(self), mload(other), r_mod))
            }
        }
    
        function add_fr(Types.Fr memory a, Types.Fr memory b)
            internal
            pure
            returns (Types.Fr memory out)
        {
            assembly {
                mstore(out, addmod(mload(a), mload(b), r_mod))
            }
        }
    
        // overloaded add_fr fn, to supply custom modulus
        function add_fr(
            Types.Fr memory a,
            Types.Fr memory b,
            uint256 modulus
        ) internal pure returns (Types.Fr memory out) {
            assembly {
                mstore(out, addmod(mload(a), mload(b), modulus))
            }
        }
    
        function sub_assign(Types.Fr memory self, Types.Fr memory other)
            internal
            pure
        {
            assembly {
                mstore(self, addmod(mload(self), sub(r_mod, mload(other)), r_mod))
            }
        }
    
        function sub_fr(Types.Fr memory a, Types.Fr memory b)
            internal
            pure
            returns (Types.Fr memory out)
        {
            assembly {
                mstore(out, addmod(mload(a), sub(r_mod, mload(b)), r_mod))
            }
        }
    
        function neg_assign(Types.Fr memory self) internal pure {
            assembly {
                mstore(self, mod(sub(r_mod, mload(self)), r_mod))
            }
        }
    
        function mul_assign(Types.Fr memory self, Types.Fr memory other)
            internal
            pure
        {
            assembly {
                mstore(self, mulmod(mload(self), mload(other), r_mod))
            }
        }
    
        function mul_fr(Types.Fr memory a, Types.Fr memory b)
            internal
            pure
            returns (Types.Fr memory out)
        {
            // uint256 mulValue;
            assembly {
                mstore(out, mulmod(mload(a), mload(b), r_mod))
            }
            // return Types.Fr(mulValue);
        }
    
        function sqr_fr(Types.Fr memory a)
            internal
            pure
            returns (Types.Fr memory out)
        {
            assembly {
                let aVal := mload(a)
                mstore(out, mulmod(aVal, aVal, r_mod))
            }
        }
    
        function pow_2(Types.Fr memory self) internal pure returns (Types.Fr memory) {
            uint256 input = self.value;
    
            assembly {
                input := mulmod(input, input, r_mod)
            }
            return Types.Fr(input);
        }
    
        function pow_3(Types.Fr memory self) internal pure returns (Types.Fr memory) {
            uint256 input = self.value;
    
            assembly {
                input := mulmod(input, input, r_mod)
                input := mulmod(input, mload(self), r_mod)
            }
            return Types.Fr(input);
        }
    
        function pow_4(Types.Fr memory self) internal pure returns (Types.Fr memory) {
            uint256 input = self.value;
    
            assembly {
                input := mulmod(input, input, r_mod)
                input := mulmod(input, input, r_mod)
            }
            return Types.Fr(input);
        }
    
        function get_msb(uint256 input) internal pure returns (uint256 bit_position) {
            assembly {
                input := or(input, shr(1, input))
                input := or(input, shr(2, input))
                input := or(input, shr(4, input))
                input := or(input, shr(8, input))
                input := or(input, shr(16, input))
                input := or(input, shr(32, input))
                input := or(input, shr(64, input))
                input := or(input, shr(128, input))
                input := shr(1, add(input, 1))
                let m := mload(0x40)
                mstore(m, 0x00)
                mstore(
                    add(m, 0x1f),
                    0x0000016d02d06e1303dad1e66f8e14310469dbdfd280e7b070948f3b15bb3200
                )
                mstore(
                    add(m, 0x3f),
                    0x05476ae3dc38e0fbd3c881fee89eb120712695d690c43caa1640bccb330c00ed
                )
                mstore(
                    add(m, 0x5f),
                    0x065248846b11e42fddae3900e1f9fc1ed4a8c9eb822dff1ce91a9fa1b26421a3
                )
                mstore(
                    add(m, 0x7f),
                    0x727a27b49600d7669144c5233d4faba517774174bd5ccc7c34c00d290058eeb6
                )
                mstore(
                    add(m, 0x9f),
                    0x075f539849f285006ccf12d9e58d3068de7faf933aba0046e237fac7fd9d1f25
                )
                mstore(
                    add(m, 0xbf),
                    0xd5c3a93fca0bec5183102ead00f81da7ea2c1b19a063a279b3006543224ea476
                )
                mstore(
                    add(m, 0xdf),
                    0x735b7bbf2857b55e97f100ced88c677e92b94536c69c24c23e0a500facf7a62b
                )
                mstore(
                    add(m, 0xff),
                    0x18627800424d755abe565df0cd8b7db8359bc1090ef62a61004c5955ef8ab79a
                )
                mstore(
                    add(m, 0x11f),
                    0x08f5604b548999f44a88f3878600000000000000000000000000000000000000
                )
                // let isolated_high_bit := and(input, sub(0, input))
                let index := mod(input, 269)
                bit_position := mload(add(m, index))
                bit_position := and(bit_position, 0xff)
            }
        }
    
        function pow_small(
            Types.Fr memory base,
            uint256 exp,
            uint256 mod
        ) internal pure returns (Types.Fr memory) {
            uint256 result = 1;
            uint256 input = base.value;
            for (uint256 count = 1; count <= exp; count *= 2) {
                if (exp & count != 0) {
                    result = mulmod(result, input, mod);
                }
                input = mulmod(input, input, mod);
            }
            return new_fr(result);
        }
    
        function pow(Types.Fr memory self, uint256 power)
            internal
            view
            returns (Types.Fr memory)
        {
            uint256[6] memory input = [32, 32, 32, self.value, power, r_mod];
            uint256[1] memory result;
            bool success;
            assembly {
                success := staticcall(gas(), 0x05, input, 0xc0, result, 0x20)
            }
            require(success);
            return Types.Fr({value: result[0]});
        }
    
        // Calculates the result of an expression of the form: (a + bc + d).
        // a, b, c, d are Fr elements
        function compute_bracket(
            Types.Fr memory a,
            Types.Fr memory b,
            Types.Fr memory c,
            Types.Fr memory d
        ) internal pure returns (Types.Fr memory) {
            uint256 aPlusD;
            assembly {
                aPlusD := addmod(mload(a), mload(d), r_mod)
            }
    
            uint256 bMulC;
            assembly {
                bMulC := mulmod(mload(b), mload(c), r_mod)
            }
    
            uint256 result;
            assembly {
                result := addmod(aPlusD, bMulC, r_mod)
            }
            return new_fr(result);
        }
    
        // Calculates the result of an expression of the form: (abcd)
        // a, b, c are Fr elements
        // d is a G1Point
        function compute_product_3(
            Types.Fr memory a,
            Types.Fr memory b,
            Types.Fr memory c
        ) internal pure returns (Types.Fr memory) {
            Types.Fr memory scalar_product = mul_fr(a, mul_fr(b, c));
            return scalar_product;
        }
    
        // calculates the result of an expression of the form: (abc)
        // a, b are Fr elements
        // c is a G1Point
        function compute_product_3_mixed(
            Types.Fr memory a,
            Types.Fr memory b,
            Types.G1Point memory c
        ) internal view returns (Types.G1Point memory) {
            Types.Fr memory scalar_product = mul_fr(a, b);
            Types.G1Point memory result = point_mul(c, scalar_product);
            return result;
        }
    
        function compute_elliptic_mul(
            Types.G1Point memory first_term,
            Types.G1Point memory second_term,
            Types.G1Point memory third_term,
            Types.G1Point memory fourth_term,
            Types.G1Point memory fifth_term
        ) internal view returns (Types.G1Point memory) {
            Types.G1Point memory accumulator = copy_g1(first_term);
            accumulator = point_add(accumulator, second_term);
            accumulator = point_add(accumulator, third_term);
            accumulator = point_add(accumulator, fourth_term);
            accumulator = point_add(accumulator, fifth_term);
            return accumulator;
        }
    
        function accumulate_six(
            Types.G1Point memory first_term,
            Types.G1Point memory second_term,
            Types.G1Point memory third_term,
            Types.G1Point memory fourth_term,
            Types.G1Point memory fifth_term,
            Types.G1Point memory sixth_term
        ) internal view returns (Types.G1Point memory) {
            Types.G1Point memory accumulator = copy_g1(first_term);
            accumulator = point_add(accumulator, second_term);
            accumulator = point_add(accumulator, third_term);
            accumulator = point_add(accumulator, fourth_term);
            accumulator = point_add(accumulator, fifth_term);
            accumulator = point_add(accumulator, sixth_term);
            return accumulator;
        }
    
        function P1() internal pure returns (Types.G1Point memory) {
            return Types.G1Point(1, 2);
        }
    
        function new_g1(uint256 x, uint256 y)
            internal
            pure
            returns (Types.G1Point memory)
        {
            uint256 xValue;
            uint256 yValue;
            assembly {
                xValue := mod(x, r_mod)
                yValue := mod(y, r_mod)
            }
            return Types.G1Point(xValue, yValue);
        }
    
        function new_g2(uint256[2] memory x, uint256[2] memory y)
            internal
            pure
            returns (Types.G2Point memory)
        {
            return Types.G2Point(x, y);
        }
    
        function copy_g1(Types.G1Point memory self)
            internal
            pure
            returns (Types.G1Point memory result)
        {
            result.X = self.X;
            result.Y = self.Y;
        }
    
        function P2() internal pure returns (Types.G2Point memory) {
            // for some reason ethereum expects to have c1*v + c0 form
    
            return
                Types.G2Point(
                    [
                        0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
                        0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed
                    ],
                    [
                        0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b,
                        0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa
                    ]
                );
        }
    
        function negate(Types.G1Point memory self) internal pure {
            // The prime q in the base field F_q for G1
            if (self.X == 0 && self.Y == 0) return;
            self.Y = q_mod - self.Y;
        }
    
        function point_add(Types.G1Point memory p1, Types.G1Point memory p2)
            internal
            view
            returns (Types.G1Point memory r)
        {
            point_add_into_dest(p1, p2, r);
            return r;
        }
    
        function point_add_assign(Types.G1Point memory p1, Types.G1Point memory p2)
            internal
            view
        {
            point_add_into_dest(p1, p2, p1);
        }
    
        function point_add_into_dest(
            Types.G1Point memory p1,
            Types.G1Point memory p2,
            Types.G1Point memory dest
        ) internal view {
            validateG1Point(p1);
            validateG1Point(p2);
            uint256[4] memory input;
            if (p2.X == 0 && p2.Y == 0) {
                // we add zero, nothing happens
                dest.X = p1.X;
                dest.Y = p1.Y;
                return;
            } else if (p1.X == 0 && p1.Y == 0) {
                // we add into zero, and we add non-zero point
                dest.X = p2.X;
                dest.Y = p2.Y;
                return;
            } else {
                input[0] = p1.X;
                input[1] = p1.Y;
                input[2] = p2.X;
                input[3] = p2.Y;
            }
            bool success = false;
            assembly {
                success := staticcall(gas(), 6, input, 0x80, dest, 0x40)
            }
            require(success);
        }
    
        function point_sub_assign(Types.G1Point memory p1, Types.G1Point memory p2)
            internal
            view
        {
            point_sub_into_dest(p1, p2, p1);
        }
    
        function point_sub(Types.G1Point memory p1, Types.G1Point memory p2)
            internal
            view
            returns (Types.G1Point memory r)
        {
            point_sub_into_dest(p1, p2, r);
            return r;
        }
    
        function point_sub_into_dest(
            Types.G1Point memory p1,
            Types.G1Point memory p2,
            Types.G1Point memory dest
        ) internal view {
            validateG1Point(p1);
            validateG1Point(p2);
            uint256[4] memory input;
            if (p2.X == 0 && p2.Y == 0) {
                // we subtracted zero, nothing happens
                dest.X = p1.X;
                dest.Y = p1.Y;
                return;
            } else if (p1.X == 0 && p1.Y == 0) {
                // we subtract from zero, and we subtract non-zero point
                dest.X = p2.X;
                dest.Y = q_mod - p2.Y;
                return;
            } else {
                input[0] = p1.X;
                input[1] = p1.Y;
                input[2] = p2.X;
                input[3] = q_mod - p2.Y;
            }
            bool success = false;
            assembly {
                success := staticcall(gas(), 6, input, 0x80, dest, 0x40)
            }
            require(success);
        }
    
        function point_mul(Types.G1Point memory p, Types.Fr memory s)
            internal
            view
            returns (Types.G1Point memory r)
        {
            point_mul_into_dest(p, s, r);
            return r;
        }
    
        function point_mul_assign(Types.G1Point memory p, Types.Fr memory s)
            internal
            view
        {
            point_mul_into_dest(p, s, p);
        }
    
        function point_mul_into_dest(
            Types.G1Point memory p,
            Types.Fr memory s,
            Types.G1Point memory dest
        ) internal view {
            validateG1Point(p);
            validateScalar(s);
            uint256[3] memory input;
            input[0] = p.X;
            input[1] = p.Y;
            input[2] = s.value;
            bool success;
    
            assembly {
                success := staticcall(gas(), 7, input, 0x60, dest, 0x40)
            }
            require(success);
        }
    
        function pairing(Types.G1Point[] memory p1, Types.G2Point[] memory p2)
            internal
            view
            returns (bool)
        {
            require(p1.length == p2.length);
    
            for (uint256 i = 0; i < p1.length; i += 1) {
                validateG1Point(p1[i]);
            }
            uint256 elements = p1.length;
            uint256 inputSize = elements * 6;
            uint256[] memory input = new uint256[](inputSize);
    
            for (uint256 i = 0; i < elements; i++) {
                input[i * 6 + 0] = p1[i].X;
                input[i * 6 + 1] = p1[i].Y;
                input[i * 6 + 2] = p2[i].X[0];
                input[i * 6 + 3] = p2[i].X[1];
                input[i * 6 + 4] = p2[i].Y[0];
                input[i * 6 + 5] = p2[i].Y[1];
            }
            uint256[1] memory out;
            bool success;
            assembly {
                success := staticcall(
                    gas(),
                    8,
                    add(input, 0x20),
                    mul(inputSize, 0x20),
                    out,
                    0x20
                )
            }
            require(success);
            if (out[0] != 0) {
                return true;
            } else return false;
        }
    
        /// Convenience method for a pairing check for two pairs.
        function pairingProd2(
            Types.G1Point memory a1,
            Types.G2Point memory a2,
            Types.G1Point memory b1,
            Types.G2Point memory b2
        ) internal view returns (bool) {
            Types.G1Point[] memory p1 = new Types.G1Point[](2);
            Types.G2Point[] memory p2 = new Types.G2Point[](2);
            p1[0] = a1;
            p1[1] = b1;
            p2[0] = a2;
            p2[1] = b2;
            return pairing(p1, p2);
        }
    
        function validateG1Point(Types.G1Point memory point) internal pure {
            require(point.X < q_mod, "PairingsBn254: x > q_mod");
            require(point.Y < q_mod, "Pairng: y > q_mod");
            require(point.X != uint256(0), "PairingsBn254: x = 0");
            require(point.Y != uint256(0), "PairingsBn254: y = 0");
    
            // validating on curve: check y^2 = x^3 + 3 mod q_mod holds
            Types.Fr memory lhs = pow_small(new_fr(point.Y), 2, q_mod);
            Types.Fr memory rhs = add_fr(
                pow_small(new_fr(point.X), 3, q_mod),
                new_fr(3),
                q_mod
            );
            require(lhs.value == rhs.value, "PairingsBn254: not on curve");
        }
    
        function validateScalar(Types.Fr memory scalar) internal pure {
            require(scalar.value < r_mod, "PairingsBn254: scalar invalid");
        }
    }
    
    
    "# };
}