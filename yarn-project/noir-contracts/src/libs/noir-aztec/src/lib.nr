mod abi;
mod context;
mod generators;
mod log;
mod messaging;
mod note;
mod state_vars;
mod types;
mod utils;
mod private_call_stack_item;
mod public_call_stack_item;
mod oracle;
mod entrypoint;

global PRIVATE_DATA_TREE_HEIGHT: comptime Field = 32;
global L1_TO_L2_MSG_TREE_HEIGHT: comptime Field = 16;
