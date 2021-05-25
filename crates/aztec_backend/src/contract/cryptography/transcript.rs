#[macro_export]
macro_rules! TRANSCRIPT_LIBRARY {
    () => { r#"

/**
 * @title Challenge transcript library
 * @dev Used to collect the data necessary to calculate the various challenges: beta, gamma, alpha, zeta, nu[7], u
 */
library TranscriptLibrary {
    // When creating `transcript.data` we pre-allocate all the memory required to store the entire transcript, minus public inputs
    uint256 constant NUM_TRANSCRIPT_BYTES = 1248;

    struct Transcript {
        bytes data;
        bytes32 current_challenge;
        uint32 challenge_counter;
        uint256 num_public_inputs;
    }

    /**
     * Instantiate a transcript and calculate the initial challenge, from which other challenges are derived.
     *
     * Resembles the preamble round in the Plonk prover
     */
    function new_transcript(uint256 circuit_size, uint256 num_public_inputs)
        internal
        pure
        returns (Transcript memory transcript)
    {
        transcript.current_challenge = compute_initial_challenge(circuit_size, num_public_inputs);
        transcript.challenge_counter = 0;

        // manually format the transcript.data bytes array
        // This is because we want to reserve memory that is greatly in excess of the array's initial size
        bytes memory transcript_data_pointer;
        bytes32 transcript_data = transcript.current_challenge;
        uint256 total_transcript_bytes = NUM_TRANSCRIPT_BYTES;
        assembly {
            transcript_data_pointer := mload(0x40)
            mstore(0x40, add(transcript_data_pointer, total_transcript_bytes))
            // update length of transcript.data
            mstore(transcript_data_pointer, 0x20)
            // insert current challenge
            mstore(add(transcript_data_pointer, 0x20), transcript_data)
        }
        transcript.data = transcript_data_pointer;
        transcript.num_public_inputs = num_public_inputs;
    }

    
    /**
     * Compute keccak256 hash of 2 4-byte variables (circuit_size, num_public_inputs)
     */
    function compute_initial_challenge(uint256 circuit_size, uint256 num_public_inputs) internal pure returns (bytes32 challenge) {
        assembly {
            let mPtr := mload(0x40)
            mstore8(add(mPtr, 0x20), shr(24, circuit_size))
            mstore8(add(mPtr, 0x21), shr(16, circuit_size))
            mstore8(add(mPtr, 0x22), shr(8, circuit_size))
            mstore8(add(mPtr, 0x23), circuit_size)           
            mstore8(add(mPtr, 0x24), shr(24, num_public_inputs))
            mstore8(add(mPtr, 0x25), shr(16, num_public_inputs))
            mstore8(add(mPtr, 0x26), shr(8, num_public_inputs))
            mstore8(add(mPtr, 0x27), num_public_inputs)
            challenge := keccak256(add(mPtr, 0x20), 0x08)
        }
    }

    /**
     * Add a uint256 into the transcript
     */
    function update_with_u256(Transcript memory self, uint256 value) internal pure {
        bytes memory data_ptr = self.data;
        assembly {
            // update length of transcript data
            let array_length := mload(data_ptr)
            mstore(data_ptr, add(0x20, array_length))
            // insert new 32-byte value at the end of the array
            mstore(add(data_ptr, add(array_length, 0x20)), value)
        }
    }

    /**
     * Add a g1 point into the transcript
     */
    function update_with_g1(Transcript memory self, Types.G1Point memory p) internal pure {
        // in the C++ prover, the y coord is appended first before the x
        bytes memory data_ptr = self.data;
        assembly {
            // update length of transcript data
            let array_length := mload(data_ptr)
            mstore(data_ptr, add(0x40, array_length))
            // insert new 64-byte value at the end of the array
            mstore(add(data_ptr, add(array_length, 0x20)), mload(add(p, 0x20)))
            mstore(add(data_ptr, add(array_length, 0x40)), mload(p))
        }
    }


    /**
     * Add a g1 point into the transcript
     */
    function update_with_four_g1_elements(
        Transcript memory self,
        Types.G1Point memory p1,
        Types.G1Point memory p2,
        Types.G1Point memory p3,
        Types.G1Point memory p4
    ) internal pure {
        // in the C++ prover, the y coord is appended first before the x
        bytes memory data_ptr = self.data;
        assembly {
            // update length of transcript data
            let array_length := mload(data_ptr)
            mstore(data_ptr, add(0x100, array_length))
            data_ptr := add(data_ptr, array_length)
            // insert new 64-byte value at the end of the array
            mstore(add(data_ptr, 0x20), mload(add(p1, 0x20)))
            mstore(add(data_ptr, 0x40), mload(p1))
            mstore(add(data_ptr, 0x60), mload(add(p2, 0x20)))
            mstore(add(data_ptr, 0x80), mload(p2))
            mstore(add(data_ptr, 0xa0), mload(add(p3, 0x20)))
            mstore(add(data_ptr, 0xc0), mload(p3))
            mstore(add(data_ptr, 0xe0), mload(add(p4, 0x20)))
            mstore(add(data_ptr, 0x100), mload(p4))
        }
    }

    /**
     * Append byte
     */
    function append_byte(Transcript memory self, uint8 value) internal pure {
        bytes memory data_ptr = self.data;
        uint256 array_length = 0;
        assembly {
            // update length of transcript data
            array_length := mload(data_ptr)
            mstore(data_ptr, add(0x01, array_length))
            // insert new 1-byte value at the end of the array
            mstore8(add(data_ptr, add(array_length, 0x20)), value)
        }
    }

    /**
     * Reset challenge array to equal a single bytes32 value
     */
    function reset_to_bytes32(Transcript memory self, bytes32 value) internal pure {
        bytes memory data_ptr = self.data;
        {
            assembly {
                mstore(data_ptr, 0x20)
                mstore(add(data_ptr, 0x20), value)
            }
        }
    }

    /**
     * Draw a challenge
     */
    function get_challenge(Transcript memory self) internal pure returns (uint256) {
        bytes32 challenge;
        bytes memory data_ptr = self.data;
        assembly {
            let length := mload(data_ptr)
            challenge := keccak256(add(data_ptr, 0x20), length)
        }
        self.current_challenge = challenge;

        // reset self.data by setting length to 0x20 and update first element
        {
            assembly {
                mstore(data_ptr, 0x20)
                mstore(add(data_ptr, 0x20), challenge)
            }
        }
        uint256 p = Bn254Crypto.r_mod;
        assembly {
            challenge := mod(challenge, p)
        }
        return (uint256)(challenge);
    }

    /**
     * We treat the beta challenge as a special case, because it includes the public inputs.
     * The number of public inputs can be extremely large for rollups and we want to minimize mem consumption.
     * => we directly allocate memory to hash the public inputs, in order to prevent the global memory pointer from increasing
     */
    function get_beta_gamma_challenges(
        Transcript memory self,
        Types.ChallengeTranscript memory challenges,
        uint256 num_public_inputs
    ) internal pure  {
        bytes32 challenge;
        bytes32 old_challenge = self.current_challenge;
        uint256 p = Bn254Crypto.r_mod;
        uint256 reduced_challenge;
        assembly {
            let m_ptr := mload(0x40)

            // N.B. If the calldata ABI changes this code will need to change!
            // We can copy all of the public inputs, followed by the wire commitments, into memory
            // using calldatacopy
            mstore(m_ptr, old_challenge)
            m_ptr := add(m_ptr, 0x20)
            let inputs_start := add(calldataload(0x04), 0x24)
            // num_calldata_bytes = public input size + 256 bytes for the 4 wire commitments
            let num_calldata_bytes := add(0x100, mul(num_public_inputs, 0x20))
            calldatacopy(m_ptr, inputs_start, num_calldata_bytes)

            let start := mload(0x40)
            let length := add(num_calldata_bytes, 0x20)

            challenge := keccak256(start, length)
            reduced_challenge := mod(challenge, p)
        }
        challenges.beta = reduced_challenge;

        // get gamma challenge by appending 1 to the beta challenge and hash
        assembly {
            mstore(0x00, challenge)
            mstore8(0x20, 0x01)
            challenge := keccak256(0, 0x21)
            reduced_challenge := mod(challenge, p)
        }
        challenges.gamma = reduced_challenge;

        bytes memory data_ptr = self.data;
        self.current_challenge = challenge;

        // reset self.data by setting length to 0x20 and update first element
        {
            assembly {
                mstore(data_ptr, 0x20)
                mstore(add(data_ptr, 0x20), challenge)
            }
        }
    }


    /**
     * We compute our initial nu challenge by hashing the following proof elements (with the current challenge):
     *
     * w1, w2, w3, w4, sigma1, sigma2, sigma3, q_arith, q_ecc, q_c, linearization_poly, grand_product_at_z_omega,
     * w1_omega, w2_omega, w3_omega, w4_omega
     *
     * These values are placed linearly in the proofData, we can extract them with a calldatacopy call
     *
     */
    function get_nu_challenges(Transcript memory self, uint256 quotient_poly_eval, Types.ChallengeTranscript memory challenges) internal pure
    {
        // get a calldata pointer that points to the start of the data we want to copy
        uint256 calldata_ptr;
        assembly {
            calldata_ptr := add(calldataload(0x04), 0x24)
        }
        {
            uint256 num_public_inputs = self.num_public_inputs;
            assembly {
                calldata_ptr := add(calldata_ptr, mul(num_public_inputs, 0x20))
            }
        }
        // There are NINE G1 group elements added into the transcript in the `beta` round, that we need to skip over
        assembly {
            calldata_ptr := add(calldata_ptr, 0x240) // 9 * 0x40 = 0x240
        }

        uint256 p = Bn254Crypto.r_mod;
        bytes memory data_ptr = self.data;
        uint256 base_v_challenge;
        uint256 updated_v;

        // We want to copy SIXTEEN field elements from calldata into memory to hash
        // But we start by adding the quotient poly evaluation to the hash transcript
        assembly {
            mstore(add(data_ptr, 0x40), quotient_poly_eval)
            calldatacopy(add(data_ptr, 0x60), calldata_ptr, 0x200) // 16 * 0x20 = 0x200
            base_v_challenge := keccak256(add(data_ptr, 0x20), 0x240) // hash length = 0x240, we include the previous challenge in the hash
            updated_v := mod(base_v_challenge, p)
        }

        // assign the first challenge value
        challenges.v0 = updated_v;

        // for subsequent challenges we iterate 10 times.
        // At each iteration i \in [1, 10] we compute challenges.vi = keccak256(base_v_challenge, byte(i))
        assembly {
            mstore(0x00, base_v_challenge)
            mstore8(0x20, 0x01)
            updated_v := mod(keccak256(0x00, 0x21), p)
        }
        challenges.v1 = updated_v;
        assembly {
            mstore8(0x20, 0x02)
            updated_v := mod(keccak256(0x00, 0x21), p)
        }
        challenges.v2 = updated_v;
        assembly {
            mstore8(0x20, 0x03)
            updated_v := mod(keccak256(0x00, 0x21), p)
        }
        challenges.v3 = updated_v;
        assembly {
            mstore8(0x20, 0x04)
            updated_v := mod(keccak256(0x00, 0x21), p)
        }
        challenges.v4 = updated_v;
        assembly {
            mstore8(0x20, 0x05)
            updated_v := mod(keccak256(0x00, 0x21), p)
        }
        challenges.v5 = updated_v;
        assembly {
            mstore8(0x20, 0x06)
            updated_v := mod(keccak256(0x00, 0x21), p)
        }
        challenges.v6 = updated_v;
        assembly {
            mstore8(0x20, 0x07)
            updated_v := mod(keccak256(0x00, 0x21), p)
        }
        challenges.v7 = updated_v;
        assembly {
            mstore8(0x20, 0x08)
            updated_v := mod(keccak256(0x00, 0x21), p)
        }
        challenges.v8 = updated_v;
        assembly {
            mstore8(0x20, 0x09)
            updated_v := mod(keccak256(0x00, 0x21), p)
        }
        challenges.v9 = updated_v;

        // update the current challenge when computing the final nu challenge
        bytes32 challenge;
        assembly {
            mstore8(0x20, 0x0a)
            challenge := keccak256(0x00, 0x21)

            mstore(data_ptr, 0x20)
            mstore(add(data_ptr, 0x20), challenge)
            updated_v := mod(challenge, p)
        }
        challenges.v10 = updated_v;

        self.current_challenge = challenge;
    }
}

    
    
    "# };
}
