#!/usr/bin/env node
import { Command, Option } from "commander";
const program = new Command();
import { chooseProject } from "./scripts/steps/chooseBox.js";
import { sandboxRun } from "./scripts/steps/sandbox/run.js";
import { sandboxInstallOrUpdate } from "./scripts/steps/sandbox/install.js";
import axios from "axios";
import pino from "pino";
import pretty from "pino-pretty";
import ora from "ora";
import { AZTEC_REPO } from "./scripts/config.js";

const getLatestStable = async () => {
  const { data } = await axios.get(
    `https://api.github.com/repos/AztecProtocol/aztec-packages/releases`,
  );
  return data[0].tag_name.split("-v")[1];
};

const init = async ({ debug, github_token, version }) => {
  const axiosOpts = {
    timeout: 5000,
    headers: github_token ? { Authorization: `token ${github_token}` } : {},
  };

  const prettyOpts = {
    sync: true,
    colorize: true,
    include: debug ? "time" : "",
    customLevels: "success:80",
    customColors: "success:bgGreen",
  };

  const prettyStream = pretty(prettyOpts);
  const logger = pino(
    {
      customLevels: {
        success: 80,
      },
      level: debug ? "debug" : "info",
    },
    prettyStream,
  );

  global.debug = (msg) => logger.debug(msg);
  global.info = (msg) => logger.info(msg);
  global.success = (msg) => logger.success(msg);

  global.warn = (msg) => logger.warn(msg);
  global.error = (msg) => logger.error(msg);

  global.github = async ({ path, raw = false }) => {
    try {
      const url = raw
        ? `https://raw.githubusercontent.com/${AZTEC_REPO}/${path}`
        : `https://api.github.com/repos/${AZTEC_REPO}/contents/${path}`;
      const { data } = await axios.get(url, axiosOpts);
      global.debug(data);
      return data;
    } catch (e) {
      global.error(e);
    }
  };

  // versioning is confusing here because "latest" and "master" point to the same thing at times
  // so let's clarify a bit:
  //
  // if the user has set a version (ex. "master" or "0.23.0"), use that
  // otherwise use the stable release (ex. 0.24.0)
  global.latestStable = await getLatestStable();
  global.version = version || global.latestStable;

  // if the user has set a semver version (matches the regex), fetch that tag (i.e. aztec-packages-v0.23.0)
  // otherwise use the version as the tag
  global.tag = global.version.match(/^\d+\.\d+\.\d+$/)
    ? `aztec-packages-v${global.version}`
    : global.version;

  global.debug(`Version: ${global.version}`);
  global.debug(`Tag: ${global.tag}`);
  global.debug(`LatestStable: ${global.latestStable}`);

  global.spinner = ora({ color: "blue" });
};

program.option("-d, --debug", "output extra debugging");
program.option("-gh, --github_token <github_token>", "a github token");
program.option("-v, --version <version>", "a version number or master tag");
program.option("-s, --skip-sandbox", "install and run sandbox after cloning");

program.option(
  "-t, --project-type <projectType>",
  "the type of the project to clone ('app' or 'contract')",
);
program.option(
  "-n, --project-name <projectName>",
  "the name of the project to clone",
);
program.parse();

// this is some bad code, but it's def fun
// I'm matching all keys started with project and
// then using using modulo to say "if one is defined, two must be defined"
const optsKeys = Object.keys(program.opts()).filter((e) => /project*/g.test(e));
if (optsKeys.length % 2) {
  throw Error("You must define both the project type and the project name");
}

program.action(async (options) => {
  const { projectType, projectName, skipSandbox } = options;
  // SETUP: Initialize global variables
  await init(options);
  // STEP 1: Choose the boilerplate
  await chooseProject({ projectType, projectName });

  if (skipSandbox) return;
  // STEP 2: Install the Sandbox
  await sandboxInstallOrUpdate({ skipQuestion: skipSandbox });
  // STEP 3: Running the Sandbox
  await sandboxRun({ skipQuestion: skipSandbox });
});
program.parse();
