import select from "@inquirer/select";
import {
  getAvailableBoxes,
  getAvailableContracts,
  processProject,
  replacePaths,
  clone,
} from "../utils.js";
import { getPlaceholders } from "../config.js";

async function chooseAndCloneBox() {
  const availableBoxes = await getAvailableBoxes();
  const appType = await select({
    message: `Please choose your Aztec boilerplate:`,
    choices: [
      ...availableBoxes.map((box) => {
        return { value: box.name, name: box.description };
      }),
      { value: "skip", name: "Skip this step" },
    ],
  });

  const rootDir = await clone({
    path: "boxes/boxes",
    choice: appType,
    type: "box",
    tag,
    version,
  });

  await replacePaths({
    rootDir,
    tag,
    version,
    prefix: "",
  });
  success("Your code is ready!");
}

async function chooseAndCloneContract() {
  const availableContracts = await getAvailableContracts();
  // let user choose one of the contracts in noir-projects
  const contract = await select({
    message: `Please choose your Aztec boilerplate:`,
    choices: [
      ...availableContracts.map((contract) => {
        return { value: contract.name, name: contract.name };
      }),
      { value: "skip", name: "Skip this step" },
    ],
  });

  const rootDir = await clone({
    path: "noir-projects/noir-contracts/contracts",
    choice: contract,
    type: "contract",
    tag,
    version,
  });

  await replacePaths({
    rootDir,
    tag,
    version,
    prefix: "noir-projects/",
  });

  await processProject({
    rootDir,
    placeholders: getPlaceholders(contract),
  });
  success("Your code is ready!");

  // get the e2e test for that contract from yarn-project/end-to-end
}

export async function chooseProject() {
  const projectType = await select({
    message: `Please choose your type of project:`,
    choices: [
      { value: "fs_app", name: "Boilerplate project with frontend" },
      { value: "contract_only", name: "Just a contract example" },
      { value: "skip", name: "Skip this step" },
    ],
  });

  if (projectType === "skip") {
    return;
  } else if (projectType === "contract_only") {
    await chooseAndCloneContract();
  } else if (projectType === "fs_app") {
    await chooseAndCloneBox();
  }
}
