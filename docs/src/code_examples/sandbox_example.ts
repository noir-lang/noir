// docs:start:index
import {
  AztecRPC,
  L2BlockL2Logs,
  PrivateKey,
  createAztecRpcClient,
  createDebugLogger,
  getSchnorrAccount,
  mustSucceedFetch,
} from '@aztec/aztec.js';
import { PrivateTokenContract } from '@aztec/noir-contracts/types';

////////////// CREATE THE CLIENT INTERFACE AND CONTACT THE SANDBOX //////////////
const logger = createDebugLogger('private-token');
const sandboxUrl = 'http://localhost:8080';

const aztecRpc = createAztecRpcClient(sandboxUrl, mustSucceedFetch);

const nodeInfo = await aztecRpc.getNodeInfo();

logger('Aztec Sandbox Info ', nodeInfo);
// docs:end:index

// docs:start:Accounts
////////////// CREATE SOME ACCOUNTS WITH SCHNORR SIGNERS //////////////
// Creates new accounts using an account contract that verifies schnorr signatures
// Returns once the deployment transactions have settled
const createSchnorrAccounts = async (numAccounts: number, aztecRpc: AztecRPC) => {
  const accountManagers = Array(numAccounts)
    .fill(0)
    .map(x =>
      getSchnorrAccount(
        aztecRpc,
        PrivateKey.random(), // encryption private key
        PrivateKey.random(), // signing private key
      ),
    );
  return await Promise.all(
    accountManagers.map(async x => {
      await x.waitDeploy({});
      return x;
    }),
  );
};

// Create 2 accounts and wallets to go with each
logger(`Creating accounts using schnorr signers...`);
const accounts = await createSchnorrAccounts(2, aztecRpc);

////////////// VERIFY THE ACCOUNTS WERE CREATED SUCCESSFULLY //////////////

const [alice, bob] = (await Promise.all(accounts.map(x => x.getCompleteAddress()))).map(x => x.address);

// Verify that the accounts were deployed
const registeredAccounts = (await aztecRpc.getAccounts()).map(x => x.address);
for (const [account, name] of [
  [alice, 'Alice'],
  [bob, 'Bob'],
] as const) {
  if (registeredAccounts.find(acc => acc.equals(account))) {
    logger(`Created ${name}'s account at ${account.toShortString()}`);
    continue;
  }
  logger(`Failed to create account for ${name}!`);
}
// docs:end:Accounts

// docs:start:Deployment
////////////// DEPLOY OUR PRIVATE TOKEN CONTRACT //////////////

// Deploy a private token contract, create a contract abstraction object and link it to the owner's wallet
// The contract's constructor takes 2 arguments, the initial supply and the owner of that initial supply
const initialSupply = 1_000_000;
logger(`Deploying private token contract minting an initial ${initialSupply} tokens to Alice...`);
const tokenContractTx = PrivateTokenContract.deploy(
  aztecRpc,
  initialSupply, // the initial supply
  alice, // the owner of the initial supply
).send();
// wait for the tx to settle
await tokenContractTx.isMined();
const receipt = await tokenContractTx.getReceipt();
logger(`Transaction status is ${receipt.status}`);
const contractData = await aztecRpc.getContractData(receipt.contractAddress!);
if (contractData) {
  logger(`Contract successfully deployed at address ${receipt.contractAddress!.toShortString()}`);
}
// docs:end:Deployment
// docs:start:Logs

////////////// RETRIEVE THE UNENCRYPTED LOGS EMITTED DURING DEPLOYMENT //////////////

// We can view the unencrypted logs emitted by the contract...
const viewUnencryptedLogs = async () => {
  const lastBlock = await aztecRpc.getBlockNum();
  logger(`Retrieving unencrypted logs for block ${lastBlock}`);
  const logs = await aztecRpc.getUnencryptedLogs(lastBlock, 1);
  const unrolledLogs = L2BlockL2Logs.unrollLogs(logs);
  const asciiLogs = unrolledLogs.map(log => log.toString('ascii'));
  logger(`Emitted logs: `, asciiLogs);
};
await viewUnencryptedLogs();

// docs:end:Logs
// docs:start:Balance

////////////// QUERYING THE TOKEN BALANCE FOR EACH ACCOUNT //////////////

// Create the contract abstraction and link to Alice's wallet for future signing
const tokenContractAlice = await PrivateTokenContract.at(receipt.contractAddress!, await accounts[0].getWallet());

// Bob wants to mint some funds, the contract is already deployed, create an abstraction and link it his wallet
const tokenContractBob = await PrivateTokenContract.at(receipt.contractAddress!, await accounts[1].getWallet());

const checkBalances = async () => {
  // Check Alice's balance
  logger(`Alice's balance ${await tokenContractAlice.methods.getBalance(alice).view()}`);
  // Check Bob's balance
  logger(`Bob's balance ${await tokenContractBob.methods.getBalance(bob).view()}`);
};
// Check the initial balances
await checkBalances();
// docs:end:Balance
// docs:start:Transfer
////////////// TRANSFER FUNDS FROM ALICE TO BOB //////////////

// We will now transfer tokens from ALice to Bob
const transferQuantity = 543;
logger(`Transferring ${transferQuantity} tokens from Alice to Bob...`);
await tokenContractAlice.methods.transfer(transferQuantity, alice, bob).send().wait();

// See if any logs were emitted
await viewUnencryptedLogs();

// Check the new balances
await checkBalances();
// docs:end:Transfer
// docs:start:Mint
////////////// MINT SOME MORE TOKENS TO BOB'S ACCOUNT //////////////

// Now mint some further funds for Bob
const mintQuantity = 10_000;
logger(`Minting ${mintQuantity} tokens to Bob...`);
await tokenContractBob.methods.mint(mintQuantity, bob).send().wait();

// See if any logs were emitted
await viewUnencryptedLogs();

// Check the new balances
await checkBalances();
// docs:end:Mint
