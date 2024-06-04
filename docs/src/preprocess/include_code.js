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

  const countLeadingSpaces = (line) => {
    const match = line.match(/^ */);
    return match ? match[0].length : 0;
  };
  let indention = 200;
  let resultLines = [];

  for (let line of lines) {
    mutated = false;
    line = processLine(line, regex1, replacement1);
    line = processLine(line, regex2, replacement2);
    line = processLine(line, regex3, replacement3);
    line = processLine(line, regex4, replacement4);

    if (!(line === "" && mutated)) {
      resultLines.push(line);

      const leadingSpaces = countLeadingSpaces(line);
      if (line.length > 0 && leadingSpaces < indention) {
        indention = leadingSpaces;
      }
    }
  }

  let result = "";
  for (let line of resultLines) {
    result +=
      (line.length > indention ? line.substring(indention) : line).trimEnd() +
      "\n";
  }
  return result.trimEnd();
}

/**
 * Parse a code file, looking for identifiers of the form:
 * `docs:start:${identifier}` and `docs:end:{identifier}`.
 * Extract that section of code.
 *
 * It's complicated if code snippet identifiers overlap (i.e. the 'start' of one code snippet is in the
 * middle of another code snippet). The extra logic in this function searches for all identifiers, and
 * removes any which fall within the bounds of the code snippet for this particular `identifier` param.
 * @returns the code snippet, and start and end line numbers which can later be used for creating a link to github source code.
 */
function extractCodeSnippet(filePath, identifier) {
  let fileContent = fs.readFileSync(filePath, "utf-8");
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

  // The code snippet might contain some docusaurus highlighting comments for other identifiers. We should remove those.
  codeSnippet = processHighlighting(codeSnippet, identifier);

  return [codeSnippet, startLineNum, endLineNum];
}

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

async function preprocessIncludeCode(markdownContent, filePath, rootDir) {
  // Process each include tag in the current markdown file
  let updatedContent = markdownContent;
  let matchesFound = false;
  let match;
  while ((match = regex.exec(markdownContent))) {
    matchesFound = true;
    const fullMatch = match[0];
    const identifier = match[1];
    let codeFilePath = match[2];
    const language = match[3];
    const opts = match[4] || "";

    if (codeFilePath.slice(0) != "/") {
      // Absolute path to the code file from the root of the Docusaurus project
      // Note: without prefixing with `/`, the later call to `path.resolve()` gives an incorrect path (absolute instead of relative)
      codeFilePath = `/${codeFilePath}`;
    }

    const noTitle = opts.includes("noTitle");
    const noLineNumbers = opts.includes("noLineNumbers");
    const noSourceLink = opts.includes("noSourceLink");

    try {
      const absCodeFilePath = path.join(rootDir, codeFilePath);

      // Extract the code snippet between the specified comments
      const [codeSnippet, startLine, endLine] = extractCodeSnippet(
        absCodeFilePath,
        identifier,
        filePath
      );

      const relativeCodeFilePath = path
        .resolve(rootDir, codeFilePath)
        .replace(/^\//, "");
      const urlText = `${relativeCodeFilePath}#L${startLine}-L${endLine}`;
      const tag = process.env.COMMIT_TAG
        ? `${process.env.COMMIT_TAG}`
        : "master";
      const url = `https://github.com/AztecProtocol/aztec-packages/blob/${tag}/${urlText}`;

      const title = noTitle ? "" : `title="${identifier}"`;
      const lineNumbers = noLineNumbers ? "" : "showLineNumbers";
      const source = noSourceLink
        ? ""
        : `\n> <sup><sub><a href="${url}" target="_blank" rel="noopener noreferrer">Source code: ${urlText}</a></sub></sup>`;
      const replacement =
        language === "raw"
          ? codeSnippet
          : `\`\`\`${language} ${title} ${lineNumbers} \n${codeSnippet}\n\`\`\`${source}\n`;

      // Replace the include tag with the code snippet
      updatedContent = updatedContent.replace(fullMatch, replacement);
    } catch (error) {
      const lineNum = getLineNumberFromIndex(markdownContent, match.index);
      // We were warning here, but code snippets were being broken. So making this throw an error instead:
      throw new Error(
        `Error processing "${filePath}:${lineNum}": ${error.message}.`
      );
    }
  }

  return { content: updatedContent, isUpdated: matchesFound };
}

module.exports = {
  preprocessIncludeCode,
};
