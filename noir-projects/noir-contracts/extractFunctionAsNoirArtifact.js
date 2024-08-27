const path = require("path");
const fs = require("fs").promises;

// Simple script to extract a contract function as a separate Noir artifact.
// We need to use this since the transpiling that we do on public functions make the contract artifacts
// unreadable by noir tooling, since they are no longer following the noir artifact format.
async function main() {
  let [contractArtifactPath, functionName] = process.argv.slice(2);
  if (!contractArtifactPath || !functionName) {
    console.log(
      "Usage: node extractFunctionAsNoirArtifact.js <contractArtifactPath> <functionName>"
    );
    return;
  }

  const contractArtifact = JSON.parse(
    await fs.readFile(contractArtifactPath, "utf8")
  );
  const func = contractArtifact.functions.find((f) => f.name === functionName);
  if (!func) {
    console.error(
      `Function ${functionName} not found in ${contractArtifactPath}`
    );
    return;
  }

  const artifact = {
    noir_version: contractArtifact.noir_version,
    hash: 0,
    abi: func.abi,
    bytecode: func.bytecode,
    debug_symbols: func.debug_symbols,
    file_map: contractArtifact.file_map,
    names: ["main"],
    brillig_names: func.brillig_names,
  };

  const outputDir = path.dirname(contractArtifactPath);
  const outputName =
    path.basename(contractArtifactPath, ".json") + `-${functionName}.json`;

  const outPath = path.join(outputDir, outputName);

  console.log(`Writing to ${outPath}`);

  await fs.writeFile(outPath, JSON.stringify(artifact, null, 2));
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
