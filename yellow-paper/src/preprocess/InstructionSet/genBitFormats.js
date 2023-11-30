const fs = require("fs");

const {instructionBitFormat} = require('./InstructionSize');
const {INSTRUCTION_SET} = require('./InstructionSet');

function run() {
    const formats = [];
    for (let i = 0; i < INSTRUCTION_SET.length; i++) {
        const instr = INSTRUCTION_SET[i];
        const bitFormat = instructionBitFormat(instr, i);
        console.log(JSON.stringify(bitFormat));
        formats.push(bitFormat);
    }
    fs.writeFileSync('./InstructionBitFormats.json', JSON.stringify(formats));
}
run();