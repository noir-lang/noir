import { loadContractArtifact } from '@aztec/types/abi';

import { mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'fs';
import path from 'path';

import { generateNoirContractInterface } from '../contract-interface-gen/noir.js';
import { generateTypescriptContractInterface } from '../contract-interface-gen/typescript.js';

/** Generate code options */
type GenerateCodeOptions = { /** Typescript */ ts?: boolean; /** Noir */ nr?: boolean };

/**
 * Generates Noir interface or Typescript interface for a folder or single file from a Noir compilation artifact.
 */
export function generateCode(outputPath: string, fileOrDirPath: string, opts: GenerateCodeOptions = {}) {
  const stats = statSync(fileOrDirPath);

  if (stats.isDirectory()) {
    const files = readdirSync(fileOrDirPath).filter(file => file.endsWith('.json') && !file.startsWith('debug_'));
    for (const file of files) {
      const fullPath = path.join(fileOrDirPath, file);
      generateFromNoirAbi(outputPath, fullPath, opts);
    }
  } else if (stats.isFile()) {
    generateFromNoirAbi(outputPath, fileOrDirPath, opts);
  }
}

/**
 * Generates Noir interface or Typescript interface for a single file Noir compilation artifact.
 */
function generateFromNoirAbi(outputPath: string, noirAbiPath: string, opts: GenerateCodeOptions = {}) {
  const contract = JSON.parse(readFileSync(noirAbiPath, 'utf8'));
  const aztecAbi = loadContractArtifact(contract);
  const { nr, ts } = opts;

  mkdirSync(outputPath, { recursive: true });

  if (nr) {
    const noirContract = generateNoirContractInterface(aztecAbi);
    writeFileSync(`${outputPath}/${aztecAbi.name}.nr`, noirContract);
    return;
  }

  if (ts) {
    let relativeArtifactPath = path.relative(outputPath, noirAbiPath);
    if (relativeArtifactPath === path.basename(noirAbiPath)) {
      // Prepend ./ for local import if the folder is the same
      relativeArtifactPath = `./${relativeArtifactPath}`;
    }

    const tsWrapper = generateTypescriptContractInterface(aztecAbi, relativeArtifactPath);
    writeFileSync(`${outputPath}/${aztecAbi.name}.ts`, tsWrapper);
  }
}
