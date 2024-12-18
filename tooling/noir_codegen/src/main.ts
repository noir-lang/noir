#! /usr/bin/env node

import { CompiledCircuit } from '@noir-lang/types';
import fs from 'fs';
import path from 'path';
import { parseArgs } from './parseArgs.js';
import { glob } from './utils/glob.js';
import { codegen } from './index.js';

function main() {
  const cliConfig = parseArgs();
  const cwd = process.cwd();

  const files = getFilesToProcess(cwd, cliConfig.files);
  if (files.length === 0) {
    throw new Error('No files passed.' + '\n' + `\`${cliConfig.files}\` didn't match any input files in ${cwd}`);
  }

  const programs = files.map((file_path): [string, CompiledCircuit] => {
    const program_name = path.parse(file_path).name;
    const file_contents = fs.readFileSync(file_path).toString();
    const { abi, bytecode } = JSON.parse(file_contents);

    return [program_name, { abi, bytecode }];
  });

  const result = codegen(programs, !cliConfig.externalArtifact, cliConfig.useFixedLengthArrays);

  const outputDir = path.resolve(cliConfig.outDir ?? './codegen');
  const outputFile = path.join(outputDir, 'index.ts');
  if (!fs.existsSync(outputDir)) fs.mkdirSync(outputDir);
  fs.writeFileSync(outputFile, result);
}

function getFilesToProcess(cwd: string, filesOrPattern: string[]) {
  let res = glob(cwd, filesOrPattern);

  if (res.length === 0) {
    // If there are no files found, but first parameter is surrounded with single quotes, we try again without quotes
    const regex = /(['\S\s]*)/;
const result = regex.exec(filesOrPattern[0]);
const match = result ? result[1] : null;
    if (match) res = glob(cwd, [match]);
  }

  return res;
}

main();
