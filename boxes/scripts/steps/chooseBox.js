import select from "@inquirer/select";
import {
  getAvailableBoxes,
  getAvailableContracts,
  processProject,
  replacePaths,
  clone,
} from "../utils.js";
import { execSync } from "child_process";
import { getPlaceholders } from "../config.js";

async function initGit({ dir }) {
  execSync(`yes | git -C ${dir} init`);
}

async function chooseAndCloneBox({ projectName }) {
  // if the user has already chosen a project name, we should skip the input
  // and use the one they've chosen
  const skipName = projectName || undefined;

  const availableBoxes = await getAvailableBoxes();
  if (!projectName) {
    projectName = await select({
      message: `Please choose your Aztec boilerplate:`,
      choices: [
        ...availableBoxes.map((box) => {
          return { value: box.name, name: box.description };
        }),
        { value: "skip", name: "Skip this step" },
      ],
    });
  } else {
    if (!availableBoxes.find((box) => box.name === projectName)) {
      throw new Error(`Box ${projectName} not found`);
    }
  }

  const rootDir = await clone({
    path: "boxes/boxes",
    choice: projectName,
    type: "box",
    tag,
    version,
    name: skipName,
  });

  await initGit({ dir: rootDir });

  await replacePaths({
    rootDir,
    tag,
    version,
    prefix: "",
  });
  success("Your code is ready!");
}

async function chooseAndCloneContract({ projectName }) {
  const availableContracts = await getAvailableContracts();

  // if the user has already chosen a project name, we should skip the input
  // and use the one they've chosen
  const skipName = projectName || undefined;

  if (!projectName) {
    projectName = await select({
      message: `Please choose your Aztec boilerplate:`,
      choices: [
        ...availableContracts.map((contract) => {
          return { value: contract.name, name: contract.name };
        }),
        { value: "skip", name: "Skip this step" },
      ],
    });
  } else {
    if (!availableContracts.find((contract) => contract.name === projectName)) {
      throw new Error(`Contract ${projectName} not found`);
    }
  }

  const rootDir = await clone({
    path: "noir-projects/noir-contracts/contracts",
    choice: projectName,
    type: "contract",
    tag,
    version,
    name: skipName,
  });

  await initGit({ dir: rootDir });

  await replacePaths({
    rootDir,
    tag,
    version,
    prefix: "noir-projects/",
  });

  await processProject({
    rootDir,
    placeholders: getPlaceholders(projectName),
  });
  success("Your code is ready!");

  // get the e2e test for that contract from yarn-project/end-to-end
}

export async function chooseProject({ projectType, projectName }) {
  if (!projectType) {
    projectType = await select({
      message: `Please choose your type of project:`,
      choices: [
        { value: "app", name: "Boilerplate project with frontend" },
        { value: "contract", name: "Just a contract example" },
        { value: "skip", name: "Skip this step" },
      ],
    });
  }

  if (projectType === "skip") {
    return;
  } else if (projectType === "contract") {
    await chooseAndCloneContract({ projectName: projectName });
  } else if (projectType === "app") {
    await chooseAndCloneBox({ projectName: projectName });
  }
}
