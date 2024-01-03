import { mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'fs';
import path from 'path';

import { generateContractArtifact } from '../contract-interface-gen/abi.js';
import { generateNoirContractInterface } from '../contract-interface-gen/noir.js';

/**
 *
 */
export function generateNoirInterface(outputPath: string, fileOrDirPath: string) {
  const stats = statSync(fileOrDirPath);

  if (stats.isDirectory()) {
    const files = readdirSync(fileOrDirPath).filter(file => file.endsWith('.json') && !file.startsWith('debug_'));
    for (const file of files) {
      const fullPath = path.join(fileOrDirPath, file);
      generateNoirInterfaceFromNoirAbi(outputPath, fullPath);
    }
  } else if (stats.isFile()) {
    generateNoirInterfaceFromNoirAbi(outputPath, fileOrDirPath);
  }
}

/**
 *
 */
export function generateNoirInterfaceFromNoirAbi(outputPath: string, noirAbiPath: string) {
  const contract = JSON.parse(readFileSync(noirAbiPath, 'utf8'));
  const aztecAbi = generateContractArtifact({ contract });
  const noirContract = generateNoirContractInterface(aztecAbi);
  mkdirSync(outputPath, { recursive: true });
  writeFileSync(`${outputPath}/${aztecAbi.name}.nr`, noirContract);
}
