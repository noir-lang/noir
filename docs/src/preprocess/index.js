const fs = require("fs");
const path = require("path");

const getLineNumberFromIndex = (fileContent, index) => {
  return fileContent.substr(0, index).split("\n").length;
};

function extractCodeSnippet(filePath, identifier) {
  const fileContent = fs.readFileSync(filePath, "utf-8");

  const startTag = `// docs:start:${identifier}`;
  const endTag = `// docs:end:${identifier}`;
  const startIndex = fileContent.indexOf(startTag);
  const endIndex = fileContent.indexOf(endTag);

  if (startIndex === -1 || endIndex === -1) {
    if (startIndex === -1 && endIndex === -1) {
      throw new Error(
        `Identifier "${identifier}" not found in file "${filePath}"`
      );
    } else if (startIndex === -1) {
      throw new Error(
        `Start line "docs:start:${identifier}" not found in file "${filePath}"`
      );
    } else {
      throw new Error(
        `End line "docs:end:${identifier}" not found in file "${filePath}"`
      );
    }
  }

  const slicedContent = fileContent
    .slice(startIndex + startTag.length, endIndex)
    .trim();

  const startLine = getLineNumberFromIndex(fileContent, startIndex) + 1;
  const endLine = getLineNumberFromIndex(fileContent, endIndex) - 1;

  return [slicedContent, startLine, endLine];
}

async function processMarkdownFilesInDir(rootDir, docsDir, regex) {
  const files = fs.readdirSync(docsDir);
  const contentPromises = [];

  for (const file of files) {
    const filePath = path.join(docsDir, file);
    const stat = fs.statSync(filePath);

    if (stat.isDirectory()) {
      contentPromises.push(processMarkdownFilesInDir(rootDir, filePath, regex));
    } else if (
      stat.isFile() &&
      (file.endsWith(".md") || file.endsWith(".mdx"))
    ) {
      const markdownContent = fs.readFileSync(filePath, "utf-8");

      // Process each include tag in the current markdown file
      let updatedContent = markdownContent;
      let matchesFound = false;
      let match;
      while ((match = regex.exec(markdownContent))) {
        matchesFound = true;
        const fullMatch = match[0];
        const identifier = match[1];
        const codeFilePath = match[2]; // Absolute path to the code file from the root of the Docusaurus project
        const language = match[3];

        try {
          const absoluteCodeFilePath = path.join(rootDir, codeFilePath);

          // Extract the code snippet between the specified comments
          const [codeSnippet, startLine, endLine] = extractCodeSnippet(
            absoluteCodeFilePath,
            identifier
          );

          const url = `https://github.com/AztecProtocol/aztec-packages/blob/master/${path.resolve(
            rootDir,
            codeFilePath
          )}#L${startLine}-L${endLine}`;

          const replacement = `\`\`\`${language} title=${identifier} showLineNumbers \n${codeSnippet}\n\`\`\`\n> [Link to source code.](${url})\n`;

          // Replace the include tag with the code snippet
          updatedContent = updatedContent.replace(fullMatch, replacement);
        } catch (error) {
          const lineNum = getLineNumberFromIndex(markdownContent, match.index);
          let wrapped_msg = `Error processing "${filePath}:${lineNum}": ${error.message}.`;

          // Let's just output a warning, so we don't ruin our development experience.
          // throw new Error(wrapped_msg);
          console.warn(
            "\n\x1b[33m%s\x1b[0m%s",
            "[WARNING] ",
            wrapped_msg,
            "\n"
          );
        }
      }

      contentPromises.push({
        filepath: filePath,
        content: updatedContent,
        isUpdated: matchesFound,
      });
    }
  }

  const contentArray = await Promise.all(contentPromises);

  return contentArray;
}

async function writeProcessedFiles(docsDir, destDir, cachedDestDir, content) {
  let writePromises = [];

  // if (!Array.isArray(content)) throw new Error("NOT AN ARRAY!!!!");

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
            `It looks like you might have accidentally edited files in the 'processed-docs/' dir instead of the 'docs/' dir (because there's a discrepancy between 'preprocessed-docs' and 'preprocessed-docs-cache', but they should always be the same unless they're tampered-with).\n\nWe don't want you to accidentally overwrite your work.\n\nCopy your work to the 'docs/' dir, and revert your 'processed-docs/' changes.\n\nI.e. copy from here: ${destFilePath}\n\nto here: ${content.filepath}\n\nIf this error's safety assumption is wrong, and you'd like to proceed with building, please delete the cached file ${cachedDestFilePath} and rerun the build.\n\n`
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

  return writePromises;
}

async function run() {
  const rootDir = path.join(__dirname, "../../../");
  const docsDir = path.join(rootDir, "docs", "docs");
  const destDir = path.join(rootDir, "docs", "processed-docs");
  const cachedDestDir = path.join(rootDir, "docs", "processed-docs-cache");

  /**
   * Explaining this regex:
   *
   * E.g. `#include_code snippet_identifier /circuits/my_code.cpp cpp`
   *
   * #include_code\s+(\S+)\s+(\S+)\s+(\S+)
   *   - This is the main regex to match the above format.
   *   - \s+: one or more whitespace characters (space or tab) after `include_code` command.
   *   - (\S+): one or more non-whitespaced characters. Captures this as the first argument, which is a human-readable identifier for the code block.
   *   - etc.
   *
   * Lookaheads are needed to allow us to ignore commented-out lines:
   *
   * ^(?!<!--.*)
   *   - ^: Asserts the beginning of the line.
   *   - (?!<!--.*): Negative lookahead assertion to ensure the line does not start with markdown comment syntax `<!--`.
   *
   * (?=.*STUFF)
   *   - Positive lookahead assertion to ensure the line contains the command (STUFF) we want to match.
   *
   * .*$
   *   - .*: Matches any characters (except newline) in the line.
   *   - $: Asserts the end of the line.
   *
   * `/gm`
   *   - match globally (g) across the entire input text and consider multiple lines (m) when matching. This is necessary to handle multiple include tags throughout the markdown content.
   */
  const regex = /^(?!<!--.*)(?=.*#include_code\s+(\S+)\s+(\S+)\s+(\S+)).*$/gm;

  const content = await processMarkdownFilesInDir(rootDir, docsDir, regex);

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
