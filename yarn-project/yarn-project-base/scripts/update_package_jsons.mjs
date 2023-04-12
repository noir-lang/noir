#!/usr/bin/env node

// Updates a package.json in the yarn-project folder based on inherits directives
// Run with --check to check for changes (exits with non-zero if any, useful for CI)

import { readFileSync, writeFileSync, readdirSync, statSync, existsSync } from 'fs';
import { resolve, dirname, join } from 'path';

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

async function main() {
  const checkOnly = process.argv[2] === '--check';
  console.log(
    checkOnly
      ? `Checking package.json consistency in packages...`
      : `Updating package.json in yarn-project based on custom inherits directive...`,
  );
  const files = getFiles();

  for (const file of files) {
    const packageData = JSON.parse(readFileSync(file));
    if (packageData.inherits) {
      let updated = false;
      for (const parent of packageData.inherits) {
        const parentFullPath = resolve(dirname(file), parent);
        const source = await getSource(parentFullPath);
        for (const key in source) {
          const updatedValue = {
            ...packageData[key],
            ...source[key],
          };
          updated = updated || JSON.stringify(updatedValue) !== JSON.stringify(packageData[key]);
          if (checkOnly) {
            if (updated) {
              console.error(
                `Section ${key} of ${file} is not up to date. Run 'yarn prepare' at the package root to fix.`,
                `\n Current: ${JSON.stringify(updatedValue)}`,
                `\n Updated: ${JSON.stringify(packageData[key])}`,
              );
              process.exit(2);
            }
          } else {
            packageData[key] = updatedValue;
          }
        }
      }
      if (!checkOnly && updated) {
        console.log(`Updated ${file}`);
        writeFileSync(file, JSON.stringify(packageData, null, 2) + '\n');
      }
    }
  }
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
