const {generateInstructionSet} = require('./InstructionSet/InstructionSetMarkdownGen');

async function run() {
    await generateInstructionSet();
}
run();