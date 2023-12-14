const fs = require('fs');
const path = require('path');
const axios = require('axios');

const IGNORE_VERSIONS = ['0.16.0'];
const NUMBER_OF_VERSIONS_TO_SHOW = 4;

async function main() {
  const versionsFile = path.resolve('../versions.json');

  const axiosOpts = {
    params: { per_page: 100 },
    headers: {},
  };

  if (process.env.GITHUB_TOKEN) axiosOpts.headers = { Authorization: `token ${process.env.GITHUB_TOKEN}` };

  const { data } = await axios.get('https://api.github.com/repos/noir-lang/noir/releases', axiosOpts);

  const all = data.map((release) => release.tag_name);
  console.log('All versions: ', all);
  const aztecs = data.filter((release) => release.tag_name.includes('aztec')).map((release) => release.tag_name);
  console.log('Removing aztecs: ', aztecs);
  const prereleases = data.filter((release) => !release.prerelease).map((release) => release.tag_name);
  console.log('Removing prereleases: ', prereleases);

  const stables = data
    .filter((release) => !release.prerelease && !release.tag_name.includes('aztec'))
    .filter((release) => !IGNORE_VERSIONS.includes(release.tag_name.replace('v', '')))
    .map((release) => release.tag_name)
    .slice(0, NUMBER_OF_VERSIONS_TO_SHOW);

  console.log('Stables: ', stables);

  fs.writeFileSync(versionsFile, JSON.stringify(stables, null, 2));
}

main();
