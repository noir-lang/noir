#[macro_export]
macro_rules! TRANSCRIPT_LIBRARY {
    () => { r#"
    /**
     * @title Challenge transcript library
     * @dev Used to collect the data necessary to calculate the various challenges: beta, gamma, alpha, zeta, nu[7], u
     */
    library TranscriptLibrary {
        uint256 constant r_mod = 21888242871839275222246405745257275088548364400416034343698204186575808495617;
    
        struct Transcript {
            bytes32 current_challenge;
            bytes data;
            uint32 challenge_counter;
            bytes32 debug_data;
        }
        event ChallengeDebug(bytes32 data);
    
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
            bytes memory formatted_circuit_size = format_4_byte_variable(uint32(circuit_size));
            bytes memory formatted_num_public_inputs = format_4_byte_variable(uint32(num_public_inputs));
    
            transcript.current_challenge = keccak256(abi.encodePacked(formatted_circuit_size, formatted_num_public_inputs));
            transcript.debug_data = transcript.current_challenge;
            transcript.data = abi.encodePacked(transcript.current_challenge);
            transcript.challenge_counter = 0;
        }
    
        function format_4_byte_variable(uint32 input) internal pure returns (bytes memory) {
            // uint8 byte0 = uint8(input & 0xff);
            // uint8 byte1 = uint8((input >> 8) & 0xff);
            // uint8 byte2 = uint8((input >> 16) & 0xff);
            // uint8 byte3 = uint8((input >> 24) & 0xff);
            // // TODO SWAP
            uint8 byte0 = uint8((input >> 24) & 0xff);
            uint8 byte1 = uint8((input >> 16) & 0xff);
            uint8 byte2 = uint8((input >> 8) & 0xff);
            uint8 byte3 = uint8((input) & 0xff);
            return abi.encodePacked(byte0, byte1, byte2, byte3);
        }
    
        /**
         * Add a uint256 into the transcript
         */
        function update_with_u256(Transcript memory self, uint256 value) internal pure {
            self.data = abi.encodePacked(self.data, value);
        }
    
        /**
         * Add a field element into the transcript
         */
        function update_with_fr(Transcript memory self, Types.Fr memory value) internal pure {
            update_with_u256(self, value.value);
        }
    
        /**
         * Add a g1 point into the transcript
         */
        function update_with_g1(Transcript memory self, Types.G1Point memory p) internal pure {
            // in the C++ prover, the y coord is appended first before the x
            update_with_u256(self, p.Y);
            update_with_u256(self, p.X);
        }
    
        /**
         * Append byte
         */
        function append_byte(Transcript memory self, uint8 value) internal pure {
            self.data = abi.encodePacked(self.data, value);
        }
    
        /**
         * Draw a challenge
         */
        function get_challenge(Transcript memory self) internal pure returns (Types.Fr memory) {
            bytes32 challenge = keccak256(self.data);
            self.current_challenge = challenge;
            self.data = abi.encodePacked(challenge);
            return Types.Fr({value: uint256(challenge) % r_mod});
        }
    }
    
    
    "# };
}