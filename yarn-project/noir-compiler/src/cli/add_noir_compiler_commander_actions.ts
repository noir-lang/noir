import { LogFn } from '@aztec/foundation/log';

import { Command } from 'commander';

/**
 * CLI options for configuring behavior
 */
interface Options {
  // eslint-disable-next-line jsdoc/require-jsdoc
  outdir: string;
  // eslint-disable-next-line jsdoc/require-jsdoc
  typescript: string | undefined;
  // eslint-disable-next-line jsdoc/require-jsdoc
  interface: string | undefined;
  // eslint-disable-next-line jsdoc/require-jsdoc
  compiler: string | undefined;
}

/**
 *
 */
export function addNoirCompilerCommanderActions(program: Command, log: LogFn = () => {}) {
  program
    .command('compile')
    .argument('<project-path>', 'Path to the bin or Aztec.nr project to compile')
    .option('-o, --outdir <path>', 'Output folder for the binary artifacts, relative to the project path', 'target')
    .option('-ts, --typescript <path>', 'Optional output folder for generating typescript wrappers', undefined)
    .option('-i, --interface <path>', 'Optional output folder for generating an Aztec.nr contract interface', undefined)
    .option('-c --compiler <string>', 'Which compiler to use. Either nargo or wasm.', 'wasm')
    .description('Compiles the Noir Source in the target project')

    .action(async (projectPath: string, options: Options) => {
      const { compileNoir } = await import('./compile_noir.js');
      await compileNoir(projectPath, options, log);
    });

  program
    .command('generate-typescript')
    .argument('<project-path>', 'Path to the noir project')
    .option('--artifacts <path>', 'Folder containing the compiled artifacts, relative to the project path', 'target')
    .option(
      '-o, --outdir <path>',
      'Output folder for the generated noir interfaces, relative to the project path',
      'interfaces',
    )
    .description('Generates Noir interfaces from the artifacts in the given project')

    .action(async (projectPath: string, options) => {
      const { generateTypescriptInterface } = await import('./generate_typescript_interface.js');
      generateTypescriptInterface(projectPath, options, log);
    });

  return program
    .command('generate-noir-interface')
    .argument('<project-path>', 'Path to the noir project')
    .option('--artifacts <path>', 'Folder containing the compiled artifacts, relative to the project path', 'target')
    .option(
      '-o, --outdir <path>',
      'Output folder for the generated noir interfaces, relative to the project path',
      'interfaces',
    )
    .description('Generates Noir interfaces from the artifacts in the given project')
    .action(async (projectPath: string, options) => {
      const { generateNoirInterface } = await import('./generate_noir_interface.js');
      generateNoirInterface(projectPath, options, log);
    });
}
