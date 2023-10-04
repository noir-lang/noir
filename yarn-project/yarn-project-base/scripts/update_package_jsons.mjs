#!/usr/bin/env node
// Updates a package.json in the yarn-project folder based on inherits directives
// Run with --check to check for changes (exits with non-zero if any, useful for CI)
import { existsSync, readFileSync, readdirSync, statSync, writeFileSync } from 'fs';
import { dirname, join, resolve } from 'path';

const sources = {};
async function getSource(fullpath) {
  if (!sources[fullpath]) {
    sources[fullpath] = JSON.parse(readFileSync(fullpath));
  }
  return sources[fullpath];
}

function getFiles() {
  const files = [];
  const dirs = readdirSync('.');
  for (const dir of dirs) {
    if (statSync(dir).isDirectory()) {
      const file = join(dir, 'package.json');
      if (existsSync(file)) {
        files.push(file);
      }
    }
  }
  return files;
}

function getUpdatedValue(source, target, key) {
  const value = source[key];
  if (typeof value === 'object' && !Array.isArray(value)) {
    return { ...target[key], ...value };
    // merge required files if a project requires more than what's in common
  } else if (key === 'files') {
    const res = [...target[key], ...value];
    return Array.from(new Set(res));
  } else {
    return value;
  }
}

function pick(obj, keys) {
  const result = {};
  for (const key of keys) {
    result[key] = obj[key];
  }
  return result;
}

async function main() {
  const checkOnly = process.argv[2] === '--check';
  console.log(
    checkOnly
      ? `Checking package.json consistency in packages...`
      : `Updating package.json in yarn-project based on custom inherits directive...`,
  );
  const files = getFiles();
  const updatedKeys = [];

  for (const file of files) {
    const contents = readFileSync(file);
    const packageData = JSON.parse(contents);
    const originalData = JSON.parse(contents);

    if (packageData.inherits) {
      for (const parent of packageData.inherits) {
        const parentFullPath = resolve(dirname(file), parent);
        const source = await getSource(parentFullPath);
        for (const key in source) {
          const updatedValue = getUpdatedValue(source, packageData, key);
          updatedKeys.push(key);
          packageData[key] = updatedValue;
        }
      }

      const updated =
        JSON.stringify(pick(originalData, updatedKeys)) !== JSON.stringify(pick(packageData, updatedKeys));
      if (!updated) continue;

      if (!checkOnly) {
        console.log(`Updated ${file}`);
        writeFileSync(file, JSON.stringify(packageData, null, 2) + '\n');
      } else {
        console.error(`File ${file} is not up to date. Run 'yarn prepare' at the package root to fix.`);
        process.exit(2);
      }
    }
  }
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
