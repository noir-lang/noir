const path = require("path");
const fs = require("fs");

const VERSION_IDENTIFIERS = ["noir", "aztec"];

let versions;
async function getVersions() {
  if (!versions) {
    try {
      const noirVersionPath = path.resolve(
        __dirname,
        "../../../yarn-project/noir-compiler/src/noir-version.json"
      );
      const noirVersion = JSON.parse(
        fs.readFileSync(noirVersionPath).toString()
      ).tag;
      const aztecVersionPath = path.resolve(
        __dirname,
        "../../../.release-please-manifest.json"
      );
      const aztecVersion = JSON.parse(
        fs.readFileSync(aztecVersionPath).toString()
      )["."];
      versions = {
        noir: noirVersion,
        aztec: `aztec-packages-v${aztecVersion}`,
      };
    } catch (err) {
      throw new Error(
        `Error loading versions in docusaurus preprocess step.\n${err}`
      );
    }
  }
  return versions;
}

async function preprocessIncludeVersion(markdownContent) {
  const originalContent = markdownContent;
  for (const identifier of VERSION_IDENTIFIERS) {
    const version = (await getVersions())[identifier];
    markdownContent = markdownContent.replaceAll(
      `#include_${identifier}_version`,
      version
    );
  }
  return {
    content: markdownContent,
    isUpdated: originalContent !== markdownContent,
  };
}

module.exports = { preprocessIncludeVersion };
