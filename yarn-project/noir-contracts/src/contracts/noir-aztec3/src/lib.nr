mod abi;
mod map;
mod note;
mod set;
mod vec; //This can/should be moved out into another library

global Nonce = 1;
global NoteHash = 2;
global NoteStorageSlot = 3;
global MappingStorageSlot = 4;
global Nullifier = 5;

global TREE_HEIGHT: comptime Field = 5;
