/**
 * Configurable Docusaurus plugin that injects experimental disclaimers into specified documentation
 * Can be used with any generated documentation by specifying target directories
 */

const fs = require('fs');
const path = require('path');

/**
 * Default disclaimer content
 */
const defaultDisclaimer = `import Experimental from '@site/src/components/Notes/_experimental.mdx';

<Experimental />

`;

/**
 * Add disclaimer to a single file
 */
function addDisclaimerToFile(filePath, disclaimer, skipCheckPattern) {
  try {
    const content = fs.readFileSync(filePath, 'utf8');

    // Check if disclaimer already exists
    if (content.includes(skipCheckPattern)) {
      console.log(`Disclaimer already exists in: ${path.relative(process.cwd(), filePath)}`);
      return;
    }

    const lines = content.split('\n');

    // Find the first markdown heading (# title)
    let injected = false;
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].startsWith('# ') && !injected) {
        // Insert disclaimer after the heading
        lines.splice(i + 1, 0, '', disclaimer);
        injected = true;
        break;
      }
    }

    // If no heading found, add at the beginning
    if (!injected) {
      lines.unshift(disclaimer, '');
    }

    // Write the updated content back
    fs.writeFileSync(filePath, lines.join('\n'), 'utf8');
    console.log(`Added disclaimer to: ${path.relative(process.cwd(), filePath)}`);
  } catch (error) {
    console.error(`Error processing file: ${filePath}`, error.message);
  }
}

/**
 * Find all MDX files in a directory recursively
 */
function findMarkdownFiles(dir, extensions = ['.mdx'], files = []) {
  if (!fs.existsSync(dir)) {
    return files;
  }

  const items = fs.readdirSync(dir);

  for (const item of items) {
    const fullPath = path.join(dir, item);
    const stat = fs.statSync(fullPath);

    if (stat.isDirectory()) {
      findMarkdownFiles(fullPath, extensions, files);
    } else if (extensions.some(ext => item.endsWith(ext))) {
      files.push(fullPath);
    }
  }

  return files;
}

/**
 * Process specified directories to add disclaimers
 */
function processDirectories(siteDir, targets, disclaimer, skipCheckPattern) {
  let totalFilesProcessed = 0;

  targets.forEach(target => {
    const targetDir = path.resolve(siteDir, target);
    
    if (!fs.existsSync(targetDir)) {
      console.log(`Directory not found: ${target} - may not have been generated yet`);
      return;
    }

    const files = findMarkdownFiles(targetDir);

    if (files.length === 0) {
      console.log(`No MDX files found in: ${target}`);
      return;
    }

    console.log(`Processing ${files.length} files in: ${target}`);
    files.forEach(file => addDisclaimerToFile(file, disclaimer, skipCheckPattern));
    totalFilesProcessed += files.length;
  });

  if (totalFilesProcessed > 0) {
    console.log(`Disclaimer processing complete! Processed ${totalFilesProcessed} files.`);
  }
}

/**
 * Docusaurus plugin definition
 */
function pluginExperimentalDisclaimer(context, options = {}) {
  const {
    targets = [],
    disclaimer = defaultDisclaimer,
    skipCheckPattern = "import Experimental from '@site/src/components/Notes/_experimental.mdx';"
  } = options;

  return {
    name: 'docusaurus-plugin-experimental-disclaimer',

    async loadContent() {
      if (targets.length === 0) {
        console.log('No target directories specified for experimental disclaimer plugin');
        return {};
      }

      console.log('Running experimental disclaimer plugin...');
      processDirectories(context.siteDir, targets, disclaimer, skipCheckPattern);
      return {};
    }
  };
}

module.exports = pluginExperimentalDisclaimer;