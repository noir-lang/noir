#!/usr/bin/env node
import { Command } from "commander";
import select from "@inquirer/select";
import input from "@inquirer/input";
import confirm from "@inquirer/confirm";
const program = new Command();
import tiged from "tiged";
import { exec, execSync } from "child_process";
import pty from "node-pty";
import path from "path";
import os from "os";
import fs from "fs";
import chalk from "chalk";

const { log, warn, info } = console;
const targetDir = path.join(os.homedir(), ".aztec/bin"); // Use os.homedir() to get $HOME

function updatePathEnvVar() {
  // Detect the user's shell profile file based on common shells and environment variables
  const homeDir = os.homedir();
  let shellProfile;
  if (process.env.SHELL?.includes("bash")) {
    shellProfile = path.join(homeDir, ".bashrc");
  } else if (process.env.SHELL?.includes("zsh")) {
    shellProfile = path.join(homeDir, ".zshrc");
  } else {
    // Extend with more conditions for other shells if necessary
    warn("Unsupported shell or shell not detected.");
    return;
  }

  // Read the current content of the shell profile to check if the path is already included
  const profileContent = fs.readFileSync(shellProfile, "utf8");
  if (profileContent.includes(targetDir)) {
    log(`${targetDir} is already in PATH.`);
    return;
  }

  // Append the export command to the shell profile file
  const exportCmd = `\nexport PATH="$PATH:${targetDir}" # Added by Node.js script\n`;
  fs.appendFileSync(shellProfile, exportCmd);

  info(`Added ${targetDir} to PATH in ${shellProfile}.`);
}

program.action(async () => {
  const appType = await select({
    message: "Please choose your boilerplate:",
    choices: [
      { value: "blank", name: "Barebones HTML/TS project" },
      { value: "blank-react", name: "Not-so-barebones React project" },
    ],
  });

  log(chalk.yellow(`You chose: ${appType}`));

  try {
    // STEP 1: Clone the box
    const appName = await input({
      message: "Your app name:",
      default: "my-aztec-app",
    });

    const emitter = tiged(`AztecProtocol/aztec-packages/boxes/${appType}`);

    chalk.blue("Cloning the boilerplate code...");
    emitter.on("info", (info) => {
      log(info.message);
    });

    await emitter.clone(`./${appName}`).then(() => {
      log(chalk.bgGreen("Your code is ready!"));
    });
  } catch (error) {
    log(chalk.bgRed(error.message));
    process.exit(1);
  }

  // STEP 2: Checking for docker
  try {
    execSync("docker info >/dev/null 2>&1");
  } catch (error) {
    log(
      chalk.bgRed(
        "Doesn't seem like Docker is installed. Please visit https://docs.aztec.network",
      ),
    );
    process.exit(1);
  }

  // STEP 2: Checking for the Aztec Sandbox
  try {
    execSync("docker image inspect aztecprotocol/aztec > /dev/null 2>&1");
  } catch (error) {
    const answer = await confirm({
      message:
        "Seems like you don't have the Aztec Sandbox installed. Do you want to install it?",
      default: true,
    });

    if (answer) {
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

          ptyProcess.write(
            "echo y | bash -i <(curl -s install.aztec.network); exit\n",
          );

          ptyProcess.on("exit", function (exitCode, signal) {
            updatePathEnvVar();
            resolve();
            log(chalk.bgGreen("The Sandbox is installed!"));
          });
        });

        await ptySession;
      } catch (error) {
        log(
          chalk.bgRed(
            "Failed to install the Sandbox. Please visit the docs at https://docs.aztec.network",
          ),
        );
      }
    }
  }

  // STEP 2: Running the Sandbox
  try {
    await fetch("http://localhost:8080", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        jsonrpc: "2.0",
        method: "node_getVersion",
        id: "null",
      }),
    });
  } catch (error) {
    const answer = await confirm({
      message:
        "I can't reach the Sandbox on port 8080. Do you want to start it?",
      default: true,
    });

    if (answer) {
      log(
        chalk.green("Starting the sandbox... This might take a few minutes."),
      );
      log(chalk.bgGreen(`Go and explore the boilerplate code while you wait!`));
      execSync(`$HOME/.aztec/bin/aztec sandbox`, { stdio: "inherit" });
    }
  }
});

program.parse();
