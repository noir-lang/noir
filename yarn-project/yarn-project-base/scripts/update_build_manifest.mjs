#!/usr/bin/env node
// Updates build manifest for a package based on a package.json
import { existsSync, readFileSync, writeFileSync } from 'fs';
import { basename, dirname, join, resolve } from 'path';
import { cwd } from 'process';

// Update build_manifest.json with new dependencies
function updateBuildManifest(buildManifestFile, allDependencies, projectKey, options) {
  // Check if build_manifest.json exists
  if (!existsSync(buildManifestFile)) {
    console.error(`Error: ${buildManifestFile} not found (cwd ${cwd()}).`);
    process.exit(2);
  }

  // Read build_manifest.json
  const buildManifestData = JSON.parse(readFileSync(buildManifestFile, 'utf-8'));

  if (projectKey in buildManifestData) {
    // Filter package names from dependencies that start with "@aztec/"
    const aztecDependencies = Object.keys(allDependencies).filter(packageName => packageName.startsWith('@aztec/'));

    // Update the "dependencies" key in the corresponding section of the buildManifestData
    // Take just the folder name component
    const updatedDependencies = aztecDependencies.map(packageName => packageName.split('/')[1]);

    // If we are just checking, throw if dependencies don't match
    if (options.checkOnly) {
      const currentDependencies = buildManifestData[projectKey]['dependencies'];
      if (
        updatedDependencies.length !== currentDependencies.length ||
        !updatedDependencies.reduce((ret, val, idx) => ret && val === currentDependencies[idx], true)
      ) {
        console.error(
          `Dependencies for project ${projectKey} have changed and the build_manifest needs to be updated. Run yarn prepare on the yarn-project root.`,
          `\n Current: ${JSON.stringify(currentDependencies)}`,
          `\n Updated: ${JSON.stringify(updatedDependencies)}`,
        );
        process.exit(10);
      }
    }
    // Otherwise, update them
    else {
      buildManifestData[projectKey]['dependencies'] = updatedDependencies;
    }

    // Write the updated data back to build_manifest.json
    writeFileSync(buildManifestFile, JSON.stringify(buildManifestData, null, 2));
  } else {
    console.error(`Error: '${projectKey}' not found in build_manifest.json`);
    process.exit(3);
  }
}

// Entry point for the script
function main() {
  try {
    // Check if the path to the package.json file is provided as a command-line argument
    if (process.argv.length === 2) {
      console.error(`Usage: ${process.argv[0]} path/to/package.json`);
      process.exit(1);
    }

    const packageJsonFile = process.argv[2];

    // Check if package.json exists
    if (!existsSync(packageJsonFile)) {
      console.error(`Error: ${packageJsonFile} not found.`);
      process.exit(2);
    }

    // Process options if any
    const options = { checkOnly: false };
    for (const arg of process.argv.slice(3)) {
      if (arg === '--check') {
        options.checkOnly = true;
      } else {
        console.error(`Unknown option ${arg}`);
        process.exit(3);
      }
    }

    // Read package.json
    const packageData = JSON.parse(readFileSync(packageJsonFile, 'utf-8'));

    // Get the directory name of the directory that holds package.json
    const projectKey = basename(dirname(resolve(packageJsonFile)));

    // Add the path to the build-manifest.json file
    const buildManifestFile = join(dirname(packageJsonFile), '..', '..', 'build_manifest.json');

    // Update build_manifest.json with the new dependencies
    updateBuildManifest(
      buildManifestFile,
      { ...packageData.dependencies, ...packageData.devDependencies },
      projectKey,
      options,
    );
  } catch (err) {
    console.error(`Failed updating ${resolve(process.argv[2])}`);
    console.error(err);
  }
}
main();
