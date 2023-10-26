const fs = require('fs');
const path = require('path');
const process = require('process');

function main() {
  const configFile = path.join(__dirname, '../docusaurus.config.js');

  // Read the Docusaurus config
  const config = require(configFile);

  config.presets[0][1].docs.lastVersion = process.env.STABLE.replace('v', ''); // Get the version from the script argument

  // // Update the lastVersion property
  // const newVersion = process.env.STABLE.replace('v', ''); // Get the version from the script argument
  // if (!newVersion) {
  //   process.exit(1);
  // }
  // const updatedContent = configContent.replace(/lastVersion: '[^']+'/, `lastVersion: '${newVersion}'`);

  // Write the updated content back
  fs.writeFileSync(configFile, config, 'utf8');
}

main();
