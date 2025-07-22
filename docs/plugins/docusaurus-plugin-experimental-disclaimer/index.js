/**
 * Docusaurus plugin that injects experimental disclaimers into noir_wasm documentation
 * This runs after TypeDoc generation to add disclaimers to all noir_wasm pages
 */

const fs = require('fs');
const path = require('path');

const experimentalDisclaimer = `import Experimental from '@site/src/components/Notes/_experimental.mdx';

<Experimental />

`;

/**
 * Add experimental disclaimer to a single file
 */
function addExperimentalDisclaimerToFile(filePath) {
  try {
    const content = fs.readFileSync(filePath, 'utf8');

    // Check if experimental disclaimer already exists
    if (content.includes("import Experimental from '@site/src/components/Notes/_experimental.mdx';")) {
      console.log('Experimental disclaimer already exists in:', path.relative(process.cwd(), filePath));
      return;
    }

    const lines = content.split('\n');

    // Find the first markdown heading (# title)
    let injected = false;
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].startsWith('# ') && !injected) {
        // Insert disclaimer after the heading
        lines.splice(i + 1, 0, '', experimentalDisclaimer);
        injected = true;
        break;
      }
    }

    // If no heading found, add at the beginning
    if (!injected) {
      lines.unshift(experimentalDisclaimer, '');
    }

    // Write the updated content back
    fs.writeFileSync(filePath, lines.join('\n'), 'utf8');
    console.log('Added experimental disclaimer to:', path.relative(process.cwd(), filePath));
  } catch (error) {
    console.error('Error processing file:', filePath, error.message);
  }
}

/**
 * Find all MDX files in a directory recursively
 */
function findMarkdownFiles(dir, files = []) {
  if (!fs.existsSync(dir)) {
    return files;
  }

  const items = fs.readdirSync(dir);

  for (const item of items) {
    const fullPath = path.join(dir, item);
    const stat = fs.statSync(fullPath);

    if (stat.isDirectory()) {
      findMarkdownFiles(fullPath, files);
    } else if (item.endsWith('.mdx')) {
      files.push(fullPath);
    }
  }

  return files;
}

/**
 * Process TypeDoc-generated noir_wasm files to add experimental disclaimers
 */
function processNoirWasmFiles(siteDir) {
  const noirWasmDir = path.join(siteDir, 'processed-docs', 'reference', 'NoirJS', 'noir_wasm');

  if (!fs.existsSync(noirWasmDir)) {
    console.log('No noir_wasm directory found - TypeDoc may not have generated files yet');
    return;
  }

  const files = findMarkdownFiles(noirWasmDir);

  if (files.length === 0) {
    console.log('No noir_wasm MDX files found to process');
    return;
  }

  console.log(`Adding experimental disclaimers to ${files.length} noir_wasm files...`);
  files.forEach(addExperimentalDisclaimerToFile);
  console.log('Experimental disclaimer processing complete!');
}

/**
 * Docusaurus plugin definition
 */
function pluginExperimentalDisclaimer(context, options) {
  return {
    name: 'docusaurus-plugin-experimental-disclaimer',

    async loadContent() {
      // This runs early in the process, inject disclaimers before content processing  
      console.log('Running experimental disclaimer plugin during content loading...');
      processNoirWasmFiles(context.siteDir);
      return {};
    }
  };
}

module.exports = pluginExperimentalDisclaimer;