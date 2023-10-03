// docs:start:imports
import {
  Fr,
  NotePreimage,
  PXE,
  computeMessageSecretHash,
  createDebugLogger,
  createPXEClient,
  getSandboxAccountsWallets,
  getSchnorrAccount,
  waitForSandbox,
} from '@aztec/aztec.js';
import { GrumpkinScalar } from '@aztec/circuits.js';
import { TokenContract } from '@aztec/noir-contracts/types';

const { SANDBOX_URL = 'http://localhost:8080' } = process.env;
// docs:end:imports

describe('e2e_sandbox_example', () => {
  it('sandbox example works', async () => {
    // docs:start:setup
    ////////////// CREATE THE CLIENT INTERFACE AND CONTACT THE SANDBOX //////////////
    const logger = createDebugLogger('token');

    // We create PXE client connected to the sandbox URL
    const pxe = createPXEClient(SANDBOX_URL);
    // Wait for sandbox to be ready
    await waitForSandbox(pxe);

    const nodeInfo = await pxe.getNodeInfo();

    logger('Aztec Sandbox Info ', nodeInfo);
    // docs:end:setup

    expect(typeof nodeInfo.protocolVersion).toBe('number');
    expect(typeof nodeInfo.chainId).toBe('number');
    expect(typeof nodeInfo.l1ContractAddresses.rollupAddress).toBe('object');

    // For the sandbox quickstart we just want to show them preloaded accounts (since it is a quickstart)
    // We show creation of accounts in a later test

    // docs:start:load_accounts
    ////////////// LOAD SOME ACCOUNTS FROM THE SANDBOX //////////////
    // The sandbox comes with a set of created accounts. Load them
    const accounts = await getSandboxAccountsWallets(pxe);
    const alice = accounts[0].getAddress();
    const bob = accounts[1].getAddress();
    logger(`Loaded alice's account at ${alice.toShortString()}`);
    logger(`Loaded bob's account at ${bob.toShortString()}`);
    // docs:end:load_accounts

    // docs:start:Deployment
    ////////////// DEPLOY OUR TOKEN CONTRACT //////////////

    // Deploy a token contract, create a contract abstraction object and link it to the owner's wallet
    const initialSupply = 1_000_000n;

    logger(`Deploying token contract minting an initial ${initialSupply} tokens to Alice...`);
    const contract = await TokenContract.deploy(pxe, alice).send().deployed();

    // Create the contract abstraction and link to Alice's wallet for future signing
    const tokenContractAlice = await TokenContract.at(contract.address, accounts[0]);

    // add Bob as a minter
    await tokenContractAlice.methods.set_minter(bob, true).send().wait();

    logger(`Contract successfully deployed at address ${contract.address.toShortString()}`);

    const secret = Fr.random();
    const secretHash = await computeMessageSecretHash(secret);

    const receipt = await tokenContractAlice.methods.mint_private(initialSupply, secretHash).send().wait();

    const pendingShieldsStorageSlot = new Fr(5); // The storage slot of `pending_shields` is 5.
    const preimage = new NotePreimage([new Fr(initialSupply), secretHash]);
    await pxe.addNote(alice, contract.address, pendingShieldsStorageSlot, preimage, receipt.txHash);

    await tokenContractAlice.methods.redeem_shield(alice, initialSupply, secret).send().wait();
    // docs:end:Deployment

    // ensure that token contract is registered in PXE
    expect(await pxe.getContracts()).toEqual(expect.arrayContaining([contract.address]));

    // docs:start:Balance

    ////////////// QUERYING THE TOKEN BALANCE FOR EACH ACCOUNT //////////////

    // Bob wants to mint some funds, the contract is already deployed, create an abstraction and link it his wallet
    // Since we already have a token link, we can simply create a new instance of the contract linked to Bob's wallet
    const tokenContractBob = tokenContractAlice.withWallet(accounts[1]);

    let aliceBalance = await tokenContractAlice.methods.balance_of_private(alice).view();
    logger(`Alice's balance ${aliceBalance}`);

    let bobBalance = await tokenContractBob.methods.balance_of_private(bob).view();
    logger(`Bob's balance ${bobBalance}`);

    // docs:end:Balance

    expect(aliceBalance).toBe(initialSupply);
    expect(bobBalance).toBe(0n);

    // docs:start:Transfer
    ////////////// TRANSFER FUNDS FROM ALICE TO BOB //////////////

    // We will now transfer tokens from ALice to Bob
    const transferQuantity = 543n;
    logger(`Transferring ${transferQuantity} tokens from Alice to Bob...`);
    await tokenContractAlice.methods.transfer(alice, bob, transferQuantity, 0).send().wait();

    // Check the new balances
    aliceBalance = await tokenContractAlice.methods.balance_of_private(alice).view();
    logger(`Alice's balance ${aliceBalance}`);

    bobBalance = await tokenContractBob.methods.balance_of_private(bob).view();
    logger(`Bob's balance ${bobBalance}`);
    // docs:end:Transfer

    expect(aliceBalance).toBe(initialSupply - transferQuantity);
    expect(bobBalance).toBe(transferQuantity);

    // docs:start:Mint
    ////////////// MINT SOME MORE TOKENS TO BOB'S ACCOUNT //////////////

    // Now mint some further funds for Bob
    const mintQuantity = 10_000n;
    logger(`Minting ${mintQuantity} tokens to Bob...`);
    const mintPrivateReceipt = await tokenContractBob.methods.mint_private(mintQuantity, secretHash).send().wait();

    const bobPendingShield = new NotePreimage([new Fr(mintQuantity), secretHash]);
    await pxe.addNote(bob, contract.address, pendingShieldsStorageSlot, bobPendingShield, mintPrivateReceipt.txHash);

    await tokenContractBob.methods.redeem_shield(bob, mintQuantity, secret).send().wait();

    // Check the new balances
    aliceBalance = await tokenContractAlice.methods.balance_of_private(alice).view();
    logger(`Alice's balance ${aliceBalance}`);

    bobBalance = await tokenContractBob.methods.balance_of_private(bob).view();
    logger(`Bob's balance ${bobBalance}`);
    // docs:end:Mint

    expect(aliceBalance).toBe(initialSupply - transferQuantity);
    expect(bobBalance).toBe(transferQuantity + mintQuantity);
  }, 120_000);

  it('can create accounts on the sandbox', async () => {
    const logger = createDebugLogger('token');
    // We create PXE client connected to the sandbox URL
    const pxe = createPXEClient(SANDBOX_URL);
    // Wait for sandbox to be ready
    await waitForSandbox(pxe);

    // docs:start:create_accounts
    ////////////// CREATE SOME ACCOUNTS WITH SCHNORR SIGNERS //////////////
    // Creates new accounts using an account contract that verifies schnorr signatures
    // Returns once the deployment transactions have settled
    const createSchnorrAccounts = async (numAccounts: number, pxe: PXE) => {
      const accountManagers = Array(numAccounts)
        .fill(0)
        .map(() =>
          getSchnorrAccount(
            pxe,
            GrumpkinScalar.random(), // encryption private key
            GrumpkinScalar.random(), // signing private key
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
    const accounts = await createSchnorrAccounts(2, pxe);

    ////////////// VERIFY THE ACCOUNTS WERE CREATED SUCCESSFULLY //////////////

    const [alice, bob] = (await Promise.all(accounts.map(x => x.getCompleteAddress()))).map(x => x.address);

    // Verify that the accounts were deployed
    const registeredAccounts = (await pxe.getRegisteredAccounts()).map(x => x.address);
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
    // docs:end:create_accounts

    // check that alice and bob are in registeredAccounts
    expect(registeredAccounts.find(acc => acc.equals(alice))).toBeTruthy();
    expect(registeredAccounts.find(acc => acc.equals(bob))).toBeTruthy();
  });
});
