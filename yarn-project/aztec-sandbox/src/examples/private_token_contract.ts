import {
  AccountWallet,
  AztecAddress,
  Contract,
  GrumpkinScalar,
  createAztecRpcClient,
  createRecipient,
  getUnsafeSchnorrAccount,
} from '@aztec/aztec.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { PrivateTokenContract } from '@aztec/noir-contracts/types';

const logger = createDebugLogger('aztec:http-rpc-client');

export const privateKey = GrumpkinScalar.fromString('ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80');

const url = 'http://localhost:8080';

const aztecRpcClient = createAztecRpcClient(url);
let wallet: AccountWallet;

const INITIAL_BALANCE = 333n;
const SECONDARY_AMOUNT = 33n;

/**
 * Deploys the Private Token contract.
 * @param owner - The address that the initial balance will belong to.
 * @returns An Aztec Contract object with the private token's ABI.
 */
async function deployZKContract(owner: AztecAddress) {
  logger('Deploying L2 contract...');
  const contract = await PrivateTokenContract.deploy(aztecRpcClient, INITIAL_BALANCE, owner).send().deployed();
  logger('L2 contract deployed');
  return contract;
}

/**
 * Gets a user's balance from a Private Token contract.
 * @param contract - The Private Token contract.
 * @param ownerAddress - Balance owner's Aztec Address.
 * @returns The owner's current balance of the token.
 */
async function getBalance(contract: Contract, ownerAddress: AztecAddress) {
  return await contract.methods.getBalance(ownerAddress).view({ from: ownerAddress });
}

/**
 * Main function.
 */
async function main() {
  logger('Running ZK contract test on HTTP interface.');

  wallet = await getUnsafeSchnorrAccount(aztecRpcClient, privateKey).waitDeploy();
  const owner = wallet.getCompleteAddress();
  const recipient = await createRecipient(aztecRpcClient);

  logger(`Created Owner account ${owner.toString()}`);

  const zkContract = await deployZKContract(owner.address);
  const [balance1] = await zkContract.methods.getBalance(owner.address).view({ from: owner.address });
  logger(`Initial owner balance: ${balance1}`);

  // Mint more tokens
  logger(`Minting ${SECONDARY_AMOUNT} more coins`);
  await zkContract.methods.mint(SECONDARY_AMOUNT, owner.address).send().wait({ interval: 0.5 });
  const balanceAfterMint = await getBalance(zkContract, owner.address);
  logger(`Owner's balance is now: ${balanceAfterMint}`);

  // Perform a transfer
  logger(`Transferring ${SECONDARY_AMOUNT} tokens from owner to another account.`);
  await zkContract.methods.transfer(SECONDARY_AMOUNT, recipient.address).send().wait({ interval: 0.5 });
  const balanceAfterTransfer = await getBalance(zkContract, owner.address);
  const receiverBalance = await getBalance(zkContract, recipient.address);
  logger(`Owner's balance is now ${balanceAfterTransfer}`);
  logger(`The transfer receiver's balance is ${receiverBalance}`);
}

main()
  .then(() => {
    logger('Finished running successfully.');
    process.exit(0);
  })
  .catch(err => {
    logger.error('Error in main fn: ', err);
    process.exit(1);
  });
