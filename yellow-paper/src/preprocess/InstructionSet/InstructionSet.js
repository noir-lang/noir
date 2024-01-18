const {instructionSize} = require('./InstructionSize');

const TOPICS_IN_TABLE = [
    "Name", "Summary", "Bit-size", "Expression",
];
const TOPICS_IN_SECTIONS = [
    "Name", "Summary", "Category", "Flags", "Args", "Expression", "Details", "Tag checks", "Tag updates", "Bit-size",
];

const IN_TAG_DESCRIPTION = "The [tag/size](./state-model#tags-and-tagged-memory) to check inputs against and tag the destination with.";
const DST_TAG_DESCRIPTION = "The [tag/size](./state-model#tags-and-tagged-memory) to tag the destination with but not to check inputs against.";
const INDIRECT_FLAG_DESCRIPTION = "Toggles whether each memory-offset argument is an indirect offset. 0th bit corresponds to 0th offset arg, etc. Indirect offsets result in memory accesses like `M[M[offset]]` instead of the more standard `M[offset]`.";

const INSTRUCTION_SET_RAW = [
    {
        "id": "add",
        "Name": "`ADD`",
        "Category": "Compute - Arithmetic",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
            {"name": "inTag", "description": IN_TAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "aOffset", "description": "memory offset of the operation's left input"},
            {"name": "bOffset", "description": "memory offset of the operation's right input"},
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = M[aOffset] + M[bOffset] mod 2^k`",
        "Summary": "Addition (a + b)",
        "Details": "",
        "Tag checks": "`T[aOffset] == T[bOffset] == inTag`",
        "Tag updates": "`T[dstOffset] = inTag`",
    },
    {
        "id": "sub",
        "Name": "`SUB`",
        "Category": "Compute - Arithmetic",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
            {"name": "inTag", "description": IN_TAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "aOffset", "description": "memory offset of the operation's left input"},
            {"name": "bOffset", "description": "memory offset of the operation's right input"},
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = M[aOffset] - M[bOffset] mod 2^k`",
        "Summary": "Subtraction (a - b)",
        "Details": "",
        "Tag checks": "`T[aOffset] == T[bOffset] == inTag`",
        "Tag updates": "`T[dstOffset] = inTag`",
    },
    {
        "id": "mul",
        "Name": "`MUL`",
        "Category": "Compute - Arithmetic",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
            {"name": "inTag", "description": IN_TAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "aOffset", "description": "memory offset of the operation's left input"},
            {"name": "bOffset", "description": "memory offset of the operation's right input"},
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = M[aOffset] * M[bOffset] mod 2^k`",
        "Summary": "Multiplication (a * b)",
        "Details": "",
        "Tag checks": "`T[aOffset] == T[bOffset] == inTag`",
        "Tag updates": "`T[dstOffset] = inTag`",
    },
    {
        "id": "div",
        "Name": "`DIV`",
        "Category": "Compute - Arithmetic",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
            {"name": "inTag", "description": IN_TAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "aOffset", "description": "memory offset of the operation's left input"},
            {"name": "bOffset", "description": "memory offset of the operation's right input"},
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = M[aOffset] / M[bOffset]`",
        "Summary": "Unsigned division (a / b)",
        "Details": "",
        "Tag checks": "`T[aOffset] == T[bOffset] == inTag`",
        "Tag updates": "`T[dstOffset] = inTag`",
    },
    {
        "id": "eq",
        "Name": "`EQ`",
        "Category": "Compute - Comparators",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
            {"name": "inTag", "description": IN_TAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "aOffset", "description": "memory offset of the operation's left input"},
            {"name": "bOffset", "description": "memory offset of the operation's right input"},
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result", "type": "u8"},
        ],
        "Expression": "`M[dstOffset] = M[aOffset] == M[bOffset] ? 1 : 0`",
        "Summary": "Equality check (a == b)",
        "Details": "",
        "Tag checks": "`T[aOffset] == T[bOffset] == inTag`",
        "Tag updates": "`T[dstOffset] = inTag`",
    },
    {
        "id": "lt",
        "Name": "`LT`",
        "Category": "Compute - Comparators",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
            {"name": "inTag", "description": IN_TAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "aOffset", "description": "memory offset of the operation's left input"},
            {"name": "bOffset", "description": "memory offset of the operation's right input"},
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result", "type": "u8"},
        ],
        "Expression": "`M[dstOffset] = M[aOffset] < M[bOffset] ? 1 : 0`",
        "Summary": "Less-than check (a < b)",
        "Details": "",
        "Tag checks": "`T[aOffset] == T[bOffset] == inTag`",
        "Tag updates": "`T[dstOffset] = inTag`",
    },
    {
        "id": "lte",
        "Name": "`LTE`",
        "Category": "Compute - Comparators",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
            {"name": "inTag", "description": IN_TAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "aOffset", "description": "memory offset of the operation's left input"},
            {"name": "bOffset", "description": "memory offset of the operation's right input"},
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result", "type": "u8"},
        ],
        "Expression": "`M[dstOffset] = M[aOffset] <= M[bOffset] ? 1 : 0`",
        "Summary": "Less-than-or-equals check (a <= b)",
        "Details": "",
        "Tag checks": "`T[aOffset] == T[bOffset] == inTag`",
        "Tag updates": "`T[dstOffset] = inTag`",
    },
    {
        "id": "and",
        "Name": "`AND`",
        "Category": "Compute - Bitwise",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
            {"name": "inTag", "description": IN_TAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "aOffset", "description": "memory offset of the operation's left input"},
            {"name": "bOffset", "description": "memory offset of the operation's right input"},
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = M[aOffset] AND M[bOffset]`",
        "Summary": "Bitwise AND (a & b)",
        "Details": "",
        "Tag checks": "`T[aOffset] == T[bOffset] == inTag`",
        "Tag updates": "`T[dstOffset] = inTag`",
    },
    {
        "id": "or",
        "Name": "`OR`",
        "Category": "Compute - Bitwise",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
            {"name": "inTag", "description": IN_TAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "aOffset", "description": "memory offset of the operation's left input"},
            {"name": "bOffset", "description": "memory offset of the operation's right input"},
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = M[aOffset] OR M[bOffset]`",
        "Summary": "Bitwise OR (a | b)",
        "Details": "",
        "Tag checks": "`T[aOffset] == T[bOffset] == inTag`",
        "Tag updates": "`T[dstOffset] = inTag`",
    },
    {
        "id": "xor",
        "Name": "`XOR`",
        "Category": "Compute - Bitwise",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
            {"name": "inTag", "description": IN_TAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "aOffset", "description": "memory offset of the operation's left input"},
            {"name": "bOffset", "description": "memory offset of the operation's right input"},
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = M[aOffset] XOR M[bOffset]`",
        "Summary": "Bitwise XOR (a ^ b)",
        "Details": "",
        "Tag checks": "`T[aOffset] == T[bOffset] == inTag`",
        "Tag updates": "`T[dstOffset] = inTag`",
    },
    {
        "id": "not",
        "Name": "`NOT`",
        "Category": "Compute - Bitwise",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
            {"name": "inTag", "description": IN_TAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "aOffset", "description": "memory offset of the operation's input"},
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = NOT M[aOffset]`",
        "Summary": "Bitwise NOT (inversion)",
        "Details": "",
        "Tag checks": "`T[aOffset] == inTag`",
        "Tag updates": "`T[dstOffset] = inTag`",
    },
    {
        "id": "shl",
        "Name": "`SHL`",
        "Category": "Compute - Bitwise",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
            {"name": "inTag", "description": IN_TAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "aOffset", "description": "memory offset of the operation's left input"},
            {"name": "bOffset", "description": "memory offset of the operation's right input"},
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = M[aOffset] << M[bOffset]`",
        "Summary": "Bitwise leftward shift (a << b)",
        "Details": "",
        "Tag checks": "`T[aOffset] == T[bOffset] == inTag`",
        "Tag updates": "`T[dstOffset] = inTag`",
    },
    {
        "id": "shr",
        "Name": "`SHR`",
        "Category": "Compute - Bitwise",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
            {"name": "inTag", "description": IN_TAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "aOffset", "description": "memory offset of the operation's left input"},
            {"name": "bOffset", "description": "memory offset of the operation's right input"},
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = M[aOffset] >> M[bOffset]`",
        "Summary": "Bitwise rightward shift (a >> b)",
        "Details": "",
        "Tag checks": "`T[aOffset] == T[bOffset] == inTag`",
        "Tag updates": "`T[dstOffset] = inTag`",
    },
    {
        "id": "cast",
        "Name": "`CAST`",
        "Category": "Type Conversions",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
            {"name": "dstTag", "description": DST_TAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "aOffset", "description": "memory offset of word to cast"},
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = cast<dstTag>(M[aOffset])`",
        "Summary": "Type cast",
        "Details": "Cast a word in memory based on the `dstTag` specified in the bytecode. Truncates (`M[dstOffset] = M[aOffset] mod 2^dstsize`) when casting to a smaller type, left-zero-pads when casting to a larger type. See [here](./state-model#cast-and-tag-conversions) for more details.",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = dstTag`",
    },
    {
        "id": "address",
        "Name": "`ADDRESS`",
        "Category": "Execution Environment",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.environment.address`",
        "Summary": "Get the address of the currently executing l2 contract",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u32`",
    },
    {
        "id": "storageaddress",
        "Name": "`STORAGEADDRESS`",
        "Category": "Execution Environment",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.environment.storageAddress`",
        "Summary": "Get the _storage_ address of the currently executing context",
        "Details": "The storage address is used for public storage accesses.",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u32`",
    },
    {
        "id": "origin",
        "Name": "`ORIGIN`",
        "Category": "Execution Environment",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.environment.origin`",
        "Summary": "Get the transaction's origination address",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u32`",
    },
    {
        "id": "sender",
        "Name": "`SENDER`",
        "Category": "Execution Environment",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.environment.sender`",
        "Summary": "Get the address of the sender (caller of the current context)",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u32`",
    },
    {
        "id": "portal",
        "Name": "`PORTAL`",
        "Category": "Execution Environment",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.environment.portal`",
        "Summary": "Get the address of the l1 portal contract",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u32`",
    },
    {
        "id": "feeperl1gas",
        "Name": "`FEEPERL1GAS`",
        "Category": "Execution Environment",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.environment.feePerL1Gas`",
        "Summary": "Get the fee to be paid per \"L1 gas\" - constant for entire transaction",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u32`",
    },
    {
        "id": "feeperl2gas",
        "Name": "`FEEPERL2GAS`",
        "Category": "Execution Environment",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.environment.feePerL2Gas`",
        "Summary": "Get the fee to be paid per \"L2 gas\" - constant for entire transaction",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u32`",
    },
    {
        "id": "feeperdagas",
        "Name": "`FEEPERDAGAS`",
        "Category": "Execution Environment",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.environment.feePerDaGas`",
        "Summary": "Get the fee to be paid per \"DA gas\" - constant for entire transaction",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u32`",
    },
    {
        "id": "contractcalldepth",
        "Name": "`CONTRACTCALLDEPTH`",
        "Category": "Execution Environment",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.environment.contractCallDepth`",
        "Summary": "Get how many contract calls deep the current call context is",
        "Details": "Note: security issues with EVM's tx.origin can be resolved by asserting `calldepth == 0`.",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u8`",
    },
    {
        "id": "chainid",
        "Name": "`CHAINID`",
        "Category": "Execution Environment - Globals",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.environment.globals.chainId`",
        "Summary": "Get this rollup's L1 chain ID",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u32`",
    },
    {
        "id": "version",
        "Name": "`VERSION`",
        "Category": "Execution Environment - Globals",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.environment.globals.version`",
        "Summary": "Get this rollup's L2 version ID",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u32`",
    },
    {
        "id": "blocknumber",
        "Name": "`BLOCKNUMBER`",
        "Category": "Execution Environment - Globals",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.environment.globals.blocknumber`",
        "Summary": "Get this L2 block's number",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u32`",
    },
    {
        "id": "timestamp",
        "Name": "`TIMESTAMP`",
        "Category": "Execution Environment - Globals",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.environment.globals.timestamp`",
        "Summary": "Get this L2 block's timestamp",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u64`",
    },
    {
        "id": "coinbase",
        "Name": "`COINBASE`",
        "Category": "Execution Environment - Globals",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.environment.globals.coinbase`",
        "Summary": "Get the block's beneficiary address",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u32`",
    },
    {
        "id": "blockl1gaslimit",
        "Name": "`BLOCKL1GASLIMIT`",
        "Category": "Execution Environment - Globals",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.environment.globals.l1GasLimit`",
        "Summary": "Total amount of \"L1 gas\" that a block can consume",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u32`",
    },
    {
        "id": "blockl2gaslimit",
        "Name": "`BLOCKL2GASLIMIT`",
        "Category": "Execution Environment - Globals",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.environment.globals.l2GasLimit`",
        "Summary": "Total amount of \"L2 gas\" that a block can consume",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u32`",
    },
    {
        "id": "blockdagaslimit",
        "Name": "`BLOCKDAGASLIMIT`",
        "Category": "Execution Environment - Globals",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.environment.globals.daGasLimit`",
        "Summary": "Total amount of \"DA gas\" that a block can consume",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u32`",
    },
    {
        "id": "calldatacopy",
        "Name": "`CALLDATACOPY`",
        "Category": "Execution Environment - Calldata",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "cdOffset", "description": "offset into calldata to copy from"},
            {"name": "copySize", "description": "number of words to copy", "mode": "immediate", "type": "u32"},
            {"name": "dstOffset", "description": "memory offset specifying where to copy the first word to"},
        ],
        "Expression": "`M[dstOffset:dstOffset+copySize] = context.environment.calldata[cdOffset:cdOffset+copySize]`",
        "Summary": "Copy calldata into memory",
        "Details": "Calldata is read-only and cannot be directly operated on by other instructions. This instruction moves words from calldata into memory so they can be operated on normally.",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset:dstOffset+copySize] = field`",
    },
    {
        "id": "l1gasleft",
        "Name": "`L1GASLEFT`",
        "Category": "Machine State - Gas",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.machineState.l1GasLeft`",
        "Summary": "Remaining \"L1 gas\" for this call (after this instruction)",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u32`",
    },
    {
        "id": "l2gasleft",
        "Name": "`L2GASLEFT`",
        "Category": "Machine State - Gas",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.MachineState.l2GasLeft`",
        "Summary": "Remaining \"L2 gas\" for this call (after this instruction)",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u32`",
    },
    {
        "id": "dagasleft",
        "Name": "`DAGASLEFT`",
        "Category": "Machine State - Gas",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.machineState.daGasLeft`",
        "Summary": "Remaining \"DA gas\" for this call (after this instruction)",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = u32`",
    },
    {
        "id": "jump",
        "Name": "`JUMP`",
        "Category": "Machine State - Control Flow",
        "Flags": [],
        "Args": [
            {"name": "loc", "description": "target location to jump to", "mode": "immediate", "type": "u32"},
        ],
        "Expression": "`context.machineState.pc = loc`",
        "Summary": "Jump to a location in the bytecode",
        "Details": "Target location is an immediate value (a constant in the bytecode).",
        "Tag checks": "",
        "Tag updates": "",
    },
    {
        "id": "jumpi",
        "Name": "`JUMPI`",
        "Category": "Machine State - Control Flow",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "loc", "description": "target location conditionally jump to", "mode": "immediate", "type": "u32"},
            {"name": "condOffset", "description": "memory offset of the operations 'conditional' input"},
        ],
        "Expression": "`context.machineState.pc = M[condOffset] > 0 ? loc : context.machineState.pc`",
        "Summary": "Conditionally jump to a location in the bytecode",
        "Details": "Target location is an immediate value (a constant in the bytecode). `T[condOffset]` is not checked because the greater-than-zero suboperation is the same regardless of type.",
        "Tag checks": "",
        "Tag updates": "",
    },
    {
        "id": "internalcall",
        "Name": "`INTERNALCALL`",
        "Category": "Machine State - Control Flow",
        "Flags": [],
        "Args": [
            {"name": "loc", "description": "target location to jump/call to", "mode": "immediate", "type": "u32"},
        ],
        "Expression": `
context.machineState.internalCallStack.push(context.machineState.pc)
context.machineState.pc = loc
`,
        "Summary": "Make an internal call. Push the current PC to the internal call stack and jump to the target location.",
        "Details": "Target location is an immediate value (a constant in the bytecode).",
        "Tag checks": "",
        "Tag updates": "",
    },
    {
        "id": "internalreturn",
        "Name": "`INTERNALRETURN`",
        "Category": "Machine State - Control Flow",
        "Flags": [],
        "Args": [],
        "Expression": "`context.machineState.pc = context.machineState.internalCallStack.pop()`",
        "Summary": "Return from an internal call. Pop from the internal call stack and jump to the popped location.",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "",
    },
    {
        "id": "set",
        "Name": "`SET`",
        "Category": "Machine State - Memory",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
            {"name": "inTag", "description": "The [type/size](./state-model#tags-and-tagged-memory) to check inputs against and tag the destination with. `field` type is NOT supported for SET."},
        ],
        "Args": [
            {"name": "const", "description": "an N-bit constant value from the bytecode to store in memory (any type except `field`)", "mode": "immediate"},
            {"name": "dstOffset", "description": "memory offset specifying where to store the constant"},
        ],
        "Expression": "`M[dstOffset] = const`",
        "Summary": "Set a memory word from a constant in the bytecode",
        "Details": "Set memory word at `dstOffset` to `const`'s immediate value. `const`'s bit-size (N) can be 8, 16, 32, 64, or 128 based on `inTag`. It _cannot be 254 (`field` type)_!",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = inTag`",
    },
    {
        "id": "mov",
        "Name": "`MOV`",
        "Category": "Machine State - Memory",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "srcOffset", "description": "memory offset of word to move"},
            {"name": "dstOffset", "description": "memory offset specifying where to store that word"},
        ],
        "Expression": "`M[dstOffset] = M[srcOffset]`",
        "Summary": "Move a word from source memory location to destination",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = T[srcOffset]`",
    },
    {
        "id": "cmov",
        "Name": "`CMOV`",
        "Category": "Machine State - Memory",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "aOffset", "description": "memory offset of word 'a' to conditionally move"},
            {"name": "bOffset", "description": "memory offset of word 'b' to conditionally move"},
            {"name": "condOffset", "description": "memory offset of the operations 'conditional' input"},
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = M[condOffset] > 0 ? M[aOffset] : M[bOffset]`",
        "Summary": "Move a word (conditionally chosen) from one memory location to another (`d = cond > 0 ? a : b`)",
        "Details": "One of two source memory locations is chosen based on the condition. `T[condOffset]` is not checked because the greater-than-zero suboperation is the same regardless of type.",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = M[condOffset] > 0 ? T[aOffset] : T[bOffset]`",
    },
    {
        "id": "blockheaderbynum",
        "Name": "`BLOCKHEADERBYNUM`",
        "Category": "World State",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "blockNumOffset", "description": "memory offset of the block number input"},
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result's 0th word"},
        ],
        "Expression": "`M[dstOffset:dstOffset+BLOCK_HEADER_LENGTH] = context.worldState.blockHeader[M[blockNumOffset]]`",
        "Summary": "Get the block header as of the specified block number",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset:dstOffset+BLOCK_HEADER_LENGTh] = field`",
    },
    {
        "id": "sload",
        "Name": "`SLOAD`",
        "Category": "World State - Public Storage",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "slotOffset", "description": "memory offset of the storage slot to load from"},
            {"name": "dstOffset", "description": "memory offset specifying where to store operation's result"},
        ],
        "Expression": "`M[dstOffset] = context.worldState.publicStorage[context.environment.storageAddress, M[slotOffset]]`",
        "Summary": "Load a word from storage",
        "Details": "Load a word from this contract's persistent public storage into memory.",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset] = field`",
    },
    {
        "id": "sstore",
        "Name": "`SSTORE`",
        "Category": "World State - Public Storage",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "srcOffset", "description": "memory offset of the word to store"},
            {"name": "slotOffset", "description": "memory offset containing the storage slot to store to"},
        ],
        "Expression": "`context.worldState.publicStorage[context.environment.storageAddress, M[slotOffset]] = M[srcOffset]`",
        "Summary": "Write a word to storage",
        "Details": "Store a word from memory into this contract's persistent public storage.",
        "Tag checks": "",
        "Tag updates": "",
    },
    {
        "id": "readl1tol2msg",
        "Name": "`READL1TOL2MSG`",
        "Category": "World State - Messaging",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "msgKeyOffset", "description": "memory offset of the message's key"},
            {"name": "dstOffset", "description": "memory offset to place the 0th word of the message content"},
            {"name": "msgSize", "description": "number of words in the message", "mode": "immediate", "type": "u32"},
        ],
        "Expression": "`M[dstOffset:dstOffset+msgSize] = context.worldState.l1ToL2Messages(M[msgKeyOffset])`",
        "Summary": "Reads an L1-to-L2 message",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "`T[dstOffset:dstOffset+msgSize] = field`",
    },
    {
        "id": "sendl2tol1msg",
        "Name": "`SENDL2TOL1MSG`",
        "Category": "World State - Messaging",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "msgOffset", "description": "memory offset of the message content"},
            {"name": "msgSize", "description": "number of words in the message", "mode": "immediate", "type": "u32"},
        ],
        "Expression": "`context.worldState.l2ToL1Messages.append(M[msgOffset:msgOffset+msgSize])`",
        "Summary": "Send an L2-to-L1 message",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "",
    },
    {
        "id": "emitnotehash",
        "Name": "`EMITNOTEHASH`",
        "Category": "World State - Notes & Nullifiers",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "noteHashOffset", "description": "memory offset of the note hash"},
        ],
        "Expression": "`context.worldState.newHashes.append(M[noteHashOffset])`",
        "Summary": "Emit a new note hash to be inserted into the notes tree",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "",
    },
    {
        "id": "emitnullifier",
        "Name": "`EMITNULLIFIER`",
        "Category": "World State - Notes & Nullifiers",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "nullifierOffset", "description": "memory offset of nullifier"},
        ],
        "Expression": "`context.worldState.nullifiers.append(M[nullifierOffset])`",
        "Summary": "Emit a new nullifier to be inserted into the nullifier tree",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "",
    },
    {
        "id": "emitunencryptedlog",
        "Name": "`EMITUNENCRYPTEDLOG`",
        "Category": "Accrued Substate - Logging",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "logOffset", "description": "memory offset of the data to log"},
            {"name": "logSize", "description": "number of words to log", "mode": "immediate", "type": "u32"},
        ],
        "Expression": "`context.accruedSubstate.unencryptedLogs.append(M[logOffset:logOffset+logSize])`",
        "Summary": "Emit an unencrypted log",
        "Details": "",
        "Tag checks": "",
        "Tag updates": "",
    },
    {
        "id": "call",
        "Name": "`CALL`",
        "Category": "Control Flow - Contract Calls",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "gasOffset", "description": "offset to three words containing `{l1GasLeft, l2GasLeft, daGasLeft}`: amount of gas to provide to the callee"},
            {"name": "addrOffset", "description": "address of the contract to call"},
            {"name": "argsOffset", "description": "memory offset to args (will become the callee's calldata)"},
            {"name": "argsSize", "description": "number of words to pass via callee's calldata", "mode": "immediate", "type": "u32"},
            {"name": "retOffset", "description": "destination memory offset specifying where to store the data returned from the callee"},
            {"name": "retSize", "description": "number of words to copy from data returned by callee", "mode": "immediate", "type": "u32"},
            {"name": "successOffset", "description": "destination memory offset specifying where to store the call's success (0: failure, 1: success)", "type": "u8"},
        ],
        "Expression":`
M[successOffset] = call(
    M[gasOffset], M[gasOffset+1], M[gasOffset+2],
    M[addrOffset],
    M[argsOffset], M[argsSize],
    M[retOffset], M[retSize])
`,
        "Summary": "Call into another contract",
        "Details": `Creates a new (nested) execution context and triggers execution within it until the nested context halts.
                    Then resumes execution in the current/calling context. A non-existent contract or one with no code will return success.
                    See [\"Nested contract calls\"](./avm#nested-contract-calls) to see how the caller updates its context after the nested call halts.`,
        "Tag checks": "`T[gasOffset] == T[gasOffset+1] == T[gasOffset+2] == u32`",
        "Tag updates": `
T[successOffset] = u8
T[retOffset:retOffset+retSize] = field
`,
    },
    {
        "id": "staticcall",
        "Name": "`STATICCALL`",
        "Category": "Control Flow - Contract Calls",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "gasOffset", "description": "offset to three words containing `{l1GasLeft, l2GasLeft, daGasLeft}`: amount of gas to provide to the callee"},
            {"name": "addrOffset", "description": "address of the contract to call"},
            {"name": "argsOffset", "description": "memory offset to args (will become the callee's calldata)"},
            {"name": "argsSize", "description": "number of words to pass via callee's calldata", "mode": "immediate", "type": "u32"},
            {"name": "retOffset", "description": "destination memory offset specifying where to store the data returned from the callee"},
            {"name": "retSize", "description": "number of words to copy from data returned by callee", "mode": "immediate", "type": "u32"},
            {"name": "successOffset", "description": "destination memory offset specifying where to store the call's success (0: failure, 1: success)", "type": "u8"},
        ],
        "Expression": `
M[successOffset] = staticcall(
    M[gasOffset], M[gasOffset+1], M[gasOffset+2],
    M[addrOffset],
    M[argsOffset], M[argsSize],
    M[retOffset], M[retSize])
`,
        "Summary": "Call into another contract, disallowing World State and Accrued Substate modifications",
        "Details": "Same as `CALL`, but disallows World State and Accrued Substate modifications. See [\"Nested contract calls\"](./avm#nested-contract-calls) to see how the caller updates its context after the nested call halts.",
        "Tag checks": "`T[gasOffset] == T[gasOffset+1] == T[gasOffset+2] == u32`",
        "Tag updates": `
T[successOffset] = u8
T[retOffset:retOffset+retSize] = field
`,
    },
    {
        "id": "return",
        "Name": "`RETURN`",
        "Category": "Control Flow - Contract Calls",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "retOffset", "description": "memory offset of first word to return"},
            {"name": "retSize", "description": "number of words to return", "mode": "immediate", "type": "u32"},
        ],
        "Expression": `
context.contractCallResults.output = M[retOffset:retOffset+retSize]
halt
`,
        "Summary": "Halt execution within this context (without revert), optionally returning some data",
        "Details": "Return control flow to the calling context/contract. Caller will accept World State and Accrued Substate modifications. See [\"Halting\"](./avm#halting) to learn more. See [\"Nested contract calls\"](./avm#nested-contract-calls) to see how the caller updates its context after the nested call halts.",
        "Tag checks": "",
        "Tag updates": "",
    },
    {
        "id": "revert",
        "Name": "`REVERT`",
        "Category": "Control Flow - Contract Calls",
        "Flags": [
            {"name": "indirect", "description": INDIRECT_FLAG_DESCRIPTION},
        ],
        "Args": [
            {"name": "retOffset", "description": "memory offset of first word to return"},
            {"name": "retSize", "description": "number of words to return", "mode": "immediate", "type": "u32"},
        ],
        "Expression": `
context.contractCallResults.output = M[retOffset:retOffset+retSize]
context.contractCallResults.reverted = true
halt
`,
        "Summary": "Halt execution within this context as `reverted`, optionally returning some data",
        "Details": "Return control flow to the calling context/contract. Caller will reject World State and Accrued Substate modifications. See [\"Halting\"](./avm#halting) to learn more. See [\"Nested contract calls\"](./avm#nested-contract-calls) to see how the caller updates its context after the nested call halts.",
        "Tag checks": "",
        "Tag updates": "",
    },
];
const INSTRUCTION_SET = INSTRUCTION_SET_RAW.map((instr) => {instr['Bit-size'] = instructionSize(instr); return instr;});

module.exports = {
  TOPICS_IN_TABLE,
  TOPICS_IN_SECTIONS,
  INSTRUCTION_SET,
};
