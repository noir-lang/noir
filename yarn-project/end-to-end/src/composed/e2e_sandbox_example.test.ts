// docs:start:imports
import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { getDeployedTestAccountsWallets } from '@aztec/accounts/testing';
import {
  ExtendedNote,
  Fr,
  GrumpkinScalar,
  Note,
  type PXE,
  computeSecretHash,
  createDebugLogger,
  createPXEClient,
  waitForPXE,
} from '@aztec/aztec.js';
import { TokenContract } from '@aztec/noir-contracts.js/Token';

import { format } from 'util';

const { PXE_URL = 'http://localhost:8080' } = process.env;
// docs:end:imports

describe('e2e_sandbox_example', () => {
  it('sandbox example works', async () => {
    // docs:start:setup
    ////////////// CREATE THE CLIENT INTERFACE AND CONTACT THE SANDBOX //////////////
    const logger = createDebugLogger('token');

    // We create PXE client connected to the sandbox URL
    const pxe = createPXEClient(PXE_URL);
    // Wait for sandbox to be ready
    await waitForPXE(pxe, logger);

    const nodeInfo = await pxe.getNodeInfo();

    logger.info(format('Aztec Sandbox Info ', nodeInfo));
    // docs:end:setup

    expect(typeof nodeInfo.protocolVersion).toBe('number');
    expect(typeof nodeInfo.l1ChainId).toBe('number');
    expect(typeof nodeInfo.l1ContractAddresses.rollupAddress).toBe('object');

    // For the sandbox quickstart we just want to show them preloaded accounts (since it is a quickstart)
    // We show creation of accounts in a later test

    // docs:start:load_accounts
    ////////////// LOAD SOME ACCOUNTS FROM THE SANDBOX //////////////
    // The sandbox comes with a set of created accounts. Load them
    const accounts = await getDeployedTestAccountsWallets(pxe);
    const aliceWallet = accounts[0];
    const bobWallet = accounts[1];
    const alice = aliceWallet.getAddress();
    const bob = bobWallet.getAddress();
    logger.info(`Loaded alice's account at ${alice.toShortString()}`);
    logger.info(`Loaded bob's account at ${bob.toShortString()}`);
    // docs:end:load_accounts

    // docs:start:Deployment
    ////////////// DEPLOY OUR TOKEN CONTRACT //////////////

    const initialSupply = 1_000_000n;
    logger.info(`Deploying token contract...`);

    // Deploy the contract and set Alice as the admin while doing so
    const contract = await TokenContract.deploy(aliceWallet, alice, 'TokenName', 'TokenSymbol', 18).send().deployed();
    logger.info(`Contract successfully deployed at address ${contract.address.toShortString()}`);

    // Create the contract abstraction and link it to Alice's wallet for future signing
    const tokenContractAlice = await TokenContract.at(contract.address, aliceWallet);

    // Create a secret and a corresponding hash that will be used to mint funds privately
    const aliceSecret = Fr.random();
    const aliceSecretHash = computeSecretHash(aliceSecret);

    logger.info(`Minting tokens to Alice...`);
    // Mint the initial supply privately "to secret hash"
    const receipt = await tokenContractAlice.methods.mint_private(initialSupply, aliceSecretHash).send().wait();

    // Add the newly created "pending shield" note to PXE
    const note = new Note([new Fr(initialSupply), aliceSecretHash]);
    await pxe.addNote(
      new ExtendedNote(
        note,
        alice,
        contract.address,
        TokenContract.storage.pending_shields.slot,
        TokenContract.notes.TransparentNote.id,
        receipt.txHash,
      ),
    );

    // Make the tokens spendable by redeeming them using the secret (converts the "pending shield note" created above
    // to a "token note")
    await tokenContractAlice.methods.redeem_shield(alice, initialSupply, aliceSecret).send().wait();
    logger.info(`${initialSupply} tokens were successfully minted and redeemed by Alice`);
    // docs:end:Deployment

    // ensure that token contract is registered in PXE
    expect(await pxe.getContracts()).toEqual(expect.arrayContaining([contract.address]));

    // docs:start:Balance

    ////////////// QUERYING THE TOKEN BALANCE FOR EACH ACCOUNT //////////////

    // Bob wants to mint some funds, the contract is already deployed, create an abstraction and link it his wallet
    // Since we already have a token link, we can simply create a new instance of the contract linked to Bob's wallet
    const tokenContractBob = tokenContractAlice.withWallet(bobWallet);

    let aliceBalance = await tokenContractAlice.methods.balance_of_private(alice).simulate();
    logger.info(`Alice's balance ${aliceBalance}`);

    let bobBalance = await tokenContractBob.methods.balance_of_private(bob).simulate();
    logger.info(`Bob's balance ${bobBalance}`);

    // docs:end:Balance

    expect(aliceBalance).toBe(initialSupply);
    expect(bobBalance).toBe(0n);

    // docs:start:Transfer
    ////////////// TRANSFER FUNDS FROM ALICE TO BOB //////////////

    // We will now transfer tokens from ALice to Bob
    const transferQuantity = 543n;
    logger.info(`Transferring ${transferQuantity} tokens from Alice to Bob...`);
    await tokenContractAlice.methods.transfer(bob, transferQuantity).send().wait();

    // Check the new balances
    aliceBalance = await tokenContractAlice.methods.balance_of_private(alice).simulate();
    logger.info(`Alice's balance ${aliceBalance}`);

    bobBalance = await tokenContractBob.methods.balance_of_private(bob).simulate();
    logger.info(`Bob's balance ${bobBalance}`);
    // docs:end:Transfer

    expect(aliceBalance).toBe(initialSupply - transferQuantity);
    expect(bobBalance).toBe(transferQuantity);

    // docs:start:Mint
    ////////////// MINT SOME MORE TOKENS TO BOB'S ACCOUNT //////////////

    // Now mint some further funds for Bob

    // Alice is nice and she adds Bob as a minter
    await tokenContractAlice.methods.set_minter(bob, true).send().wait();

    const bobSecret = Fr.random();
    const bobSecretHash = computeSecretHash(bobSecret);
    // Bob now has a secret 🥷

    const mintQuantity = 10_000n;
    logger.info(`Minting ${mintQuantity} tokens to Bob...`);
    const mintPrivateReceipt = await tokenContractBob.methods.mint_private(mintQuantity, bobSecretHash).send().wait();

    const bobPendingShield = new Note([new Fr(mintQuantity), bobSecretHash]);
    await pxe.addNote(
      new ExtendedNote(
        bobPendingShield,
        bob,
        contract.address,
        TokenContract.storage.pending_shields.slot,
        TokenContract.notes.TransparentNote.id,
        mintPrivateReceipt.txHash,
      ),
    );

    await tokenContractBob.methods.redeem_shield(bob, mintQuantity, bobSecret).send().wait();

    // Check the new balances
    aliceBalance = await tokenContractAlice.methods.balance_of_private(alice).simulate();
    logger.info(`Alice's balance ${aliceBalance}`);

    bobBalance = await tokenContractBob.methods.balance_of_private(bob).simulate();
    logger.info(`Bob's balance ${bobBalance}`);
    // docs:end:Mint

    expect(aliceBalance).toBe(initialSupply - transferQuantity);
    expect(bobBalance).toBe(transferQuantity + mintQuantity);
  });

  it('can create accounts on the sandbox', async () => {
    const logger = createDebugLogger('token');
    // We create PXE client connected to the sandbox URL
    const pxe = createPXEClient(PXE_URL);
    // Wait for sandbox to be ready
    await waitForPXE(pxe, logger);

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
            Fr.random(), // secret key
            GrumpkinScalar.random(), // signing private key
          ),
        );
      return await Promise.all(
        accountManagers.map(async x => {
          await x.waitSetup({});
          return x;
        }),
      );
    };

    // Create 2 accounts and wallets to go with each
    logger.info(`Creating accounts using schnorr signers...`);
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
        logger.info(`Created ${name}'s account at ${account.toShortString()}`);
        continue;
      }
      logger.info(`Failed to create account for ${name}!`);
    }
    // docs:end:create_accounts

    // check that alice and bob are in registeredAccounts
    expect(registeredAccounts.find(acc => acc.equals(alice))).toBeTruthy();
    expect(registeredAccounts.find(acc => acc.equals(bob))).toBeTruthy();
  });
});
