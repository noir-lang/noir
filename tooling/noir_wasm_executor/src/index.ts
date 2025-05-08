import { CompiledCircuit, InputMap, Noir } from '@noir-lang/noir_js';
import { readFileSync, writeFileSync } from 'node:fs';
import toml from 'toml';
import { Command } from 'commander';

async function executeProgram({
  bytecodePath,
  inputsPath,
  outputPath,
}: {
  bytecodePath: string;
  inputsPath: string;
  outputPath: string;
}): Promise<void> {
  const bytecode: CompiledCircuit = JSON.parse(readFileSync(bytecodePath).toString());

  const prover_toml = readFileSync(inputsPath).toString();
  const inputs: InputMap = toml.parse(prover_toml);

  const program = new Noir(bytecode);

  const { witness } = await program.execute(inputs);

  writeFileSync(outputPath, witness);
}

// Prepare a minimal command line interface
const program = new Command();

program
  .command('execute')
  .option('-b, --bytecode-path <path>', 'bytecode path')
  .option('-i, --inputs-path <path>', 'witness path')
  .option('-o, --output-path <path>', 'output path')
  .action((args) => executeProgram(args));

program.parse(process.argv);
