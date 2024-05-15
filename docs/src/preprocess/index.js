const fs = require("fs");
const path = require("path");
const childProcess = require("child_process");

const { preprocessIncludeCode } = require("./include_code");
const { preprocessIncludeVersion } = require("./include_version");

const { generateInstructionSet } = require("./InstructionSet/genMarkdown");

async function processMarkdownFilesInDir(rootDir, docsDir, regex) {
  const files = fs.readdirSync(docsDir);
  const contentUpdates = [];

  for (const file of files) {
    const filepath = path.join(docsDir, file);
    const stat = fs.statSync(filepath);

    if (stat.isDirectory()) {
      contentUpdates.push(processMarkdownFilesInDir(rootDir, filepath, regex));
    } else if (
      stat.isFile() &&
      (file.endsWith(".md") || file.endsWith(".mdx") || file.endsWith(".json"))
    ) {
      const markdownContent = fs.readFileSync(filepath, "utf-8");

      let updatedContent = markdownContent;
      let isUpdated = false;
      for (preprocess of [preprocessIncludeCode, preprocessIncludeVersion]) {
        const result = await preprocess(updatedContent, filepath, rootDir);
        updatedContent = result.content;
        isUpdated = isUpdated || result.isUpdated;
      }

      contentUpdates.push({
        content: updatedContent,
        filepath,
        isUpdated,
      });
    }
  }

  return Promise.all(contentUpdates);
}

async function writeProcessedFiles(docsDir, destDir, cachedDestDir, content) {
  let writePromises = [];

  if (Array.isArray(content)) {
    // It's a dir
    if (content.length > 0) {
      // It's a nonempty dir
      writePromises.push(
        await Promise.all(
          content.map((a) =>
            writeProcessedFiles(docsDir, destDir, cachedDestDir, a)
          )
        )
      );
    } else {
      // empty dir
    }
  } else if (!content.filepath) {
    // Do nothing
  } else {
    // It's a file
    // Derive the destination path from the original path:
    const relPath = path.relative(docsDir, content.filepath);
    const destFilePath = path.resolve(destDir, relPath);
    const destDirName = path.dirname(destFilePath);
    const cachedDestFilePath = path.resolve(cachedDestDir, relPath);
    const cachedDestDirName = path.dirname(cachedDestFilePath);

    if (!fs.existsSync(destDirName)) {
      fs.mkdirSync(destDirName, { recursive: true });
    }
    if (!fs.existsSync(cachedDestDirName)) {
      fs.mkdirSync(cachedDestDirName, { recursive: true });
    }

    // If the file exists, don't overwrite unless we need to:
    if (fs.existsSync(destFilePath)) {
      const existingFileContent = fs.readFileSync(destFilePath, "utf-8");

      // Safety: try and check whether the dev has been making code edits in the wrong dir!
      if (fs.existsSync(cachedDestFilePath)) {
        const cachedFileContent = fs.readFileSync(cachedDestFilePath, "utf-8");
        if (existingFileContent !== cachedFileContent) {
          throw new Error(
            `It looks like you might have accidentally edited files in the 'processed-docs/' dir instead of the 'docs/' dir (because there's a discrepancy between 'preprocessed-docs' and 'preprocessed-docs-cache', but they should always be the same unless they're tampered-with).\n\nWe don't want you to accidentally overwrite your work.\n\nCopy your work to the 'docs/' dir, and revert your 'processed-docs/' changes.\n\nI.e. copy from here: ${destFilePath}\n\nto here: ${content.filepath}\n\nIf this error's safety assumption is wrong, and you'd like to proceed with building, please delete the cached file ${cachedDestFilePath} and rerun the build.\n\nAnd if you've not made any changes at all to the docs and you've just pulled master and are wondering what is going on, you might want to run \`yarn clear\` from this docs dir.`
          );
        }
      }

      // Don't write if no change.
      if (existingFileContent === content.content) {
        // Do nothing: the content doesn't need to be overwritten.
        // This will speed up the docusaurus build.
        return;
      }
    }

    writePromises.push(
      fs.promises.writeFile(destFilePath, content.content, {
        encoding: "utf8",
        flag: "w", // overwrite
      })
    );

    // Cache the dest data as well, as a safety measure, to ensure no one edits the processed-docs instead of the docs, by mistake.
    writePromises.push(
      fs.promises.writeFile(cachedDestFilePath, content.content, {
        encoding: "utf8",
        flag: "w", // overwrite
      })
    );
  }

  return Promise.all(writePromises);
}

async function run() {
  await generateInstructionSet();

  const rootDir = path.join(__dirname, "../../../");
  const docsDir = path.join(rootDir, "docs", "docs");
  const destDir = path.join(rootDir, "docs", "processed-docs");
  const cachedDestDir = path.join(rootDir, "docs", "processed-docs-cache");

  const content = await processMarkdownFilesInDir(rootDir, docsDir);

  await writeProcessedFiles(docsDir, destDir, cachedDestDir, content);

  console.log("Preprocessing complete.");
}

/**
 * Parses all .md and .mdx files in docs/ for lines of the form:
 *   #include_code snippet_identifier /circuits/my_code.cpp cpp
 *
 * Reads the code file and extracts the code snippet bookended by `docs:start:snippet_identifier` and `docs:end:snippet_identifier.
 *
 * Replaces the `#include_code` line with the code snippet (in memory).
 *
 * Writes the updated .md or .mdx file to a `processed-docs/` dir.
 *
 * docusaurus then can build from the `processed-docs/` dir.
 */
run();
