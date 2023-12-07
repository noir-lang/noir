const {generateInstructionSet} = require('./InstructionSet/genMarkdown');

async function run() {
    await generateInstructionSet();
}
run();