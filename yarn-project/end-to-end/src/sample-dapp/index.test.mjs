import { createAccount } from '@aztec/accounts/testing';
import { Contract, ExtendedNote, Fr, Note, computeSecretHash, createPXEClient, waitForPXE } from '@aztec/aztec.js';
import { TokenContractArtifact } from '@aztec/noir-contracts.js/Token';

const { PXE_URL = 'http://localhost:8080', ETHEREUM_HOST = 'http://localhost:8545' } = process.env;

describe('token', () => {
  // docs:start:setup
  let owner, recipient, token;
  beforeAll(async () => {
    const pxe = createPXEClient(PXE_URL);
    await waitForPXE(pxe);
    owner = await createAccount(pxe);
    recipient = await createAccount(pxe);

    token = await Contract.deploy(owner, TokenContractArtifact, [owner.getCompleteAddress(), 'TokenName', 'TKN', 18])
      .send()
      .deployed();

    const initialBalance = 20n;
    const secret = Fr.random();
    const secretHash = await computeSecretHash(secret);
    const receipt = await token.methods.mint_private(initialBalance, secretHash).send().wait();

    const storageSlot = token.artifact.storageLayout['pending_shields'].slot;
    const noteTypeId = token.artifact.notes['TransparentNote'].id;
    const note = new Note([new Fr(initialBalance), secretHash]);
    const extendedNote = new ExtendedNote(
      note,
      owner.getAddress(),
      token.address,
      storageSlot,
      noteTypeId,
      receipt.txHash,
    );
    await pxe.addNote(extendedNote);

    await token.methods.redeem_shield({ address: owner.getAddress() }, initialBalance, secret).send().wait();
  }, 120_000);
  // docs:end:setup

  // docs:start:test
  it('increases recipient funds on transfer', async () => {
    expect(await token.methods.balance_of_private(recipient.getAddress()).simulate()).toEqual(0n);
    await token.methods.transfer(owner.getAddress(), recipient.getAddress(), 20n, 0).send().wait();
    expect(await token.methods.balance_of_private(recipient.getAddress()).simulate()).toEqual(20n);
  }, 30_000);
  // docs:end:test
});
