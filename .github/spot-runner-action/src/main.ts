import * as core from "@actions/core";
import { ActionConfig } from "./config";
import { Ec2Instance } from "./ec2";
import { GithubClient } from "./github";
import { assertIsError } from "./utils";

async function pollSpotStatus(
  config: ActionConfig,
  ec2Client: Ec2Instance,
  ghClient: GithubClient
): Promise<"usable" | "unusable" | "none"> {
  // 12 iters x 10000 ms = 2 minutes
  for (let iter = 0; iter < 12; iter++) {
    const instances = await ec2Client.getInstancesForTags();
    const hasInstance =
      instances.filter((i) => i.State?.Name === "running").length > 0;
    if (!hasInstance) {
      // we need to start an instance
      return "none";
    }
    try {
      core.info("Found ec2 instance, looking for runners.");
      if (await ghClient.hasRunner([config.githubJobId])) {
        // we have runners
        return "usable";
      }
    } catch (err) {}
    // wait 10 seconds
    await new Promise((r) => setTimeout(r, 10000));
  }
  // we have a bad state for a while, error
  core.warning(
    "Looped for 2 minutes and could only find spot with no runners!"
  );
  return "unusable";
}

async function start() {
  const config = new ActionConfig();
  if (config.subaction === "stop") {
    await stop();
    return;
  } else if (config.subaction === "restart") {
    await stop();
    // then we make a fresh instance
  } else if (config.subaction !== "start") {
    throw new Error("Unexpected subaction: " + config.subaction);
  }
  // subaction is 'start' or 'restart'estart'
  const ec2Client = new Ec2Instance(config);
  const ghClient = new GithubClient(config);
  const spotStatus = await pollSpotStatus(config, ec2Client, ghClient);
  if (spotStatus === "usable") {
    core.info(
      `Runner already running. Continuing as we can target it with jobs.`
    );
    return;
  }
  if (spotStatus === "unusable") {
    core.warning(
      "Taking down spot as it has no runners! If we were mistaken, this could impact existing jobs."
    );
    if (config.subaction === "restart") {
      throw new Error(
        "Taking down spot we just started. This seems wrong, erroring out."
      );
    }
    await stop();
  }

  var ec2SpotStrategies: string[];
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

  var instanceId = "";
  for (const ec2Strategy of ec2SpotStrategies) {
    core.info(`Starting instance with ${ec2Strategy} strategy`);
    // 6 * 10000ms = 1 minute per strategy
    // TODO make longer lived spot request?
    for (let i = 0; i < 6; i++) {
      try {
        // Start instance
        instanceId = await ec2Client.requestMachine(
          // we fallback to on-demand
          ec2Strategy.toLocaleLowerCase() === "none"
        ) || "";
        if (instanceId) {
          break;
        }
        // let's exit, only loop on InsufficientInstanceCapacity
        break;
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
      await new Promise((r) => setTimeout(r, 10000));
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
  if (instanceId) await ghClient.pollForRunnerCreation([config.githubJobId]);
  else {
    core.error("Instance failed to register with Github Actions");
    throw Error("Instance failed to register with Github Actions");
  }
}

async function stop() {
  try {
    core.info("Starting instance cleanup");
    const config = new ActionConfig();
    const ec2Client = new Ec2Instance(config);
    const ghClient = new GithubClient(config);
    const instances = await ec2Client.getInstancesForTags();
    await ec2Client.terminateInstances(instances.map((i) => i.InstanceId!));
    core.info("Clearing previously installed runners");
    const result = await ghClient.removeRunnersWithLabels([config.githubJobId]);
    if (result) {
      core.info("Finished runner cleanup");
    } else {
      throw Error(
        "Failed to cleanup runners. Continuing, but failure expected!"
      );
    }
  } catch (error) {
    core.info(error);
  }
}

(async function () {
  try {
    start();
  } catch (error) {
    stop();
    assertIsError(error);
    core.error(error);
    core.setFailed(error.message);
  }
})();
