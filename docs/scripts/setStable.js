const fs = require('fs');
const path = require('path');

const process = require('process');
const console = require('console');

function main() {
  const configFile = path.join('docusaurus.config.js');

  // Read the Docusaurus config
  const configContent = fs.readFileSync(configFile, 'utf8');

  // Update the lastVersion property
  const newVersion = process.env.STABLE; // Get the version from the script argument
  if (!newVersion) {
    console.log('No stable version provided');
    process.exit(1);
  }
  console.log(newVersion);
  const updatedContent = configContent.replace(/lastVersion: '[^']+'/, `lastVersion: '${newVersion}'`);

  console.log(updatedContent);
  // Write the updated content back
  fs.writeFileSync(configFile, updatedContent, 'utf8');
}

main();
