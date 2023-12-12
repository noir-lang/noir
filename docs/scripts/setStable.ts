/* eslint-disable @typescript-eslint/no-var-requires */
const fs = require('fs');
const path = require('path');
const axios = require('axios');

const IGNORE_VERSIONS = ['0.16.0'];
const NUMBER_OF_VERSIONS_TO_SHOW = 3;

async function main() {
  const versionsFile = path.resolve('../versions.json');

  const axiosOpts = {
    params: { per_page: 100 },
    headers: {},
  };

  if (process.env.GITHUB_TOKEN) axiosOpts.headers = { Authorization: `token ${process.env.GITHUB_TOKEN}` };

  const { data } = await axios.get('https://api.github.com/repos/noir-lang/noir/releases', axiosOpts);

  const stables = data
    .filter(
      (release) => !release.prerelease && !release.tag_name.includes('aztec') && !release.tag_name.includes('aztec'),
    )
    .filter((release) => !IGNORE_VERSIONS.includes(release.tag_name.replace('v', '')))
    .map((release) => release.tag_name)
    .slice(0, NUMBER_OF_VERSIONS_TO_SHOW);

  console.log('Filtered down to stables: ', stables);

  const onlyLatestPatches = [];
  const minorsSet = new Set(stables.map((el) => el.split('.')[1]));
  for (const minor of minorsSet) {
    const minorVersions = stables.filter((el) => el.split('.')[1] === minor);
    const max = minorVersions.reduce((prev, current) => {
      return prev > current ? prev : current;
    });
    onlyLatestPatches.push(max);
  }

  console.log('Only latest patches: ', onlyLatestPatches);

  fs.writeFileSync(versionsFile, JSON.stringify(onlyLatestPatches, null, 2));
}

main();
