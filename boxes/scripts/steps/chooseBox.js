import select from "@inquirer/select";
import input from "@inquirer/input";
import tiged from "tiged";
import { getAvailableBoxes, replacePaths } from "../utils.js";
import chalk from "chalk";
import ora from "ora";
const { log } = console;

export async function chooseAndCloneBox(tag, version) {
  const availableBoxes = await getAvailableBoxes(tag, version);
  const appType = await select({
    message: `Please choose your Aztec boilerplate:`,
    choices: [
      ...availableBoxes.map((box) => {
        return { value: box.name, name: box.description };
      }),
      { value: "skip", name: "Skip this step" },
    ],
  });

  if (appType === "skip") return;

  log(chalk.yellow(`You chose: ${appType}`));

  const spinner = ora({
    text: "Cloning the boilerplate code...",
    color: "blue",
  });

  try {
    // STEP 1: Clone the box
    const appName = await input({
      message: "Your app name:",
      default: "my-aztec-app",
    });

    spinner.start();

    const emitter = tiged(
      // same as the nargo dependencies above:
      // but if the user has set a semver version, we want that tag (i.e. aztec-packages-v0.23.0)
      `AztecProtocol/aztec-packages/boxes/${appType}${tag && `#${tag}`}`,
      {
        verbose: true,
      },
    );

    emitter.on("info", (info) => {
      log(info.message);
    });

    await emitter.clone(`./${appName}`).then(() => {
      replacePaths(`./${appName}`, tag, version);
      log(chalk.bgGreen("Your code is ready!"));
    });
  } catch (error) {
    log(chalk.bgRed(error.message));
    process.exit(1);
  } finally {
    spinner.stop();
  }
}
