const { match } = require("assert");
const fs = require("fs");
const path = require("path");

const getLineNumberFromIndex = (fileContent, index) => {
  return fileContent.substring(0, index).split("\n").length;
};

/**
 * Search for lines of the form
 */
function processHighlighting(codeSnippet, identifier) {
  const lines = codeSnippet.split("\n");
  /**
   * For an identifier = bar:
   *
   * Matches of the form: `highlight-next-line:foo:bar:baz` will be replaced with "highlight-next-line".
   * Matches of the form: `highlight-next-line:foo:baz` will be replaced with "".
   */
  const regex1 = /highlight-next-line:([a-zA-Z0-9-._:]+)/;
  const replacement1 = "highlight-next-line";
  const regex2 = /highlight-start:([a-zA-Z0-9-._:]+)/;
  const replacement2 = "highlight-start";
  const regex3 = /highlight-end:([a-zA-Z0-9-._:]+)/;
  const replacement3 = "highlight-end";
  const regex4 = /this-will-error:([a-zA-Z0-9-._:]+)/;
  const replacement4 = "this-will-error";

  let result = "";
  let mutated = false;

  const processLine = (line, regex, replacement) => {
    const match = line.match(regex);
    if (match) {
      mutated = true;

      const identifiers = match[1].split(":");
      if (identifiers.includes(identifier)) {
        line = line.replace(match[0], replacement);
      } else {
        // Remove matched text completely
        line = line.replace(match[0], "");
      }
    } else {
      // No match: it's an ordinary line of code.
    }
    return line.trim() == "//" || line.trim() == "#" ? "" : line;
  };

  for (let line of lines) {
    mutated = false;
    line = processLine(line, regex1, replacement1);
    line = processLine(line, regex2, replacement2);
    line = processLine(line, regex3, replacement3);
    line = processLine(line, regex4, replacement4);
    result += line === "" && mutated ? "" : line + "\n";
  }

  return result.trim();
}

/**
 * Parse a code file, looking for identifiers of the form:
 * `docs:start:${identifier}` and `docs:end:{identifier}`.
 * Extract that section of code.
 *
 * It's complicated if code snippet identifiers overlap (i.e. the 'start' of one code snippet is in the
 * middle of another code snippet). The extra logic in this function searches for all identifiers, and
 * removes any which fall within the bounds of the code snippet for this particular `identifier` param.
 * @param {string} filePath
 * @param {string} identifier
 * @returns the code snippet, and start and end line numbers which can later be used for creating a link to github source code.
 */
function extractCodeSnippet(filePath, identifier) {
  let fileContent = fs.readFileSync(filePath, "utf-8");
  let lineRemovalCount = 0;
  let linesToRemove = [];

  const startRegex = /(?:\/\/|#)\s+docs:start:([a-zA-Z0-9-._:]+)/g; // `g` will iterate through the regex.exec loop
  const endRegex = /(?:\/\/|#)\s+docs:end:([a-zA-Z0-9-._:]+)/g;

  /**
   * Search for one of the regex statements in the code file. If it's found, return the line as a string and the line number.
   */
  const lookForMatch = (regex) => {
    let match;
    let matchFound = false;
    let matchedLineNum = null;
    let actualMatch = null;
    let lines = fileContent.split("\n");
    while ((match = regex.exec(fileContent))) {
      if (match !== null) {
        const identifiers = match[1].split(":");
        let tempMatch = identifiers.includes(identifier) ? match : null;

        if (tempMatch === null) {
          // If it's not a match, we'll make a note that we should remove the matched text, because it's from some other identifier and should not appear in the snippet for this identifier.
          for (let i = 0; i < lines.length; i++) {
            let line = lines[i];
            if (line.trim() == match[0].trim()) {
              linesToRemove.push(i + 1); // lines are indexed from 1
              ++lineRemovalCount;
            }
          }
        } else {
          if (matchFound === true) {
            throw new Error(
              `Duplicate for regex ${regex} and identifier ${identifier}`
            );
          }
          matchFound = true;
          matchedLineNum = getLineNumberFromIndex(fileContent, tempMatch.index);
          actualMatch = tempMatch;
        }
      }
    }

    return [actualMatch, matchedLineNum];
  };

  let [startMatch, startLineNum] = lookForMatch(startRegex);
  let [endMatch, endLineNum] = lookForMatch(endRegex);

  // Double-check that the extracted line actually contains the required start and end identifier.
  if (startMatch !== null) {
    const startIdentifiers = startMatch[1].split(":");
    startMatch = startIdentifiers.includes(identifier) ? startMatch : null;
  }
  if (endMatch !== null) {
    const endIdentifiers = endMatch[1].split(":");
    endMatch = endIdentifiers.includes(identifier) ? endMatch : null;
  }

  if (startMatch === null || endMatch === null) {
    if (startMatch === null && endMatch === null) {
      throw new Error(
        `Identifier "${identifier}" not found in file "${filePath}"`
      );
    } else if (startMatch === null) {
      throw new Error(
        `Start line "docs:start:${identifier}" not found in file "${filePath}"`
      );
    } else {
      throw new Error(
        `End line "docs:end:${identifier}" not found in file "${filePath}"`
      );
    }
  }

  let lines = fileContent.split("\n");

  // We only want to remove lines which actually fall within the bounds of our code snippet, so narrow down the list of lines that we actually want to remove.
  linesToRemove = linesToRemove.filter((lineNum) => {
    const removal_in_bounds = lineNum >= startLineNum && lineNum <= endLineNum;
    return removal_in_bounds;
  });

  // Remove lines which contain `docs:` comments for unrelated identifiers:
  lines = lines.filter((l, i) => {
    return !linesToRemove.includes(i + 1); // lines are indexed from 1
  });

  // Remove lines from the snippet which fall outside the `docs:start` and `docs:end` values.
  lines = lines.filter((l, i) => {
    return i + 1 > startLineNum && i + 1 < endLineNum - linesToRemove.length; // lines are indexed from 1
  });

  // We have our code snippet!
  let codeSnippet = lines.join("\n");

  let startCharIndex = startMatch.index;
  let endCharIndex = endMatch.index;

  const startLine = getLineNumberFromIndex(codeSnippet, startCharIndex) + 1;
  const endLine =
    getLineNumberFromIndex(codeSnippet, endCharIndex) -
    1 -
    linesToRemove.length;

  // The code snippet might contain some docusaurus highlighting comments for other identifiers. We should remove those.
  codeSnippet = processHighlighting(codeSnippet, identifier);

  return [codeSnippet, startLine, endLine];
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
        const opts = match[4] || "";

        const noTitle = opts.includes("noTitle");
        const noLineNumbers = opts.includes("noLineNumbers");
        const noSourceLink = opts.includes("noSourceLink");

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

          const title = noTitle ? "" : `title="${identifier}"`;
          const lineNumbers = noLineNumbers ? "" : "showLineNumbers";
          const source = noSourceLink
            ? ""
            : `\n> [<sup><sub>Source code: ${url}</sub></sup>](${url})`;
          const replacement = `\`\`\`${language} ${title} ${lineNumbers} \n${codeSnippet}\n\`\`\`${source}\n`;

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
  const regex =
    /^(?!<!--.*)(?=.*#include_code\s+(\S+)\s+(\S+)\s+(\S+)(?:[ ]+(\S+))?).*$/gm;

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
