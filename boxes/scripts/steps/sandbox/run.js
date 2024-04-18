import confirm from "@inquirer/confirm";
import { execSync } from "child_process";
import axios from "axios";

export async function sandboxRun() {
  spinner.text = "Trying to reach the sandbox...";

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
    spinner.succeed();
    success("The Sandbox is already running!");
    process.exit(0);
  } catch (error) {
    spinner.fail();
    const answer = await confirm({
      message:
        "Sandbox can't be reached on localhost:8080. Do you want to start it?",
      default: true,
    });
    if (answer) {
      info("Starting the sandbox... This might take a few minutes.");
      info(`Go and explore the boilerplate code while you wait!`);
      execSync(`$HOME/.aztec/bin/aztec sandbox`, {
        stdio: "inherit",
      });
    }
  }
}
