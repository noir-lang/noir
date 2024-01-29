import { LogFn } from '@aztec/foundation/log';

import { Command } from 'commander';
import { dirname } from 'path';

export function addNoirCompilerCommanderActions(program: Command, log: LogFn = () => {}) {
  addCodegenCommanderAction(program, log);
}

export function addCodegenCommanderAction(program: Command, _: LogFn = () => {}) {
  program
    .command('codegen')
    .argument('<noir-abi-path>', 'Path to the Noir ABI or project dir.')
    .option('-o, --outdir <path>', 'Output folder for the generated code.')
    .option('--ts', 'Generate TypeScript wrapper.')
    .option('--nr', 'Generate Noir interface.')
    .description('Validates and generates an Aztec Contract ABI from Noir ABI.')
    .action(async (noirAbiPath: string, { outdir, ts, nr }) => {
      if (ts && nr) {
        throw new Error('--ts and --nr are mutually exclusive.');
      }
      const { generateCode } = await import('./codegen.js');
      generateCode(outdir || dirname(noirAbiPath), noirAbiPath, { ts, nr });
    });
}
