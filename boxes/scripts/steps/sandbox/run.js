import confirm from "@inquirer/confirm";
import axios from "axios";

export async function sandboxRunStep() {
  spinner.text = "Trying to reach the sandbox...";

  try {
    spinner.start();
    await axios.post(
      "http://localhost:8080",
      JSON.stringify({
        jsonrpc: "2.0",
        method: "node_getVersion",
        id: "null",
      }),
      {
        headers: {
          Accept: "*/*",
          "Content-Type": "application/json",
        },
      },
    );
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
      start();
    }
  }
}
