const fs = require('fs');
const path = require('path');

const stableVersionPath = path.join(__dirname, '../stableVersion.json');
const stableVersionConfig = require(stableVersionPath);

stableVersionConfig.lastVersion = process.env.STABLE.replace('v', '');

fs.writeFileSync(stableVersionPath, JSON.stringify(stableVersionConfig, null, 2));
