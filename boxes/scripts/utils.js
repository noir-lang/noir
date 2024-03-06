import path from "path";
import os from "os";
import fs from "fs";
import { parse, stringify } from "@iarna/toml";
import { default as axiosBase } from "axios";

const { log, warn, info } = console;
const targetDir = path.join(os.homedir(), ".aztec/bin"); // Use os.homedir() to get $HOME

const { GITHUB_TOKEN } = process.env;
const axiosOpts = {};
if (GITHUB_TOKEN) {
  axiosOpts.headers = { Authorization: `token ${GITHUB_TOKEN}` };
}

export const axios = axiosBase.create(axiosOpts);

export async function getAvailableBoxes(tag, version) {
  const { GITHUB_TOKEN } = process.env;
  const axiosOpts = {};
  if (GITHUB_TOKEN) {
    axiosOpts.headers = { Authorization: `token ${GITHUB_TOKEN}` };
  }

  // TODO: Remove this try catch. Boxes are currently in "boxes" but from this PR on, they will be in "boxes/boxes"
  let data;
  try {
    ({ data } = await axios.get(
      `https://api.github.com/repos/AztecProtocol/aztec-packages/contents/boxes/boxes${tag == "master" ? "" : `?ref=${tag}`}`,
      axiosOpts,
    ));
  } catch (e) {
    if (e.response.statusText === "Not Found") {
      ({ data } = await axios.get(
        `https://api.github.com/repos/AztecProtocol/aztec-packages/contents/boxes${tag == "master" ? "" : `?ref=${tag}`}`,
        axiosOpts,
      ));
    }
  }

  let availableBoxes = data
    .filter(
      (content) => content.type === "dir" && !content.name.startsWith("."),
    )
    .map(async ({ path, name }) => {
      ({ data } = await axios.get(
        `https://raw.githubusercontent.com/AztecProtocol/aztec-packages/${tag == "master" ? "master" : tag}/${path}/package.json`,
        axiosOpts,
      ));

      return {
        name,
        description: data.description || name,
      };
    });

  return await Promise.all(availableBoxes);
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

export function updatePathEnvVar() {
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

export async function replacePaths(rootDir, tag, version) {
  const files = fs.readdirSync(path.resolve(".", rootDir), {
    withFileTypes: true,
  });

  files.forEach((file) => {
    const filePath = path.join(rootDir, file.name);
    if (file.isDirectory()) {
      replacePaths(filePath, tag, version); // Recursively search subdirectories
    } else if (file.name === "Nargo.toml") {
      let content = parse(fs.readFileSync(filePath, "utf8"));

      try {
        Object.keys(content.dependencies).forEach((dep) => {
          const directory = content.dependencies[dep].path.replace(
            /^(..\/)+/,
            "",
          );
          content.dependencies[dep] = {
            git: "https://github.com/AztecProtocol/aztec-packages/",
            tag,
            directory,
          };
        });
      } catch (e) {
        log(e);
      }

      fs.writeFileSync(filePath, prettyPrintNargoToml(content), "utf8");
    } else if (file.name === "package.json") {
      try {
        let content = JSON.parse(fs.readFileSync(filePath, "utf8"));
        Object.keys(content.dependencies)
          .filter((deps) => deps.match("@aztec"))
          // "master" actually means "latest" for the npm release
          .map(
            (dep) =>
              (content.dependencies[dep] =
                `${version === "master" ? "latest" : `^${version}`}`),
          );
        fs.writeFileSync(filePath, JSON.stringify(content), "utf8");
      } catch (e) {
        log("No package.json to update");
      }
    }
  });
}
