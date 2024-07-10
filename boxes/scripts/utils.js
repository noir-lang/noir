import path from "path";
import os from "os";
import fs from "fs/promises";
import { parse, stringify } from "@iarna/toml";
import { CONTRACTS_TO_SHOW, AZTEC_REPO } from "./config.js";

import input from "@inquirer/input";
import tiged from "tiged";

const targetDir = path.join(os.homedir(), ".aztec/bin"); // Use os.homedir() to get $HOME

export async function getAvailableBoxes() {
  try {
    const data = await github({
      path: `boxes/boxes${tag == "master" ? "" : `?ref=${tag}`}`,
    });

    let availableBoxes = data
      .filter(
        (content) => content.type === "dir" && !content.name.startsWith("."),
      )
      .map(async ({ path, name }) => {
        const { description } = await github({
          path: `${tag == "master" ? "master" : tag}/${path}/package.json`,
          raw: true,
        });

        return {
          name,
          description: description || name,
        };
      });

    return await Promise.all(availableBoxes);
  } catch (e) {
    error(e);
  }
}

export async function getAvailableContracts() {
  try {
    const data = await github({
      path: `noir-projects/noir-contracts/contracts${tag == "master" ? "" : `?ref=${tag}`}`,
    });
    let availableContracts = data.filter((content) =>
      CONTRACTS_TO_SHOW.includes(content.name),
    );

    return await Promise.all(availableContracts);
  } catch (e) {
    error(e);
  }
}

export async function clone({ path, choice, type, name }) {
  if (!name) {
    name = await input({
      message: `Your ${type} name:`,
      default: `my-aztec-${type}`,
    });
  }

  spinner.text = `Cloning the ${type} code...`;
  try {
    spinner.start();

    const emitter = tiged(
      `${AZTEC_REPO}/${path}/${choice}${tag && `#${tag}`}`,
      { verbose: true },
    );
    emitter.on("info", ({ message }) => debug(message));
    emitter.on("warn", ({ message }) => error(message));
    await emitter.clone(`./${name}`);

    if (type === "contract") {
      spinner.text = `Cloning default contract project...`;
      const baseEmitter = tiged(
        `${AZTEC_REPO}/boxes/contract-only${tag && `#${tag}`}`,
        { verbose: true },
      );
      baseEmitter.on("info", debug);
      baseEmitter.on("warn", error);
      await baseEmitter.clone(`./${name}/base`);
      await fs.cp(`./${name}/base`, `./${name}`, {
        recursive: true,
        force: true,
      });
      await fs.rm(`./${name}/base`, { recursive: true, force: true });
    }
    spinner.succeed();
    return `./${name}`;
  } catch (e) {
    spinner.fail();
    error(e);
    process.exit(1);
  }
}

export async function processProject({ rootDir, placeholders }) {
  spinner.text = `Processing the code...`;
  try {
    spinner.start();
    const processes = [];
    const findAndReplace = async (dir, placeholders) => {
      const files = await fs.readdir(dir, {
        withFileTypes: true,
      });
      files.forEach(async (file) => {
        const filePath = path.join(dir, file.name);
        if (file.isDirectory()) {
          findAndReplace(filePath, placeholders);
        } else {
          processes.push(
            new Promise(async (resolve, reject) => {
              let content = await fs.readFile(filePath, "utf8");
              placeholders.forEach(({ key, value }) => {
                content = content.replace(new RegExp(key, "g"), value);
              });
              await fs.writeFile(filePath, content, "utf8");

              resolve();
            }),
          );
        }
      });
    };

    await findAndReplace(path.resolve(rootDir), placeholders);
    await Promise.all(processes);
    spinner.succeed();
  } catch (e) {
    spinner.fail();
    error(e);
    process.exit(1);
  }
}

export function prettyPrintNargoToml(config) {
  const withoutDependencies = Object.fromEntries(
    Object.entries(config).filter(([key]) => key !== "dependencies"),
  );

  const partialToml = stringify(withoutDependencies);
  const dependenciesToml = Object.entries(config.dependencies).map(
    ([name, dep]) => {
      const depToml = stringify.value(dep);
      return `${name} = ${depToml}`;
    },
  );

  return (
    partialToml + "\n[dependencies]\n" + dependenciesToml.join("\n") + "\n"
  );
}

export async function updatePathEnvVar() {
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
  const profileContent = await fs.readFile(shellProfile, "utf8");
  if (profileContent.includes(targetDir)) {
    info(`${targetDir} is already in PATH.`);
    return;
  }

  // Append the export command to the shell profile file
  const exportCmd = `\nexport PATH="$PATH:${targetDir}" # Added by Node.js script\n`;
  await fs.appendFile(shellProfile, exportCmd);

  info(`Added ${targetDir} to PATH in ${shellProfile}.`);
}

export async function replacePaths({ rootDir, prefix = "" }) {
  spinner.text = `Replacing paths...`;

  try {
    spinner.start();
    const replaces = [];
    const findAndReplace = async (dir, prefix) => {
      const files = await fs.readdir(dir, {
        withFileTypes: true,
      });
      files.forEach(async (file) => {
        const filePath = path.join(dir, file.name);
        if (file.isDirectory()) {
          findAndReplace(filePath, prefix); // Recursively search subdirectories
        } else if (file.name === "Nargo.toml") {
          replaces.push(
            new Promise(async (resolve, reject) => {
              let content = parse(await fs.readFile(filePath, "utf8"));
              if (!content.dependencies) return;
              Object.keys(content.dependencies).forEach((dep) => {
                const directory = content.dependencies[dep].path.replace(
                  /^(..\/)+/,
                  "",
                );
                content.dependencies[dep] = {
                  git: `https://github.com/${AZTEC_REPO}/`,
                  tag,
                  directory: `${prefix}${directory}`,
                };
              });

              await fs.writeFile(
                filePath,
                prettyPrintNargoToml(content),
                "utf8",
              );
              resolve();
            }),
          );
        } else if (file.name === "package.json") {
          replaces.push(
            new Promise(async (resolve, reject) => {
              let content = JSON.parse(await fs.readFile(filePath, "utf8"));
              if (!content.dependencies) return;
              Object.keys(content.dependencies)
                .filter((deps) => deps.match("@aztec"))
                // "master" actually means "latest" for the npm release
                .map(
                  (dep) =>
                    (content.dependencies[dep] =
                      `${version === "master" ? "latest" : `^${version}`}`),
                );
              await fs.writeFile(filePath, JSON.stringify(content), "utf8");
              resolve();
            }),
          );
        }
      });
    };

    await findAndReplace(path.resolve(rootDir), prefix);
    await Promise.all(replaces);
    spinner.succeed();
    return;
  } catch (e) {
    spinner.fail();
    error(e);
    process.exit(1);
  }
}
