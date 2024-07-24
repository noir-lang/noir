/* eslint-disable no-console */
import { loadContractArtifact } from '@aztec/types/abi';

import crypto from 'crypto';
import { access, mkdir, readFile, readdir, stat, writeFile } from 'fs/promises';
import path from 'path';

import { generateTypescriptContractInterface } from './typescript.js';

const cacheFilePath = './codegenCache.json';
let cache: Record<string, { contractName: string; hash: string }> = {};

/** Generate code options */
export type GenerateCodeOptions = { force?: boolean };

/**
 * Generates Noir interface or Typescript interface for a folder or single file from a Noir compilation artifact.
 */
export async function generateCode(outputPath: string, fileOrDirPath: string, opts: GenerateCodeOptions = {}) {
  await readCache();
  const results = [];
  const stats = await stat(fileOrDirPath);

  if (stats.isDirectory()) {
    const files = (await readdir(fileOrDirPath, { recursive: true, encoding: 'utf-8' })).filter(
      file => file.endsWith('.json') && !file.startsWith('debug_'),
    );
    for (const file of files) {
      const fullPath = path.join(fileOrDirPath, file);
      results.push(await generateFromNoirAbi(outputPath, fullPath, opts));
    }
  } else if (stats.isFile()) {
    results.push(await generateFromNoirAbi(outputPath, fileOrDirPath, opts));
  }
  await writeCache();
  return results;
}

/**
 * Generates Noir interface or Typescript interface for a single file Noir compilation artifact.
 */
async function generateFromNoirAbi(outputPath: string, noirAbiPath: string, opts: GenerateCodeOptions = {}) {
  const fileName = path.basename(noirAbiPath);
  const currentHash = await generateFileHash(noirAbiPath);
  const cachedInstance = isCacheValid(fileName, currentHash);
  if (cachedInstance && !opts.force) {
    console.log(`${fileName} has not changed. Skipping generation.`);
    return `${outputPath}/${cachedInstance.contractName}.ts`;
  }

  const file = await readFile(noirAbiPath, 'utf8');
  const contract = JSON.parse(file);
  const aztecAbi = loadContractArtifact(contract);

  await mkdir(outputPath, { recursive: true });

  let relativeArtifactPath = path.relative(outputPath, noirAbiPath);
  if (relativeArtifactPath === path.basename(noirAbiPath)) {
    // Prepend ./ for local import if the folder is the same
    relativeArtifactPath = `./${relativeArtifactPath}`;
  }

  const tsWrapper = generateTypescriptContractInterface(aztecAbi, relativeArtifactPath);
  const outputFilePath = `${outputPath}/${aztecAbi.name}.ts`;

  await writeFile(outputFilePath, tsWrapper);

  updateCache(fileName, aztecAbi.name, currentHash);
  return outputFilePath;
}

async function generateFileHash(filePath: string) {
  const fileBuffer = await readFile(filePath);
  const hashSum = crypto.createHash('sha256');
  hashSum.update(fileBuffer);
  const hex = hashSum.digest('hex');
  return hex;
}

async function readCache() {
  if (await exists(cacheFilePath)) {
    const cacheRaw = await readFile(cacheFilePath, 'utf8');
    cache = JSON.parse(cacheRaw);
  }
}

async function writeCache() {
  await writeFile(cacheFilePath, JSON.stringify(cache, null, 2), 'utf8');
}

function isCacheValid(contractName: string, currentHash: string) {
  return cache[contractName]?.hash === currentHash && cache[contractName];
}

function updateCache(fileName: string, contractName: string, hash: string): void {
  cache[fileName] = { contractName, hash };
}

async function exists(filePath: string) {
  try {
    await access(filePath);
    return true;
  } catch {
    return false;
  }
}
