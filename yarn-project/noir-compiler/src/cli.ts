#!/usr/bin/env node
import nodePath from 'path';
import fs from 'fs/promises';
import fsExtra from 'fs-extra';
import { Command } from 'commander';
import { ContractCompiler } from './compile.js';

const program = new Command();

const main = async () => {
  program
    .name('aztec_noir_compiler')
    .command('compile')
    .argument('[path]', 'Path to the contract project', '.')
    .action(async (path: string) => {
      const projectPath = nodePath.resolve(path);

      const compiler = new ContractCompiler(projectPath);
      const contracts = await compiler.compile();

      const buildFolderPath = nodePath.join(projectPath, 'target');

      await fsExtra.mkdirp(buildFolderPath);

      for (const contract of contracts) {
        const contractPath = nodePath.join(buildFolderPath, `aztec-${contract.name}.json`);
        await fs.writeFile(contractPath, JSON.stringify(contract, null, 2));
      }
    });

  await program.parseAsync(process.argv);
};

main().catch(err => {
  console.log(`Error thrown: ${err}`);
  process.exit(1);
});
