/* eslint-disable no-console */
import { loadContractArtifact } from '@aztec/types/abi';

import crypto from 'crypto';
import { existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'fs';
import path from 'path';

import { generateTypescriptContractInterface } from './typescript.js';

const cacheFilePath = './codegenCache.json';
let cache: Record<string, string> = {};

/** Generate code options */
export type GenerateCodeOptions = { force?: boolean };

/**
 * Generates Noir interface or Typescript interface for a folder or single file from a Noir compilation artifact.
 */
export function generateCode(outputPath: string, fileOrDirPath: string, opts: GenerateCodeOptions = {}) {
  readCache();
  const stats = statSync(fileOrDirPath);

  if (stats.isDirectory()) {
    const files = readdirSync(fileOrDirPath, { recursive: true, encoding: 'utf-8' }).filter(
      file => file.endsWith('.json') && !file.startsWith('debug_'),
    );
    for (const file of files) {
      const fullPath = path.join(fileOrDirPath, file);
      generateFromNoirAbi(outputPath, fullPath, opts);
    }
  } else if (stats.isFile()) {
    generateFromNoirAbi(outputPath, fileOrDirPath, opts);
  }
  writeCache();
}

/**
 * Generates Noir interface or Typescript interface for a single file Noir compilation artifact.
 */
function generateFromNoirAbi(outputPath: string, noirAbiPath: string, opts: GenerateCodeOptions = {}) {
  const contractName = path.basename(noirAbiPath);
  const currentHash = generateFileHash(noirAbiPath);

  if (isCacheValid(contractName, currentHash) && !opts.force) {
    console.log(`${contractName} has not changed. Skipping generation.`);
    return;
  }

  const contract = JSON.parse(readFileSync(noirAbiPath, 'utf8'));
  const aztecAbi = loadContractArtifact(contract);

  mkdirSync(outputPath, { recursive: true });

  let relativeArtifactPath = path.relative(outputPath, noirAbiPath);
  if (relativeArtifactPath === path.basename(noirAbiPath)) {
    // Prepend ./ for local import if the folder is the same
    relativeArtifactPath = `./${relativeArtifactPath}`;
  }

  const tsWrapper = generateTypescriptContractInterface(aztecAbi, relativeArtifactPath);
  writeFileSync(`${outputPath}/${aztecAbi.name}.ts`, tsWrapper);

  updateCache(contractName, currentHash);
}

function generateFileHash(filePath: string) {
  const fileBuffer = readFileSync(filePath);
  const hashSum = crypto.createHash('sha256');
  hashSum.update(fileBuffer);
  const hex = hashSum.digest('hex');
  return hex;
}

function readCache(): void {
  if (existsSync(cacheFilePath)) {
    const cacheRaw = readFileSync(cacheFilePath, 'utf8');
    cache = JSON.parse(cacheRaw);
  }
}

function writeCache(): void {
  writeFileSync(cacheFilePath, JSON.stringify(cache, null, 2), 'utf8');
}

function isCacheValid(contractName: string, currentHash: string): boolean {
  return cache[contractName] === currentHash;
}

function updateCache(contractName: string, hash: string): void {
  cache[contractName] = hash;
}
