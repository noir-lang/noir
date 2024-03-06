#!/usr/bin/env node
import { Command } from "commander";
const program = new Command();
import { chooseAndCloneBox } from "./scripts/steps/chooseBox.js";
import { sandboxRun } from "./scripts/steps/sandbox/run.js";
import { sandboxInstallOrUpdate } from "./scripts/steps/sandbox/install.js";
import { axios } from "./scripts/utils.js";

const getLatestStable = async () => {
  const { data } = await axios.get(
    `https://api.github.com/repos/AztecProtocol/aztec-packages/releases`,
  );
  return data[0].tag_name.split("-v")[1];
};

// versioning is confusing here because "latest" and "master" point to the same thing at times
// so let's clarify a bit:
//
// if the user has set a version (ex. "master" or "0.23.0"), use that
// otherwise use the stable release (ex. 0.24.0)
const latestStable = await getLatestStable();
const versionToInstall = process.env.VERSION || latestStable;

// if the user has set a semver version (matches the regex), fetch that tag (i.e. aztec-packages-v0.23.0)
// otherwise use the version as the tag
const tagToUse = versionToInstall.match(/^\d+\.\d+\.\d+$/)
  ? `aztec-packages-v${versionToInstall}`
  : versionToInstall;

program.action(async () => {
  // STEP 1: Choose the boilerplate
  await chooseAndCloneBox(tagToUse, versionToInstall);

  // STEP 2: Install the Sandbox
  await sandboxInstallOrUpdate(latestStable, versionToInstall);

  // STEP 3: Running the Sandbox
  await sandboxRun(versionToInstall);
});

program.parse();
