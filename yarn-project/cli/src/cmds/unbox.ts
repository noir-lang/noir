import { LogFn } from '@aztec/foundation/log';

import { parse } from '@iarna/toml';
import { execSync } from 'child_process';
import { appendFileSync, cpSync, existsSync, readFileSync, writeFileSync } from 'fs';
import { dirname, resolve } from 'path';
import { fileURLToPath } from 'url';

import { prettyPrintNargoToml } from '../utils.js';

const resolutions: { [key: string]: string } = {
  '@aztec/accounts': 'portal:.aztec-packages/yarn-project/accounts',
  '@aztec/aztec.js': 'portal:.aztec-packages/yarn-project/aztec.js',
  '@aztec/circuits.js': 'portal:.aztec-packages/yarn-project/circuits.js',
  '@aztec/foundation': 'portal:.aztec-packages/yarn-project/foundation',
  '@aztec/bb.js': 'portal:.aztec-packages/barretenberg/ts',
  '@aztec/circuit-types': 'portal:.aztec-packages/yarn-project/circuit-types',
  '@aztec/ethereum': 'portal:.aztec-packages/yarn-project/ethereum',
  '@aztec/types': 'portal:.aztec-packages/yarn-project/types',
};

/**
 * Unboxes one of the pre-created projects in /boxes.
 * This involves copying the project to destination location, and making some adjustments.
 */
export function unbox(boxName: string, directory: string | undefined, cliVersion: string, _: LogFn) {
  const dirName = dirname(fileURLToPath(import.meta.url));
  const destPath = directory ? directory : boxName;
  const sourcePath = resolve(dirName + '/../../../../boxes/' + boxName);
  if (!existsSync(sourcePath)) {
    throw new Error(`Box ${boxName} not found at ${sourcePath}`);
  }

  // Copy the box to the destination.
  cpSync(sourcePath, destPath, { recursive: true });

  // We are considered a release if the COMMIT_TAG env var is set to something.
  const isRelease = !!process.env.COMMIT_TAG;
  if (isRelease) {
    packageJsonUpdatePackageVersions(`${destPath}/package.json`, () => cliVersion);
    nargoTomlUpdateToGithubDeps(`${destPath}/src/contracts/Nargo.toml`, cliVersion);
  } else {
    // Allows testing of unbox against whatever the local code is.
    // First copy in dev dependencies to .aztec-packages.
    copyDependenciesToBox(dirName, destPath);
    // Update config to point to dev dependencies.
    packageJsonInjectLocalResolutions(`${destPath}/package.json`);
    packageJsonUpdatePackageVersions(`${destPath}/package.json`, dep => resolutions[dep]);
    nargoTomlUpdateToDevPath(`${destPath}/src/contracts/Nargo.toml`);
    // Set box to use yarn berry, and node-modules linkage.
    process.chdir(destPath);
    execSync(`yarn set version berry`, { stdio: 'inherit' });
    appendFileSync(`.yarnrc.yml`, 'nodeLinker: node-modules');
  }
}

/**
 * If we're in dev, we need dev version of our dependencies.
 * Copy them to .aztec-packages/* within the box.
 * Ignore main space guzzlers.
 */
function copyDependenciesToBox(dirName: string, destPath: string) {
  [
    'barretenberg/ts',
    'yarn-project/aztec-nr',
    'yarn-project/noir-protocol-circuits',
    'yarn-project/accounts',
    'yarn-project/aztec.js',
    'yarn-project/circuits.js',
    'yarn-project/foundation',
    'yarn-project/types',
    'yarn-project/circuit-types',
    'yarn-project/ethereum',
  ].forEach(path =>
    cpSync(dirName + '/../../../../' + path, destPath + '/.aztec-packages/' + path, {
      recursive: true,
      dereference: true,
      filter: source => !(source.includes('node_modules') || source.includes('.yarn/')),
    }),
  );
}

/**
 *
 */
function packageJsonInjectLocalResolutions(path: string) {
  const data = readFileSync(path, 'utf-8');
  const packageJson = JSON.parse(data);
  packageJson.resolutions = resolutions;
  writeFileSync(path, JSON.stringify(packageJson, null, 2), 'utf-8');
}

/**
 * Call fn to get new version for each known aztec package.
 * TODO: Should this be better done with release-please?
 */
function packageJsonUpdatePackageVersions(path: string, fn: (dep: string) => string) {
  const data = readFileSync(path, 'utf-8');
  const packageJson = JSON.parse(data);

  ['dependencies', 'devDependencies', 'peerDependencies', 'optionalDependencies'].forEach(section => {
    if (packageJson[section]) {
      Object.keys(packageJson[section]).forEach(dependency => {
        if (resolutions[dependency]) {
          packageJson[section][dependency] = fn(dependency);
        }
      });
    }
  });

  writeFileSync(path, JSON.stringify(packageJson, null, 2), 'utf-8');
}

/**
 * Adjust the dependency paths from pointing to yarn-project, to the tagged version in github.
 */
function nargoTomlUpdateToGithubDeps(path: string, cliVersion: string) {
  const data = readFileSync(path, 'utf-8');
  const content: any = parse(data);
  const tag = `aztec-packages-v${cliVersion}`;

  Object.keys(content.dependencies).forEach(dep => {
    const dependency = content.dependencies[dep];

    if (dependency.path && dependency.path.startsWith('../../../../yarn-project/')) {
      const directory = dependency.path.replace('../../../../', '');
      content.dependencies[dep] = {
        git: 'https://github.com/AztecProtocol/aztec-packages/',
        tag,
        directory: directory,
      };
    }
  });

  const updatedToml = prettyPrintNargoToml(content);

  writeFileSync(path, updatedToml, 'utf-8');
}

/**
 * Adjust the dependency paths from pointing to yarn-project, to the code copied into .aztec-packages.
 */
function nargoTomlUpdateToDevPath(path: string) {
  const data = readFileSync(path, 'utf-8');
  const content: any = parse(data);

  Object.keys(content.dependencies).forEach(dep => {
    const dependency = content.dependencies[dep];

    if (dependency.path && dependency.path.startsWith('../../../../yarn-project/')) {
      const directory = dependency.path.replace('../../../../', '../../.aztec-packages/');
      content.dependencies[dep].path = directory;
    }
  });

  const updatedToml = prettyPrintNargoToml(content);

  writeFileSync(path, updatedToml, 'utf-8');
}
