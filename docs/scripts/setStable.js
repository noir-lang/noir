/* eslint-disable */
const fs = require('fs');
const path = require('path');
const axios = require('axios');
const { release } = require('os');

const IGNORE_VERSIONS = ['0.16.0'];
const NUMBER_OF_VERSIONS_TO_SHOW = 2;

async function main() {
  const versionsFile = path.join(__dirname, '../versions.json');

  const { data } = await axios.get('https://api.github.com/repos/noir-lang/noir/releases', {
    headers: { Authorization: `token ${process.env.GITHUB_TOKEN}` },
    params: { per_page: 100 },
  });

  const stables = data
    .filter((release) => !release.prerelease && !release.tag_name.includes('aztec'))
    .filter((release) => !IGNORE_VERSIONS.includes(release.tag_name.replace('v', '')))
    .map((release) => release.tag_name.replace('v', ''))
    .slice(0, NUMBER_OF_VERSIONS_TO_SHOW);
  fs.writeFileSync(versionsFile, JSON.stringify(stables, null, 2));
}

main();
