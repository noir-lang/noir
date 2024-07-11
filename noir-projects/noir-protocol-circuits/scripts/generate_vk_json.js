const path = require("path");
const fs = require("fs/promises");
const fs_stream = require("fs");
const child_process = require("child_process");

const { fromIni } = require("@aws-sdk/credential-providers");
const { S3 } = require("@aws-sdk/client-s3");

const crypto = require("crypto");

const BB_BIN_PATH = process.env.BB_BIN || "../../barretenberg/cpp/build/bin/bb";
const BUCKET_NAME = "aztec-ci-artifacts";
const PREFIX = "protocol";

function vkBinaryFileNameForArtifactName(outputFolder, artifactName) {
  return path.join(outputFolder, `${artifactName}.vk`);
}

function vkJsonFileNameForArtifactName(outputFolder, artifactName) {
  return path.join(outputFolder, `${artifactName}.vk.json`);
}

function vkDataFileNameForArtifactName(outputFolder, artifactName) {
  return path.join(outputFolder, `${artifactName}.vk.data.json`);
}

async function getBytecodeHash(artifactPath) {
  const { bytecode } = JSON.parse(await fs.readFile(artifactPath));
  return crypto.createHash("md5").update(bytecode).digest("hex");
}

function getBarretenbergHash() {
  if (process.env.BB_HASH) {
    return Promise.resolve(process.env.BB_HASH);
  }
  return new Promise((res, rej) => {
    const hash = crypto.createHash("md5");

    const rStream = fs_stream.createReadStream(BB_BIN_PATH);
    rStream.on("data", (data) => {
      hash.update(data);
    });
    rStream.on("end", () => {
      res(hash.digest("hex"));
    });
    rStream.on("error", (err) => {
      rej(err);
    });
  });
}

async function getNewArtifactHash(artifactPath, outputFolder, artifactName) {
  const bytecodeHash = await getBytecodeHash(artifactPath);
  const barretenbergHash = await getBarretenbergHash();
  const artifactHash = `${barretenbergHash}-${bytecodeHash}`;

  const vkDataPath = vkDataFileNameForArtifactName(outputFolder, artifactName);
  try {
    const { artifactHash: previousArtifactHash } = JSON.parse(
      await fs.readFile(vkDataPath, "utf8")
    );
    if (previousArtifactHash === artifactHash) {
      return null;
    } else {
      console.log(
        `Circuit ${artifactName} has changed, old hash ${previousArtifactHash}, new hash ${artifactHash}`
      );
    }
  } catch (ignored) {
    console.log("No on disk vk found for", artifactName);
  }
  return artifactHash;
}

async function processArtifact(artifactPath, outputFolder) {
  const artifactName = path.basename(artifactPath, ".json");

  const artifactHash = await getNewArtifactHash(
    artifactPath,
    outputFolder,
    artifactName
  );
  if (!artifactHash) {
    console.log("Reusing on disk vk for", artifactName);
    return;
  }

  let vkData = await readVKFromS3(artifactName, artifactHash);

  if (!vkData) {
    vkData = await generateVKData(
      artifactName,
      outputFolder,
      artifactPath,
      artifactHash
    );
    await writeVKToS3(artifactName, artifactHash, JSON.stringify(vkData));
  } else {
    console.log("Using VK from remote cache for", artifactName);
  }

  await fs.writeFile(
    vkDataFileNameForArtifactName(outputFolder, artifactName),
    JSON.stringify(vkData, null, 2)
  );
}

async function generateVKData(
  artifactName,
  outputFolder,
  artifactPath,
  artifactHash
) {
  console.log("Generating new vk for", artifactName);

  const binaryVkPath = vkBinaryFileNameForArtifactName(
    outputFolder,
    artifactName
  );
  const jsonVkPath = vkJsonFileNameForArtifactName(outputFolder, artifactName);

  const writeVkCommand = `${BB_BIN_PATH} write_vk_ultra_honk -h -b "${artifactPath}" -o "${binaryVkPath}"`;
  const vkAsFieldsCommand = `${BB_BIN_PATH} vk_as_fields_ultra_honk -k "${binaryVkPath}" -o "${jsonVkPath}"`;

  await new Promise((resolve, reject) => {
    child_process.exec(`${writeVkCommand} && ${vkAsFieldsCommand}`, (err) => {
      if (err) {
        reject(err);
      } else {
        resolve();
      }
    });
  });
  const binaryVk = await fs.readFile(binaryVkPath);
  const jsonVk = JSON.parse(await fs.readFile(jsonVkPath, "utf8"));
  await fs.unlink(jsonVkPath);
  await fs.unlink(binaryVkPath);

  const vkData = {
    keyAsBytes: binaryVk.toString("hex"),
    keyAsFields: jsonVk,
    artifactHash,
  };
  console.log("Generated vk for", artifactName);

  return vkData;
}

async function main() {
  let [artifactPath, outputFolder] = process.argv.slice(2);
  if (!artifactPath || !outputFolder) {
    console.log(
      "Usage: node generate_vk_json.js <artifactPath> <outputFolder>"
    );
    return;
  }
  processArtifact(artifactPath, outputFolder);
}

function generateS3Client() {
  return new S3({
    credentials: fromIni({
      profile: "default",
    }),
    region: "us-east-2",
  });
}

async function writeVKToS3(artifactName, artifactHash, body) {
  if (process.env.DISABLE_VK_S3_CACHE) {
    return;
  }
  try {
    const s3 = generateS3Client();
    await s3.putObject({
      Bucket: BUCKET_NAME,
      Key: `${PREFIX}/${artifactName}-${artifactHash}.json`,
      Body: body,
    });
  } catch (err) {
    console.warn("Could not write to S3 VK remote cache", err.message);
  }
}

async function readVKFromS3(artifactName, artifactHash) {
  if (process.env.DISABLE_VK_S3_CACHE) {
    return;
  }
  const key = `${PREFIX}/${artifactName}-${artifactHash}.json`;

  try {
    const s3 = generateS3Client();
    const { Body: response } = await s3.getObject({
      Bucket: BUCKET_NAME,
      Key: key,
    });

    const result = JSON.parse(await response.transformToString());
    return result;
  } catch (err) {
    console.warn(
      `Could not read VK from remote cache at s3://${BUCKET_NAME}/${key}`,
      err.message
    );
    return undefined;
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
