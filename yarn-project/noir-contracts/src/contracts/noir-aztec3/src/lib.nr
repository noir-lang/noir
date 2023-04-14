mod abi;
mod notes;
mod state_vars;
mod types;

global Nonce = 1;
global NoteHash = 2;
global NoteStorageSlot = 3;
global MappingStorageSlot = 4;
global Nullifier = 5;

global PRIVATE_DATA_TREE_HEIGHT: comptime Field = 8;
