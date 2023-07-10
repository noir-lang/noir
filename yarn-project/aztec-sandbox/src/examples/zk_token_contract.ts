import {
  Contract,
  ContractDeployer,
  Wallet,
  createAccounts,
  createAztecRpcClient,
  pointToPublicKey,
} from '@aztec/aztec.js';
import { AztecAddress, Point } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { ZkTokenContractAbi } from '@aztec/noir-contracts/examples';

const logger = createDebugLogger('aztec:http-rpc-client');

export const privateKey = Buffer.from('ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80', 'hex');

const url = 'http://localhost:8080';

const aztecRpcClient = createAztecRpcClient(url);
let wallet: Wallet;

const INITIAL_BALANCE = 333n;
const SECONDARY_AMOUNT = 33n;

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
  const contract = new Contract(receipt.contractAddress!, ZkTokenContractAbi, wallet);
  await tx.isMined();
  await tx.getReceipt();
  logger('L2 contract deployed');
  return contract;
}

/**
 * Gets a user's balance from a ZK Token contract.
 * @param contract - The ZK Token contract.
 * @param ownerKey - Balance owner's public key.
 * @param ownerAddress - Balance owner's Aztec Address.
 * @returns The owner's current balance of the token.
 */
async function getBalance(contract: Contract, ownerKey: Point, ownerAddress: AztecAddress) {
  const [balance] = await contract.methods.getBalance(pointToPublicKey(ownerKey)).view({ from: ownerAddress });
  return balance;
}

/**
 * Main function.
 */
async function main() {
  logger('Running ZK contract test on HTTP interface.');

  wallet = await createAccounts(aztecRpcClient, privateKey, 1);
  const accounts = await aztecRpcClient.getAccounts();
  logger(`Created ${accounts.length} accounts`);
  const [ownerAddress, address2] = accounts;
  const ownerPubKeyPoint = await aztecRpcClient.getAccountPublicKey(ownerAddress);
  const address2PubKeyPoint = await aztecRpcClient.getAccountPublicKey(address2);
  logger(`Created account ${ownerAddress.toString()} with public key ${ownerPubKeyPoint.toString()}`);
  const zkContract = await deployZKContract(ownerPubKeyPoint);
  const [balance1] = await zkContract.methods
    .getBalance(pointToPublicKey(ownerPubKeyPoint))
    .view({ from: ownerAddress });
  logger(`Initial owner balance: ${balance1}`);

  // Mint more tokens
  logger(`Minting ${SECONDARY_AMOUNT} more coins`);
  const mintTx = zkContract.methods.mint(SECONDARY_AMOUNT, ownerPubKeyPoint).send({ from: ownerAddress });
  await mintTx.isMined(0, 0.5);
  const balanceAfterMint = await getBalance(zkContract, ownerPubKeyPoint, ownerAddress);
  logger(`Owner's balance is now: ${balanceAfterMint}`);

  // Perform a transfer
  logger(`Transferring ${SECONDARY_AMOUNT} tokens from owner to another account.`);
  const transferTx = zkContract.methods
    .transfer(SECONDARY_AMOUNT, ownerPubKeyPoint, address2PubKeyPoint)
    .send({ from: ownerAddress });
  await transferTx.isMined(0, 0.5);
  const balanceAfterTransfer = await getBalance(zkContract, ownerPubKeyPoint, ownerAddress);
  const receiverBalance = await getBalance(zkContract, address2PubKeyPoint, address2);
  logger(`Owner's balance is now ${balanceAfterTransfer}`);
  logger(`The transfer receiver's balance is ${receiverBalance}`);
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
