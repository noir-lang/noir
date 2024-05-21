use dep::protocol_types::constants::{GENERATOR_INDEX__NSK_M, GENERATOR_INDEX__IVSK_M, GENERATOR_INDEX__OVSK_M, GENERATOR_INDEX__TSK_M};

// Note: In fetch_key_from_registry we expect that the shared mutable slot is index * 2 + 1 for the x coordinate and
// index * 2 + 2 for the y coordinate. For example, the npk_m x coordinates will be stored in a map at storage slot
// 0 * 2 + 1 = 1, and the npk_m y coordinates at 2 * 2 + 2 = 6. If this changes the function will need to be
// refactored.
global NULLIFIER_INDEX = 0;
global INCOMING_INDEX = 1;
global OUTGOING_INDEX = 2;
global TAGGING_INDEX = 3;

global NUM_KEY_TYPES = 4;

global sk_generators = [
    GENERATOR_INDEX__NSK_M,
    GENERATOR_INDEX__IVSK_M,
    GENERATOR_INDEX__OVSK_M,
    GENERATOR_INDEX__TSK_M
];
