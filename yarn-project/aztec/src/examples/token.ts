import { getSingleKeyAccount } from '@aztec/accounts/single_key';
import { AccountWallet, Fr, GrumpkinScalar, Note, computeMessageSecretHash, createPXEClient } from '@aztec/aztec.js';
import { ExtendedNote } from '@aztec/circuit-types';
import { createDebugLogger } from '@aztec/foundation/log';
import { TokenContract } from '@aztec/noir-contracts/Token';

const logger = createDebugLogger('aztec:http-rpc-client');

export const alicePrivateKey = GrumpkinScalar.random();
export const bobPrivateKey = GrumpkinScalar.random();

const url = 'http://localhost:8080';

const pxe = createPXEClient(url);

let aliceWallet: AccountWallet;
let bobWallet: AccountWallet;

const ALICE_MINT_BALANCE = 333n;
const TRANSFER_AMOUNT = 33n;

/**
 * Main function.
 */
async function main() {
  logger('Running token contract test on HTTP interface.');

  aliceWallet = await getSingleKeyAccount(pxe, alicePrivateKey).waitDeploy();
  bobWallet = await getSingleKeyAccount(pxe, bobPrivateKey).waitDeploy();
  const alice = aliceWallet.getCompleteAddress();
  const bob = bobWallet.getCompleteAddress();

  logger(`Created Alice and Bob accounts: ${alice.address.toString()}, ${bob.address.toString()}`);

  logger('Deploying Token...');
  const token = await TokenContract.deploy(aliceWallet, alice, 'TokenName', 'TokenSymbol', 18).send().deployed();
  logger('Token deployed');

  // Create the contract abstraction and link it to Alice's and Bob's wallet for future signing
  const tokenAlice = await TokenContract.at(token.address, aliceWallet);
  const tokenBob = await TokenContract.at(token.address, bobWallet);

  // Mint tokens to Alice
  logger(`Minting ${ALICE_MINT_BALANCE} more coins to Alice...`);

  // Create a secret and a corresponding hash that will be used to mint funds privately
  const aliceSecret = Fr.random();
  const aliceSecretHash = computeMessageSecretHash(aliceSecret);
  const receipt = await tokenAlice.methods.mint_private(ALICE_MINT_BALANCE, aliceSecretHash).send().wait();

  // Add the newly created "pending shield" note to PXE
  const pendingShieldsStorageSlot = new Fr(5); // The storage slot of `pending_shields` is 5.
  const note = new Note([new Fr(ALICE_MINT_BALANCE), aliceSecretHash]);
  const extendedNote = new ExtendedNote(note, alice.address, token.address, pendingShieldsStorageSlot, receipt.txHash);
  await pxe.addNote(extendedNote);

  // Make the tokens spendable by redeeming them using the secret (converts the "pending shield note" created above
  // to a "token note")
  await tokenAlice.methods.redeem_shield(alice, ALICE_MINT_BALANCE, aliceSecret).send().wait();
  logger(`${ALICE_MINT_BALANCE} tokens were successfully minted and redeemed by Alice`);

  const balanceAfterMint = await tokenAlice.methods.balance_of_private(alice).view();
  logger(`Tokens successfully minted. New Alice's balance: ${balanceAfterMint}`);

  // We will now transfer tokens from Alice to Bob
  logger(`Transferring ${TRANSFER_AMOUNT} tokens from Alice to Bob...`);
  await tokenAlice.methods.transfer(alice, bob, TRANSFER_AMOUNT, 0).send().wait();

  // Check the new balances
  const aliceBalance = await tokenAlice.methods.balance_of_private(alice).view();
  logger(`Alice's balance ${aliceBalance}`);

  const bobBalance = await tokenBob.methods.balance_of_private(bob).view();
  logger(`Bob's balance ${bobBalance}`);
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
