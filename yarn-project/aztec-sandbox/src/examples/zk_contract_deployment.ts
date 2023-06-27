import { Contract, ContractDeployer, createAccounts, createAztecRpcClient, pointToPublicKey } from '@aztec/aztec.js';
import { Point } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { ZkTokenContractAbi } from '@aztec/noir-contracts/examples';

const logger = createDebugLogger('aztec:http-rpc-client');

export const privateKey = Buffer.from('ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80', 'hex');

const url = 'http://localhost:8080';

const aztecRpcClient = createAztecRpcClient(url);

const INITIAL_BALANCE = 333n;

/**
 * Deploys the ZK Token contract.
 * @param pubKeyPoint - The public key Point that the initial balance will belong to.
 * @returns An Aztec Contract object with the zk token's ABI.
 */
async function deployZKContract(pubKeyPoint: Point) {
  logger('Deploying L2 contract...');
  const deployer = new ContractDeployer(ZkTokenContractAbi, aztecRpcClient);
  const tx = deployer.deploy(INITIAL_BALANCE, pointToPublicKey(pubKeyPoint)).send();
  const receipt = await tx.getReceipt();
  const contract = new Contract(receipt.contractAddress!, ZkTokenContractAbi, aztecRpcClient);
  await tx.isMined();
  await tx.getReceipt();
  logger('L2 contract deployed');
  return contract;
}

/**
 * Main function.
 */
async function main() {
  logger('Running ZK contract test on HTTP interface.');

  const [address, pubKeyPoint] = (await createAccounts(aztecRpcClient, privateKey, 1))[0];
  logger(`Created account ${address.toString()} with public key ${pubKeyPoint.toString()}`);
  const zkContract = await deployZKContract(pubKeyPoint);
  const accounts = await aztecRpcClient.getAccounts();
  logger(`Created ${accounts.length} accounts`);
  const [balance1] = await zkContract.methods.getBalance(pointToPublicKey(pubKeyPoint)).view({ from: address });
  logger(`Initial owner balance: ${balance1}`);
}

main()
  .then(() => {
    logger('Finished running successfuly.');
    process.exit(0);
  })
  .catch(err => {
    logger('Error in main fn: ', err);
    process.exit(1);
  });
