const FIELD_SIZES = { // in bits
    "Opcode": 8,
    "Indirect": 8,
};

const DEFAULT_OPERAND_SIZE = 24; // for direct/indirect memory offsets

function argSize(arg) {
    if (arg['mode'] && arg['mode'] == 'immediate') {
        if (arg['type']) {
            return Number(arg['type'].replace(/u/, ''));
        } else {
            return undefined; // none specified!
        }
    } else {
        return DEFAULT_OPERAND_SIZE;
    }
}

function toOpcode(index) {
    return '0x' + index.toString(16).padStart(2, '0');
}

/* Compute bit-size of instruction based on flags and number of operands,
 * whether they are immediate (and op-type if so)
 *
 * All instructions have:
 *   - 1 byte for opcode
 *   - 1 byte to toggle indirect mode for up to 8 non-immediate args
 * 24 bits per-arg (for non-immediates)
 * N bits per immediate arg, where N is 8, 16, 32, 64, or 128 based on type
 * 1 byte for op-type
 * 1 byte for dest-type
 */
function instructionSize(instr) {
    let size = FIELD_SIZES['Opcode'] + FIELD_SIZES['Indirect'];
    let numUntypedImmediates = 0;
    for (let arg of instr['Args']) {
        const aSize = argSize(arg);
        if (aSize === undefined) {
            numUntypedImmediates++;
        } else {
            size += aSize;
        }
    }
    if (instr['Flags']) {
        // assigns each flag a byte (op-type, dest-type)
        size += instr['Flags'].length * 8;
    }
    let sizeStr = size.toString();
    if (numUntypedImmediates > 0) {
        sizeStr += '+N';
    }
    return sizeStr;
}

function instructionBitFormat(instr, index) {
    let bitFormat = { 'Name': instr['Name'], 'Opcode': {'code': toOpcode(index), 'size': 8}, 'Indirect': 8, 'Args': [], 'Flags': [] };

    //for (let arg of instr['Args']) {
    for (let a = 0; a < instr['Args'].length; a++) {
        const arg = instr['Args'][a];
        const aSize = argSize(arg);
        if (aSize === undefined) {
            bitFormat['Args'][a] = {"name": arg['name'], "size": 'N'};
        } else {
            bitFormat['Args'][a] = {"name": arg['name'], "size": aSize};
        }
    }
    for (let f = 0; f < instr['Flags'].length; f++) {
        const flag = instr['Flags'][f];
        bitFormat['Flags'][f] = {"name": flag['name'], "size": 8};
    }
    return bitFormat;
}

module.exports = {
  instructionSize,
  instructionBitFormat,
};