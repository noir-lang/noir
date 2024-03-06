import confirm from "@inquirer/confirm";
import { execSync } from "child_process";
import chalk from "chalk";
import axios from "axios";
import ora from "ora";
const { log } = console;

export async function sandboxRun(version) {
  const spinner = ora({
    text: "Trying to reach the sandbox...",
    color: "blue",
  });

  try {
    spinner.start();
    await axios("http://localhost:8080", {
      method: "POST",
      timeout: 2000,
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        jsonrpc: "2.0",
        method: "node_getVersion",
        id: "null",
      }),
    });
    spinner.stop();
    log(chalk.green("The Sandbox already running!"));
  } catch (error) {
    spinner.stop();
    const answer = await confirm({
      message:
        "Sandbox can't be reached on localhost:8080. Do you want to start it?",
      default: true,
    });

    if (answer) {
      log(
        chalk.green("Starting the sandbox... This might take a few minutes."),
      );
      log(chalk.bgGreen(`Go and explore the boilerplate code while you wait!`));
      execSync(`$HOME/.aztec/bin/aztec sandbox`, { stdio: "inherit" });
    }
  } finally {
    spinner.stop();
  }
}
