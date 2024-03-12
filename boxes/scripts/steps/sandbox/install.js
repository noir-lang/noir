import confirm from "@inquirer/confirm";
import { execSync } from "child_process";
import pty from "node-pty";
import { updatePathEnvVar } from "../../utils.js";

const runPty = async (command, { success: exitSuccess, error }) => {
  try {
    const ptySession = new Promise((resolve, reject) => {
      const ptyProcess = pty.spawn("bash", [], {
        name: "xterm-color",
        cols: 80,
        rows: 30,
        cwd: process.cwd(),
        env: process.env,
      });

      ptyProcess.on("data", function (data) {
        process.stdout.write(data);
      });

      ptyProcess.write(command);

      ptyProcess.on("exit", async function (exitCode, signal) {
        await updatePathEnvVar();
        resolve();
        if (exitCode === 0) {
          success(exitSuccess);
        } else {
          error(e);
        }
      });
    });

    await ptySession;
  } catch (e) {
    error(e);
  }
};

function findOutUserVersion() {
  /**
   * We know user has docker installed.
   * Now we get the result of the docker image inspect command
   * If it throws with an empty object, that's because the image doesn't exist so the user
   * Doesn't have the sandbox installed. We exit early since there's nothing to parse.
   *
   * If it returns an object, we parse the COMMIT_TAG field
   * - If there's anything there, that's the version of the sandbox
   * - If there's nothing there, that's because there's no tag yet, so he's on master
   */
  let sandboxVersion = null;
  let dockerOutput = null;
  try {
    dockerOutput = execSync(
      "docker image inspect --format '{{json .Config.Env}}' aztecprotocol/aztec 2>&1",
      {
        encoding: "utf8",
      },
    );
  } catch (error) {
    // Something went wrong with the docker command
    // So we assume sandbox is not installed
    sandboxVersion = null;
  }

  if (!dockerOutput) return sandboxVersion;

  // parsing the docker output to get the commit tag
  sandboxVersion = JSON.parse(dockerOutput)
    .find((env) => env.includes("COMMIT_TAG"))
    .split("=")[1];

  // There's no tag yet, so the user is on master
  if (!sandboxVersion) sandboxVersion = "master";

  return sandboxVersion;
}

export async function sandboxInstallOrUpdate() {
  // Checking for docker
  try {
    execSync("docker info >/dev/null 2>&1");
  } catch (e) {
    error(
      "Doesn't seem like Docker is installed or running. Please start it or visit https://docs.aztec.network for more information",
    );
    process.exit(1);
  }

  // Let's get which version of the sandbox the user has installed
  const sandboxVersion = findOutUserVersion();

  // Base case is that the user doesn't have the sandbox installed
  if (sandboxVersion == null) {
    const answer = await confirm({
      message:
        "Seems like you don't have the Aztec Sandbox installed. Do you want to install it?",
      default: true,
    });

    if (answer) {
      await runPty(
        "echo y | bash -i <(curl -s install.aztec.network); exit\n",
        {
          success: "The Sandbox is installed!",
          error:
            "Failed to install the Sandbox. Please visit the docs at https://docs.aztec.network",
        },
      );
    }
  } else if (
    // Another situation is where the sandbox matches the stable version (i.e. 0.24.0) or master
    (sandboxVersion === latestStable || sandboxVersion === "master") &&
    // but the user has chosen a different version (i.e. "master", 0.23.0, etc)
    sandboxVersion !== version
  ) {
    const answer = await confirm({
      message: `The sandbox is version ${sandboxVersion} but your chosen version is ${version}. Do you want to install version ${version}?`,
      default: true,
    });

    if (answer) {
      // cool thing is that user already has VERSION in the path, so we don't need to pass it here too
      execSync(`$HOME/.aztec/bin/aztec-up`, { stdio: "inherit" });
    }
  } else if (
    // Finally, there's a situation where
    // the user didn't want any specific version
    sandboxVersion !== version &&
    // and the sandbox is not up to date
    // so we need to update to that since the cloned repo is also the latest
    sandboxVersion !== latestStable &&
    // we're also aware that the user might be on master
    // so his version is actually not outdated!
    version !== "master"
  ) {
    const answer = await confirm({
      message: `The Sandbox is not up to date. Do you want to update it to ${latestStable}?`,
      default: true,
    });

    if (answer) {
      // again abusing the fact that the user has VERSION in the path
      execSync(`$HOME/.aztec/bin/aztec-up`, { stdio: "inherit" });
    }
  }
}
