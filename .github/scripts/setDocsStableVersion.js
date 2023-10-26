const fs = require('fs');
const path = require('path');

const configFile = path.join(__dirname, '..', '..', '..', '..', 'docs', 'docusaurus.config.js');

// Read the Docusaurus config
const configContent = fs.readFileSync(configFile, 'utf8');

// Update the lastVersion property
const newVersion = process.argv[2]; // Get the version from the script argument
const updatedContent = configContent.replace(/lastVersion: '[^']+'/, `lastVersion: '${newVersion}'`);

// Write the updated content back
fs.writeFileSync(configFile, updatedContent, 'utf8');

console.log(`Updated lastVersion to ${newVersion}`);
