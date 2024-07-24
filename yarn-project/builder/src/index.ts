import { type Command } from 'commander';
import { dirname } from 'path';

export function injectCommands(program: Command) {
  program
    .command('codegen')
    .argument('<noir-abi-path>', 'Path to the Noir ABI or project dir.')
    .option('-o, --outdir <path>', 'Output folder for the generated code.')
    .option('--force', 'Force code generation even when the contract has not changed.')
    .description('Validates and generates an Aztec Contract ABI from Noir ABI.')
    .action(async (noirAbiPath: string, { outdir, force }) => {
      const { generateCode } = await import('./contract-interface-gen/codegen.js');
      await generateCode(outdir || dirname(noirAbiPath), noirAbiPath, { force });
    });
  return program;
}
