mod abi;
mod context;
mod messaging;
mod notes;
mod state_vars;
mod types;
mod utils;
mod private_call_stack_item;
mod public_call_stack_item;
mod public_call;
mod debug_log;

global Nonce = 1;
global NoteHash = 2;
global NoteStorageSlot = 3;
global MappingStorageSlot = 4;
global Nullifier = 5;
global L1ToL2MessageSecret = 29;

global PRIVATE_DATA_TREE_HEIGHT: comptime Field = 8;
global L1_TO_L2_MESSAGES_TREE_HEIGHT: comptime Field = 8;
