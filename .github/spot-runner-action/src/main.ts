import * as core from "@actions/core";
import * as fs from "fs";
import { execSync } from "child_process";
import { ActionConfig } from "./config";
import { Ec2Instance } from "./ec2";
import { GithubClient } from "./github";
import { assertIsError } from "./utils";
import { spawn } from "child_process";
import * as github from "@actions/github";
require("aws-sdk/lib/maintenance_mode_message").suppress = true;

async function pollSpotStatus(
  config: ActionConfig,
  ec2Client: Ec2Instance,
  ghClient: GithubClient
): Promise<string | "unusable" | "none"> {
  // 6 iters x 10000 ms = 1 minute
  for (let iter = 0; iter < 6; iter++) {
    const instances = await ec2Client.getInstancesForTags("running");
    if (instances.length <= 0) {
      // we need to start an instance
      return "none";
    }
    try {
      core.info("Found ec2 instance, looking for runners.");
      if (await ghClient.hasRunner([config.githubJobId])) {
        // we have runners
        return instances[0].InstanceId!;
      }
    } catch (err) {}
    // wait 10 seconds
    await new Promise((r) => setTimeout(r, 10000));
  }
  // we have a bad state for a while, error
  core.warning(
    "Looped for 1 minutes and could only find spot with no runners!"
  );
  return "unusable";
}

async function requestAndWaitForSpot(config: ActionConfig): Promise<string> {
  // subaction is 'start' or 'restart'estart'
  const ec2Client = new Ec2Instance(config);

  let ec2SpotStrategies: string[];
  switch (config.ec2SpotInstanceStrategy) {
    case "besteffort": {
      ec2SpotStrategies = ["BestEffort", "none"];
      core.info(
        "Ec2 spot instance strategy is set to 'BestEffort' with 'None' as fallback"
      );
      break;
    }
    default: {
      ec2SpotStrategies = [config.ec2SpotInstanceStrategy];
      core.info(
        `Ec2 spot instance strategy is set to ${config.ec2SpotInstanceStrategy}`
      );
    }
  }

  let instanceId = "";
  for (const ec2Strategy of ec2SpotStrategies) {
    let backoff = 1;
    core.info(`Starting instance with ${ec2Strategy} strategy`);
    // 6 * 10000ms = 1 minute per strategy, unless we hit RequestLimitExceeded, then we do exponential backoff
    // TODO make longer lived spot request?
    for (let i = 0; i < 6; i++) {
      try {
        // Start instance
        instanceId =
          await ec2Client.requestMachine(
            // we fallback to on-demand
            ec2Strategy.toLocaleLowerCase() === "none"
          );
        // let's exit, only loop on InsufficientInstanceCapacity
        if (instanceId !== "RequestLimitExceeded") {
          break;
        }
      } catch (error) {
        // TODO is this still the relevant error?
        if (
          error?.code &&
          error.code === "InsufficientInstanceCapacity" &&
          ec2SpotStrategies.length > 0 &&
          ec2Strategy.toLocaleLowerCase() != "none"
        ) {
          core.info(
            "Failed to create instance due to 'InsufficientInstanceCapacity', waiting 10 seconds and trying again."
          );
          // we loop after 10 seconds
        } else {
          throw error;
        }
      }
      // wait 10 seconds
      await new Promise((r) => setTimeout(r, 10000 * 2 ** backoff));
    }
    if (instanceId) {
      core.info("Successfully requested instance with ID " + instanceId);
      break;
    }
  }
  if (instanceId) await ec2Client.waitForInstanceRunningStatus(instanceId);
  else {
    core.error("Failed to get ID of running instance");
    throw Error("Failed to get ID of running instance");
  }
  return instanceId;
}

async function startBareSpot(config: ActionConfig) {
  if (config.subaction !== "start") {
    throw new Error(
      "Unexpected subaction for bare spot, only 'start' is allowed: " +
        config.subaction
    );
  }
  const ec2Client = new Ec2Instance(config);
  const instanceId = await requestAndWaitForSpot(config);
  const ip = await ec2Client.getPublicIpFromInstanceId(instanceId);

  const tempKeyPath = installSshKey(config.ec2Key);
  core.info("Logging SPOT_IP and SPOT_KEY to GITHUB_ENV for later step use.");
  await standardSpawn("bash", ["-c", `echo SPOT_IP=${ip} >> $GITHUB_ENV`]);
  await standardSpawn("bash", [
    "-c",
    `echo SPOT_KEY=${tempKeyPath} >> $GITHUB_ENV`,
  ]);
  await establishSshContact(ip, config.ec2Key);
}

async function startWithGithubRunners(config: ActionConfig) {
  if (config.subaction === "stop") {
    await terminate();
    return "";
  } else if (config.subaction === "restart") {
    await terminate();
    // then we make a fresh instance
  } else if (config.subaction !== "start") {
    throw new Error("Unexpected subaction: " + config.subaction);
  }
  // subaction is 'start' or 'restart'estart'
  const ec2Client = new Ec2Instance(config);
  const ghClient = new GithubClient(config);
  let spotStatus = await pollSpotStatus(config, ec2Client, ghClient);
  if (spotStatus === "unusable") {
    core.warning(
      "Taking down spot as it has no runners! If we were mistaken, this could impact existing jobs."
    );
    if (config.subaction === "restart") {
      throw new Error(
        "Taking down spot we just started. This seems wrong, erroring out."
      );
    }
    await terminate();
    spotStatus = "none";
  }
  let instanceId = "";
  let ip = "";
  if (spotStatus !== "none") {
    core.info(
      `Runner already running. Continuing as we can target it with jobs.`
    );
    instanceId = spotStatus;
    ip = await ec2Client.getPublicIpFromInstanceId(instanceId);
    if (!(await establishSshContact(ip, config.ec2Key))) {
      return false;
    }
  } else {
    core.info(
      `Starting runner.`
    );
    instanceId = await requestAndWaitForSpot(config);
    ip = await ec2Client.getPublicIpFromInstanceId(instanceId);
    if (!(await establishSshContact(ip, config.ec2Key))) {
      return false;
    }
    await setupGithubRunners(ip, config);
    if (instanceId) await ghClient.pollForRunnerCreation([config.githubJobId]);
    else {
      core.error("Instance failed to register with Github Actions");
      throw Error("Instance failed to register with Github Actions");
    }
    core.info("Done setting up runner.")
  }
  // Export to github environment
  const tempKeyPath = installSshKey(config.ec2Key);
  core.info("Logging BUILDER_SPOT_IP and BUILDER_SPOT_KEY to GITHUB_ENV for later step use.");
  await standardSpawn("bash", ["-c", `echo BUILDER_SPOT_IP=${ip} >> $GITHUB_ENV`]);
  await standardSpawn("bash", [
    "-c",
    `echo BUILDER_SPOT_KEY=${tempKeyPath} >> $GITHUB_ENV`,
  ]);
  return true;
}

function standardSpawn(command: string, args: string[]): Promise<string> {
  // Wrap the process execution in a Promise to handle asynchronous execution and output streaming
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {stdio: 'inherit'});

    // Handle close event
    child.on("close", (code) => {
      if (code === 0) {
        resolve(`SSH command completed with code ${code}`);
      } else {
        reject(new Error(`SSH command failed with code ${code}`));
      }
    });

    // Handle process errors (e.g., command not found, cannot spawn process)
    child.on("error", (err) => {
      reject(new Error(`Failed to execute SSH command: ${err.message}`));
    });
  });
}
function installSshKey(encodedSshKey: string) {
  const decodedKey = Buffer.from(encodedSshKey, "base64").toString("utf8");
  const tempKeyPath = "/tmp/ec2_ssh_key.pem";
  fs.writeFileSync(tempKeyPath, decodedKey, { mode: 0o600 });
  return tempKeyPath;
}
async function establishSshContact(
  ip: String,
  encodedSshKey: string,
) {
  const tempKeyPath = installSshKey(encodedSshKey);
  // Improved SSH connection retry logic
  let attempts = 0;
  const maxAttempts = 60;
  while (attempts < maxAttempts) {
    try {
      execSync(
        `ssh -q -o StrictHostKeyChecking=no -i ${tempKeyPath} -o ConnectTimeout=1 ubuntu@${ip} true`
      );
      core.info(`SSH connection with spot at ${ip} established`);
      return true;
    } catch {
      if (attempts >= maxAttempts - 1) {
        core.error(
          `Timeout: SSH could not connect to ${ip} within 60 seconds.`
        );
        return false;
      }
      await new Promise((resolve) => setTimeout(resolve, 1000)); // Retry every second
      attempts++;
    }
  }
}

async function terminate(instanceStatus?: string, cleanupRunners = true) {
  try {
    core.info("Starting instance cleanup");
    const config = new ActionConfig();
    const ec2Client = new Ec2Instance(config);
    const ghClient = new GithubClient(config);
    const instances = await ec2Client.getInstancesForTags(instanceStatus);
    await ec2Client.terminateInstances(instances.map((i) => i.InstanceId!));
    if (cleanupRunners) {
      core.info("Clearing previously installed runners");
      const result = await ghClient.removeRunnersWithLabels([
        config.githubJobId,
      ]);
      if (result) {
        core.info("Finished runner cleanup");
      } else {
        throw Error(
          "Failed to cleanup runners. Continuing, but failure expected!"
        );
      }
    }
  } catch (error) {
    core.info(error);
  }
}

async function setupGithubRunners(ip: string, config: ActionConfig) {
  const ghClient = new GithubClient(config);
  const githubActionRunnerVersion = await ghClient.getRunnerVersion();
  // Retrieve runner registration tokens in parallel
  const tokens = await Promise.all(
    Array.from({ length: config.githubActionRunnerConcurrency }, () =>
      ghClient.getRunnerRegistrationToken()
    )
  );
  const runnerNameBase = `${config.githubJobId}-ec2`;
  // space-separated registration tokens
  const tokensSpaceSep = tokens.map((t) => t.token).join(" ");
  const bumpShutdown = `sudo shutdown -c ; sudo shutdown -P +${config.ec2InstanceTtl}`;
  // TODO could deregister runners right before shutdown starts
  const setupRunnerCmds = [
    // Shutdown rules:
    // - github actions job starts and ends always bump +ec2InstanceTtl minutes
    // - when the amount of started jobs (start_run_* files) equal the amount of finished jobs (end_run_* files), we shutdown in 5 minutes (with a reaper script installed later)
    "set -x",
    "sudo touch ~/.user-data-started",
    `cd ~`,
    `echo "${bumpShutdown}" > /home/ubuntu/delay_shutdown.sh`,
    "chmod +x /home/ubuntu/delay_shutdown.sh",
    "export ACTIONS_RUNNER_HOOK_JOB_STARTED=/home/ubuntu/delay_shutdown.sh",
    "export ACTIONS_RUNNER_HOOK_JOB_COMPLETED=/home/ubuntu/delay_shutdown.sh",
    "mkdir -p actions-runner && cd actions-runner",
    'echo "ACTIONS_RUNNER_HOOK_JOB_STARTED=/home/ubuntu/delay_shutdown.sh" > .env',
    'echo "ACTIONS_RUNNER_HOOK_JOB_COMPLETED=/home/ubuntu/delay_shutdown.sh" > .env',
    `GH_RUNNER_VERSION=${githubActionRunnerVersion}`,
    'case $(uname -m) in aarch64) ARCH="arm64" ;; amd64|x86_64) ARCH="x64" ;; esac && export RUNNER_ARCH=${ARCH}',
    "curl -O -L https://github.com/actions/runner/releases/download/v${GH_RUNNER_VERSION}/actions-runner-linux-${RUNNER_ARCH}-${GH_RUNNER_VERSION}.tar.gz",
    "tar xzf ./actions-runner-linux-${RUNNER_ARCH}-${GH_RUNNER_VERSION}.tar.gz",
    "mv externals ..", // we share the big binaries between all the runner folders, symlink instead of copy them
    // Note sharing bin doesn't work due to using it as a folder, and we don't bother splitting up sharing bin
    "rm ./actions-runner-linux-${RUNNER_ARCH}-${GH_RUNNER_VERSION}.tar.gz", // cleanup as we will copy our runner folder
    `TOKENS=(${tokensSpaceSep})`,
    `for i in {0..${config.githubActionRunnerConcurrency - 1}}; do`,
    `  cp -r . ../${runnerNameBase}-$i`,
    `  ln -s $(pwd)/../externals ../${runnerNameBase}-$i`,
    `  pushd ../${runnerNameBase}-$i`,
    `  echo \${TOKENS[i]} > .runner-token`,
    `  echo './config.sh $@ && ./run.sh' > config_and_run.sh`,
    `  nohup bash ./config_and_run.sh --unattended --url https://github.com/${github.context.repo.owner}/${github.context.repo.repo} --token \${TOKENS[i]} --labels ${config.githubActionRunnerLabel} --replace --name ${runnerNameBase}-$i 1>/dev/null 2>/dev/null &`,
    `  popd`,
    "done",
    "exit",
  ];
  const tempKeyPath = installSshKey(config.ec2Key);
  await standardSpawn("ssh", ["-o", "StrictHostKeyChecking=no", "-i", tempKeyPath, "-o", "ConnectTimeout=1", `ubuntu@${ip}`, "bash", "-c", setupRunnerCmds.join("\n")]);
}

(async function () {
  try {
    const config = new ActionConfig();
    if (config.githubActionRunnerConcurrency !== 0) {
      for (let i = 0; i < 3; i++) {
        // retry in a loop in case we can't ssh connect after a minute
        if (await startWithGithubRunners(config)) {
          break;
        }
      }
    } else {
      startBareSpot(config);
    }
  } catch (error) {
    terminate();
    assertIsError(error);
    core.error(error);
    core.setFailed(error.message);
  }
})();
