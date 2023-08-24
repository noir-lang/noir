/* eslint-disable @typescript-eslint/no-unused-vars */

/* eslint-disable import/no-duplicates */
// docs:start:imports
import {
  AztecRPC,
  PrivateKey,
  createAztecRpcClient,
  createDebugLogger,
  getSchnorrAccount,
  makeFetch,
  waitForSandbox,
} from '@aztec/aztec.js';
// docs:end:imports

/* eslint-enable @typescript-eslint/no-unused-vars */
// Note: this is a hack to make the docs use http://localhost:8080 and CI to use the SANDBOX_URL
import { createAztecRpcClient as createAztecRpcClient2 } from '@aztec/aztec.js';
import { defaultFetch } from '@aztec/foundation/json-rpc/client';
import { PrivateTokenContract } from '@aztec/noir-contracts/types';

const { SANDBOX_URL = 'http://localhost:8080' } = process.env;

describe('e2e_sandbox_example', () => {
  // Note: this is a hack to make the docs use http://localhost:8080 and CI to use the SANDBOX_URL
  const createAztecRpcClient = (url: string, fetch = defaultFetch) => {
    return createAztecRpcClient2(SANDBOX_URL!, fetch);
  };

  it('sandbox example works', async () => {
    // docs:start:setup
    ////////////// CREATE THE CLIENT INTERFACE AND CONTACT THE SANDBOX //////////////
    const logger = createDebugLogger('private-token');
    const sandboxUrl = 'http://localhost:8080';

    // We create AztecRPC client connected to the sandbox URL and we use fetch with
    // 3 automatic retries and a 1s, 2s and 3s intervals between failures.
    const aztecRpc = createAztecRpcClient(sandboxUrl, makeFetch([1, 2, 3], false));
    // Wait for sandbox to be ready
    await waitForSandbox(aztecRpc);

    const nodeInfo = await aztecRpc.getNodeInfo();

    logger('Aztec Sandbox Info ', nodeInfo);
    // docs:end:setup

    expect(typeof nodeInfo.version).toBe('number');
    expect(typeof nodeInfo.chainId).toBe('number');
    expect(typeof nodeInfo.rollupAddress).toBe('object');

    // docs:start:Accounts
    ////////////// CREATE SOME ACCOUNTS WITH SCHNORR SIGNERS //////////////
    // Creates new accounts using an account contract that verifies schnorr signatures
    // Returns once the deployment transactions have settled
    const createSchnorrAccounts = async (numAccounts: number, aztecRpc: AztecRPC) => {
      const accountManagers = Array(numAccounts)
        .fill(0)
        .map(() =>
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

    // check that alice and bob are in registeredAccounts
    expect(registeredAccounts.find(acc => acc.equals(alice))).toBeTruthy();
    expect(registeredAccounts.find(acc => acc.equals(bob))).toBeTruthy();

    // docs:start:Deployment
    ////////////// DEPLOY OUR PRIVATE TOKEN CONTRACT //////////////

    // Deploy a private token contract, create a contract abstraction object and link it to the owner's wallet
    // The contract's constructor takes 2 arguments, the initial supply and the owner of that initial supply
    const initialSupply = 1_000_000n;

    logger(`Deploying private token contract minting an initial ${initialSupply} tokens to Alice...`);
    const contract = await PrivateTokenContract.deploy(
      aztecRpc,
      initialSupply, // the initial supply
      alice, // the owner of the initial supply
    )
      .send()
      .deployed();

    logger(`Contract successfully deployed at address ${contract.address!.toShortString()}`);
    // docs:end:Deployment

    // ensure that private token contract is registered in the rpc
    expect(await aztecRpc.getContracts()).toEqual(expect.arrayContaining([contract.address]));

    // docs:start:Balance

    ////////////// QUERYING THE TOKEN BALANCE FOR EACH ACCOUNT //////////////

    // Create the contract abstraction and link to Alice's wallet for future signing
    const tokenContractAlice = await PrivateTokenContract.at(contract.address!, await accounts[0].getWallet());

    // Bob wants to mint some funds, the contract is already deployed, create an abstraction and link it his wallet
    const tokenContractBob = await PrivateTokenContract.at(contract.address!, await accounts[1].getWallet());

    let aliceBalance = await tokenContractAlice.methods.getBalance(alice).view();
    logger(`Alice's balance ${aliceBalance}`);

    let bobBalance = await tokenContractBob.methods.getBalance(bob).view();
    logger(`Bob's balance ${bobBalance}`);

    // docs:end:Balance

    expect(aliceBalance).toBe(initialSupply);
    expect(bobBalance).toBe(0n);

    // docs:start:Transfer
    ////////////// TRANSFER FUNDS FROM ALICE TO BOB //////////////

    // We will now transfer tokens from ALice to Bob
    const transferQuantity = 543n;
    logger(`Transferring ${transferQuantity} tokens from Alice to Bob...`);
    await tokenContractAlice.methods.transfer(transferQuantity, alice, bob).send().wait();

    // Check the new balances
    aliceBalance = await tokenContractAlice.methods.getBalance(alice).view();
    logger(`Alice's balance ${aliceBalance}`);

    bobBalance = await tokenContractBob.methods.getBalance(bob).view();
    logger(`Bob's balance ${bobBalance}`);
    // docs:end:Transfer

    expect(aliceBalance).toBe(initialSupply - transferQuantity);
    expect(bobBalance).toBe(transferQuantity);

    // docs:start:Mint
    ////////////// MINT SOME MORE TOKENS TO BOB'S ACCOUNT //////////////

    // Now mint some further funds for Bob
    const mintQuantity = 10_000n;
    logger(`Minting ${mintQuantity} tokens to Bob...`);
    await tokenContractBob.methods.mint(mintQuantity, bob).send().wait();

    // Check the new balances
    aliceBalance = await tokenContractAlice.methods.getBalance(alice).view();
    logger(`Alice's balance ${aliceBalance}`);

    bobBalance = await tokenContractBob.methods.getBalance(bob).view();
    logger(`Bob's balance ${bobBalance}`);
    // docs:end:Mint

    expect(aliceBalance).toBe(initialSupply - transferQuantity);
    expect(bobBalance).toBe(transferQuantity + mintQuantity);
  }, 60_000);
});
