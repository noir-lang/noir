const fs = require('fs');
const path = require('path');
const process = require('process');

function main() {
  const configFile = path.join('docusaurus.config.js');

  // Read the Docusaurus config
  const configContent = fs.readFileSync(configFile, 'utf8');

  // Update the lastVersion property
  const newVersion = process.env.STABLE.replace('v', ''); // Get the version from the script argument
  if (!newVersion) {
    process.exit(1);
  }
  const updatedContent = configContent.replace(/lastVersion: '[^']+'/, `lastVersion: '${newVersion}'`);

  // Write the updated content back
  fs.writeFileSync(configFile, updatedContent, 'utf8');
}

main();
