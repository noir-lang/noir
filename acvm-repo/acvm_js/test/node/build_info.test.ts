import { expect } from 'chai';
import { BuildInfo, buildInfo } from '@noir-lang/acvm_js';
import child_process from 'child_process';
import pkg from '../../package.json';

it('returns the correct build info', () => {
  let revision: string;

  try {
    revision = process.env.GIT_COMMIT || child_process.execSync('git rev-parse HEAD').toString().trim();
  } catch (error) {
    console.log('Failed to get revision, skipping test.');
    return;
  }

  const info: BuildInfo = buildInfo();

  // TODO: enforce that `package.json` and `Cargo.toml` are consistent.
  expect(info.version).to.be.eq(pkg.version);

  expect(info.gitHash).to.be.eq(revision);
});
