import {
  AztecNode,
  DebugLogger,
  ExtendedNote,
  Fr,
  Note,
  PXE,
  SignerlessWallet,
  TxStatus,
  Wallet,
  toBigInt,
} from '@aztec/aztec.js';
import { siloNullifier } from '@aztec/circuits.js/hash';
import { TestContract } from '@aztec/noir-contracts.js/Test';

import { setup } from './fixtures/utils.js';

describe('e2e_non_contract_account', () => {
  let aztecNode: AztecNode | undefined;
  let pxe: PXE;
  let nonContractAccountWallet: Wallet;
  let teardown: () => Promise<void>;

  let logger: DebugLogger;

  let contract: TestContract;
  let wallet: Wallet;

  beforeEach(async () => {
    ({ teardown, aztecNode, pxe, wallet, logger } = await setup(1));
    nonContractAccountWallet = new SignerlessWallet(pxe);

    logger(`Deploying L2 contract...`);
    contract = await TestContract.deploy(wallet).send().deployed();
    logger('L2 contract deployed');
  }, 100_000);

  afterEach(() => teardown());

  it('Arbitrary non-contract account can call a private function on a contract', async () => {
    const contractWithNoContractWallet = await TestContract.at(contract.address, nonContractAccountWallet);

    // Send transaction as arbitrary non-contract account
    const nullifier = new Fr(940);
    const receipt = await contractWithNoContractWallet.methods.emit_nullifier(nullifier).send().wait({ interval: 0.1 });
    expect(receipt.status).toBe(TxStatus.MINED);

    const tx = await aztecNode!.getTx(receipt.txHash);
    const expectedSiloedNullifier = siloNullifier(contract.address, nullifier);
    const siloedNullifier = tx!.newNullifiers[1];

    expect(siloedNullifier.equals(expectedSiloedNullifier)).toBeTruthy();
  }, 120_000);

  it('msg.sender is 0 when a non-contract account calls a private function on a contract', async () => {
    const contractWithNoContractWallet = await TestContract.at(contract.address, nonContractAccountWallet);

    // Send transaction as arbitrary non-contract account
    const tx = contractWithNoContractWallet.methods.emit_msg_sender().send();
    const receipt = await tx.wait({ interval: 0.1 });
    expect(receipt.status).toBe(TxStatus.MINED);

    const logs = (await tx.getUnencryptedLogs()).logs;
    expect(logs.length).toBe(1);

    const msgSender = toBigInt(logs[0].log.data);
    expect(msgSender).toBe(0n);
  }, 120_000);

  // Note: This test doesn't really belong here as it doesn't have anything to do with non-contract accounts. I needed
  // to test the FieldNote functionality and it doesn't really fit anywhere else. Creating a separate e2e test for this
  // seems wasteful. Move this test if a better place is found.
  it('can set and get a constant', async () => {
    const value = 123n;

    const receipt = await contract.methods.set_constant(value).send().wait({ interval: 0.1 });

    // check that 1 commitment was created
    const tx = await pxe.getTx(receipt.txHash);
    const nonZeroCommitments = tx?.newNoteHashes.filter(c => c.value > 0);
    expect(nonZeroCommitments?.length).toBe(1);

    // Add the note
    const note = new Note([new Fr(value)]);
    const storageSlot = new Fr(1);
    const noteTypeId = new Fr(7010510110810078111116101n); // FieldNote

    const extendedNote = new ExtendedNote(
      note,
      wallet.getCompleteAddress().address,
      contract.address,
      storageSlot,
      noteTypeId,
      receipt.txHash,
    );
    await wallet.addNote(extendedNote);

    expect(await contract.methods.get_constant().view()).toEqual(value);
  });
});
