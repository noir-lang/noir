import fs from "fs";
const {readFileSync, promises: fsPromises} = fs;
import {spawn} from "child_process";
import {ethers} from "ethers";
import solc from "solc";

const NUMBER_OF_FIELDS_IN_PROOF = 93;

// We use the solcjs compiler version in this test, although it is slower than foundry, to run the test end to end
// it simplifies of parallelising the test suite

// What does this file do?
//
// 1. Launch an instance of anvil { on a random port, for parallelism }
// 2. Compile the solidity files using solcjs
// 3. Deploy the contract
// 4. Read the previously created proof, and append public inputs
// 5. Run the test against the deployed contract
// 6. Kill the anvil instance

const getEnvVar = (envvar) => {
  const varVal = process.env[envvar];
  if (!varVal) {
    throw new Error(`Missing environment variable ${envvar}`);
  }
  return varVal;
}

// Test name is passed into environment from `flows/sol.sh`
const testName = getEnvVar("TEST_NAME");

// Get solidity files, passed into environment from `flows/sol.sh`
const keyPath = getEnvVar("KEY_PATH");
const verifierPath = getEnvVar("VERIFIER_PATH");
const testPath = getEnvVar("TEST_PATH");
const basePath = getEnvVar("BASE_PATH");
const encoding = {encoding: "utf8"};
const [key, test, verifier, base] = await Promise.all(
  [
    fsPromises.readFile(keyPath, encoding),
    fsPromises.readFile(testPath, encoding),
    fsPromises.readFile(verifierPath, encoding),
    fsPromises.readFile(basePath, encoding)
  ]);

var input = {
  language: 'Solidity',
  sources: {
    'Key.sol': {
      content: key
    },
    'Test.sol': {
      content: test
    },
    'Verifier.sol': {
      content: verifier
    },
    'BaseUltraVerifier.sol': {
      content: base
    }
  },
  settings: { // we require the optimizer
    optimizer: {
      enabled: true,
      runs: 200
    },
    outputSelection: {
      '*': {
        '*': ['evm.bytecode.object', 'abi']
      }
    }
  }
};

var output = JSON.parse(solc.compile(JSON.stringify(input)));
const contract = output.contracts['Test.sol']['Test'];
const bytecode = contract.evm.bytecode.object;
const abi = contract.abi;

/**
 * Launch anvil on the given port,
 * Resolves when ready, rejects when port is already allocated
 * @param {Number} port
 */
const launchAnvil = async (port) => {
  const handle = spawn("anvil", ["-p", port]);

  // wait until the anvil instance is ready on port
  await new Promise((resolve, reject) => {
    // If we get an error reject, which will cause the caller to retry on a new port
    handle.stderr.on("data", (data) => {
      const str = data.toString();
      if (str.includes("error binding")) {
        reject("we go again baby")
      }
    });

    // If we get a success resolve, anvil is ready
    handle.stdout.on("data", (data) => {
      const str = data.toString();
      if (str.includes("Listening on")) {
        resolve(undefined);
      }
    });
  });

  return handle;
}

/**
 * Deploys the contract
 * @param {ethers.Signer} signer
 */
const deploy = async (signer) => {
    const factory = new ethers.ContractFactory(abi, bytecode, signer);
    const deployment = await factory.deploy();
    const deployed = await deployment.waitForDeployment();
    return await deployed.getAddress();
}

/**
 * Takes in a proof as fields, and returns the public inputs, as well as the number of public inputs
 * @param {Array<String>} proofAsFields 
 * @return {Array} [number, Array<String>] 
 */
const readPublicInputs = (proofAsFields) => {
  const publicInputs = [];
  // A proof with no public inputs is 93 fields long
  const numPublicInputs = proofAsFields.length - NUMBER_OF_FIELDS_IN_PROOF;
  for (let i = 0; i < numPublicInputs; i++) {
    publicInputs.push(proofAsFields[i]);
  }
  return [numPublicInputs, publicInputs];
}

/**
 * Get Anvil
 * 
 * Creates an anvil instance on a random port, and returns the instance and the port
 * If the port is alredy allocated, it will try again
 * @returns {[ChildProcess, Number]} [anvil, port]
 */
const getAnvil = async () => {
  const port = Math.floor(Math.random() * 10000) + 10000;
  try {
    return [await launchAnvil(port), port];
  } catch (e) {
    // Recursive call should try again on a new port in the rare case the port is already taken
    // yes this looks dangerous, but it relies on 0-10000 being hard to collide on
    return getAnvil();
  }
}

const [anvil, randomPort] = await getAnvil();
const killAnvil = () => {
  anvil.kill();
  console.log(testName, " complete")
}

try {
  const proofAsFieldsPath = getEnvVar("PROOF_AS_FIELDS");
  const proofAsFields = readFileSync(proofAsFieldsPath);
  const [numPublicInputs, publicInputs] = readPublicInputs(JSON.parse(proofAsFields.toString()));

  const proofPath = getEnvVar("PROOF");
  const proof = readFileSync(proofPath);
  
  // Cut the number of public inputs off of the proof string
  const proofStr = `0x${proof.toString("hex").substring(64*numPublicInputs)}`;

  const key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
  const provider = new ethers.JsonRpcProvider(`http://localhost:${randomPort}`);
  const signer = new ethers.Wallet(key, provider);

  // deploy 
  const address = await deploy(signer);
  const contract = new ethers.Contract(address, abi, signer);

  const result = await contract.test(proofStr, publicInputs);
  if (!result) throw new Error("Test failed");
}
catch (e) {
  console.error(testName, " failed")
  console.log(e)
  throw e;
}
finally {
  // Kill anvil at the end of running
  killAnvil();
}

