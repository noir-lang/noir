import { createAccounts } from '@aztec/accounts/testing';
import {
  type AccountWallet,
  type AztecAddress,
  type AztecNode,
  type CheatCodes,
  type DebugLogger,
  ExtendedNote,
  Fr,
  Note,
  type PXE,
  PackedValues,
  TxExecutionRequest,
  type TxHash,
  computeSecretHash,
  deriveKeys,
} from '@aztec/aztec.js';
import { GasSettings, TxContext, computePartialAddress } from '@aztec/circuits.js';
import { InclusionProofsContract } from '@aztec/noir-contracts.js';
import { ClaimContract } from '@aztec/noir-contracts.js/Claim';
import { CrowdfundingContract } from '@aztec/noir-contracts.js/Crowdfunding';
import { TokenContract } from '@aztec/noir-contracts.js/Token';

import { jest } from '@jest/globals';

import { setup, setupPXEService } from './fixtures/utils.js';

jest.setTimeout(200_000);

// Tests crowdfunding via the Crowdfunding contract and claiming the reward token via the Claim contract
describe('e2e_crowdfunding_and_claim', () => {
  const donationTokenMetadata = {
    name: 'Donation Token',
    symbol: 'DNT',
    decimals: 18n,
  };

  const rewardTokenMetadata = {
    name: 'Reward Token',
    symbol: 'RWT',
    decimals: 18n,
  };

  let teardownA: () => Promise<void>;
  let teardownB: () => Promise<void>;

  let aztecNode: AztecNode;
  let operatorWallet: AccountWallet;
  let donorWallets: AccountWallet[];
  let wallets: AccountWallet[];
  let logger: DebugLogger;

  let donationToken: TokenContract;
  let rewardToken: TokenContract;
  let crowdfundingContract: CrowdfundingContract;
  let claimContract: ClaimContract;

  let crowdfundingSecretKey;
  let crowdfundingPublicKeysHash;
  let pxe: PXE;
  let cheatCodes: CheatCodes;
  let deadline: number; // end of crowdfunding period

  let valueNote!: any;

  const addPendingShieldNoteToPXE = async (
    wallet: AccountWallet,
    amount: bigint,
    secretHash: Fr,
    txHash: TxHash,
    address: AztecAddress,
  ) => {
    const note = new Note([new Fr(amount), secretHash]);
    const extendedNote = new ExtendedNote(
      note,
      wallet.getAddress(),
      address,
      TokenContract.storage.pending_shields.slot,
      TokenContract.notes.TransparentNote.id,
      txHash,
    );
    await wallet.addNote(extendedNote);
  };

  beforeAll(async () => {
    ({ cheatCodes, teardown: teardownA, logger, pxe, wallets, aztecNode } = await setup(3));
    operatorWallet = wallets[0];
    donorWallets = wallets.slice(1);

    // We set the deadline to a week from now
    deadline = (await cheatCodes.eth.timestamp()) + 7 * 24 * 60 * 60;

    donationToken = await TokenContract.deploy(
      operatorWallet,
      operatorWallet.getAddress(),
      donationTokenMetadata.name,
      donationTokenMetadata.symbol,
      donationTokenMetadata.decimals,
    )
      .send()
      .deployed();
    logger.info(`Donation Token deployed to ${donationToken.address}`);

    rewardToken = await TokenContract.deploy(
      operatorWallet,
      operatorWallet.getAddress(),
      rewardTokenMetadata.name,
      rewardTokenMetadata.symbol,
      rewardTokenMetadata.decimals,
    )
      .send()
      .deployed();
    logger.info(`Reward Token deployed to ${rewardToken.address}`);

    crowdfundingSecretKey = Fr.random();
    crowdfundingPublicKeysHash = deriveKeys(crowdfundingSecretKey).publicKeys.hash();

    const crowdfundingDeployment = CrowdfundingContract.deployWithPublicKeysHash(
      crowdfundingPublicKeysHash,
      operatorWallet,
      donationToken.address,
      operatorWallet.getAddress(),
      deadline,
    );
    const crowdfundingInstance = crowdfundingDeployment.getInstance();
    await pxe.registerAccount(crowdfundingSecretKey, computePartialAddress(crowdfundingInstance));
    crowdfundingContract = await crowdfundingDeployment.send().deployed();
    logger.info(`Crowdfunding contract deployed at ${crowdfundingContract.address}`);

    claimContract = await ClaimContract.deploy(operatorWallet, crowdfundingContract.address, rewardToken.address)
      .send()
      .deployed();
    logger.info(`Claim contract deployed at ${claimContract.address}`);

    await rewardToken.methods.set_minter(claimContract.address, true).send().wait();

    await mintDNTToDonors();
  });

  afterAll(async () => {
    await teardownA();
    await teardownB();
  });

  const mintDNTToDonors = async () => {
    const secret = Fr.random();
    const secretHash = computeSecretHash(secret);

    const [txReceipt1, txReceipt2] = await Promise.all([
      donationToken.withWallet(operatorWallet).methods.mint_private(1234n, secretHash).send().wait(),
      donationToken.withWallet(operatorWallet).methods.mint_private(2345n, secretHash).send().wait(),
    ]);

    await addPendingShieldNoteToPXE(
      donorWallets[0],
      1234n,
      secretHash,
      txReceipt1.txHash,
      donationToken.withWallet(operatorWallet).address,
    );
    await addPendingShieldNoteToPXE(
      donorWallets[1],
      2345n,
      secretHash,
      txReceipt2.txHash,
      donationToken.withWallet(operatorWallet).address,
    );

    await Promise.all([
      donationToken
        .withWallet(donorWallets[0])
        .methods.redeem_shield(donorWallets[0].getAddress(), 1234n, secret)
        .send()
        .wait(),
      donationToken
        .withWallet(donorWallets[1])
        .methods.redeem_shield(donorWallets[1].getAddress(), 2345n, secret)
        .send()
        .wait(),
    ]);
  };

  // Processes extended note such that it can be passed to a claim function of Claim contract
  const processExtendedNote = async (extendedNote: ExtendedNote) => {
    // TODO(#4956): Make fetching the nonce manually unnecessary
    // To be able to perform the inclusion proof we need to fetch the nonce of the value note
    const noteNonces = await pxe.getNoteNonces(extendedNote);
    expect(noteNonces?.length).toEqual(1);

    return {
      header: {
        // eslint-disable-next-line camelcase
        contract_address: extendedNote.contractAddress,
        // eslint-disable-next-line camelcase
        storage_slot: extendedNote.storageSlot,
        // eslint-disable-next-line camelcase
        note_hash_counter: 0, // set as 0 as note is not transient
        nonce: noteNonces[0],
      },
      value: extendedNote.note.items[0],
      // eslint-disable-next-line camelcase
      npk_m_hash: extendedNote.note.items[1],
      randomness: extendedNote.note.items[2],
    };
  };

  it('full donor flow', async () => {
    const donationAmount = 1000n;

    // 1) We add authwit so that the Crowdfunding contract can transfer donor's DNT
    {
      const action = donationToken
        .withWallet(donorWallets[0])
        .methods.transfer_from(donorWallets[0].getAddress(), crowdfundingContract.address, donationAmount, 0);
      const witness = await donorWallets[0].createAuthWit({ caller: crowdfundingContract.address, action });
      await donorWallets[0].addAuthWitness(witness);
    }

    // 2) We donate to the crowdfunding contract
    {
      const donateTxReceipt = await crowdfundingContract
        .withWallet(donorWallets[0])
        .methods.donate(donationAmount)
        .send()
        .wait({
          debug: true,
        });

      // Get the notes emitted by the Crowdfunding contract and check that only 1 was emitted (the value note)
      const notes = donateTxReceipt.debugInfo?.visibleIncomingNotes.filter(x =>
        x.contractAddress.equals(crowdfundingContract.address),
      );
      expect(notes!.length).toEqual(1);

      // Set the value note in a format which can be passed to claim function
      valueNote = await processExtendedNote(notes![0]);
    }

    // 3) We claim the reward token via the Claim contract
    {
      await claimContract
        .withWallet(donorWallets[0])
        .methods.claim(valueNote, donorWallets[0].getAddress())
        .send()
        .wait();
    }

    // Since the RWT is minted 1:1 with the DNT, the balance of the reward token should be equal to the donation amount
    const balanceRWT = await rewardToken.methods.balance_of_public(donorWallets[0].getAddress()).simulate();
    expect(balanceRWT).toEqual(donationAmount);

    const balanceDNTBeforeWithdrawal = await donationToken.methods
      .balance_of_private(operatorWallet.getAddress())
      .simulate();
    expect(balanceDNTBeforeWithdrawal).toEqual(0n);

    // 4) At last, we withdraw the raised funds from the crowdfunding contract to the operator's address
    await crowdfundingContract.methods.withdraw(donationAmount).send().wait();

    const balanceDNTAfterWithdrawal = await donationToken.methods
      .balance_of_private(operatorWallet.getAddress())
      .simulate();

    // Operator should have all the DNT now
    expect(balanceDNTAfterWithdrawal).toEqual(donationAmount);
  });

  it('cannot claim twice', async () => {
    // The first claim was executed in the previous test
    await expect(
      claimContract.withWallet(donorWallets[0]).methods.claim(valueNote, donorWallets[0].getAddress()).send().wait(),
    ).rejects.toThrow();
  });

  it('cannot claim without access to the nsk_app tied to the npk_m specified in the proof note', async () => {
    const donationAmount = 1000n;
    {
      const action = donationToken
        .withWallet(donorWallets[1])
        .methods.transfer_from(donorWallets[1].getAddress(), crowdfundingContract.address, donationAmount, 0);
      const witness = await donorWallets[1].createAuthWit({ caller: crowdfundingContract.address, action });
      await donorWallets[1].addAuthWitness(witness);
    }

    // 2) We donate to the crowdfunding contract

    const donateTxReceipt = await crowdfundingContract
      .withWallet(donorWallets[1])
      .methods.donate(donationAmount)
      .send()
      .wait({
        debug: true,
      });

    // Get the notes emitted by the Crowdfunding contract and check that only 1 was emitted (the value note)
    const notes = donateTxReceipt.debugInfo?.visibleIncomingNotes.filter(x =>
      x.contractAddress.equals(crowdfundingContract.address),
    );
    expect(notes!.length).toEqual(1);

    // Set the value note in a format which can be passed to claim function
    const anotherDonationNote = await processExtendedNote(notes![0]);

    // We create an unrelated pxe and wallet without access to the nsk_app that correlates to the npk_m specified in the proof note.
    let unrelatedWallet: AccountWallet;
    {
      const { pxe: pxeB, teardown: _teardown } = await setupPXEService(aztecNode!, {}, undefined, true);
      teardownB = _teardown;
      [unrelatedWallet] = await createAccounts(pxeB, 1);
      await pxeB.registerContract({
        artifact: ClaimContract.artifact,
        instance: claimContract.instance,
      });
    }

    // 3) We try to claim the reward token via the Claim contract with the unrelated wallet
    {
      await expect(
        claimContract
          .withWallet(unrelatedWallet)
          .methods.claim(anotherDonationNote, unrelatedWallet.getAddress())
          .send()
          .wait(),
      ).rejects.toThrow('Could not find key prefix.');
    }
  });

  it('cannot claim with a non-existent note', async () => {
    // We get a non-existent note by copy the value note and change the randomness to a random value
    const nonExistentNote = { ...valueNote };
    nonExistentNote.randomness = Fr.random();

    await expect(
      claimContract
        .withWallet(donorWallets[0])
        .methods.claim(nonExistentNote, donorWallets[0].getAddress())
        .send()
        .wait(),
    ).rejects.toThrow();
  });

  it('cannot claim with existing note which was not emitted by the crowdfunding contract', async () => {
    const owner = wallets[0].getAddress();

    // 1) Deploy IncludeProofs contract
    const inclusionsProofsContract = await InclusionProofsContract.deploy(wallets[0], 0n).send().deployed();

    // 2) Create a note
    let note: any;
    {
      const receipt = await inclusionsProofsContract.methods.create_note(owner, 5n).send().wait({ debug: true });
      const { visibleIncomingNotes } = receipt.debugInfo!;
      expect(visibleIncomingNotes.length).toEqual(1);
      note = await processExtendedNote(visibleIncomingNotes![0]);
    }

    // 3) Test the note was included
    await inclusionsProofsContract.methods.test_note_inclusion(owner, false, 0n, true).send().wait();

    // 4) Finally, check that the claim process fails
    await expect(
      claimContract.withWallet(donorWallets[0]).methods.claim(note, donorWallets[0].getAddress()).send().wait(),
    ).rejects.toThrow();
  });

  it('cannot withdraw as non operator', async () => {
    const donationAmount = 500n;

    // 1) We add authwit so that the Crowdfunding contract can transfer donor's DNT
    const action = donationToken
      .withWallet(donorWallets[1])
      .methods.transfer_from(donorWallets[1].getAddress(), crowdfundingContract.address, donationAmount, 0);
    const witness = await donorWallets[1].createAuthWit({ caller: crowdfundingContract.address, action });
    await donorWallets[1].addAuthWitness(witness);

    // 2) We donate to the crowdfunding contract
    await crowdfundingContract.withWallet(donorWallets[1]).methods.donate(donationAmount).send().wait({
      debug: true,
    });

    // Calling the function normally will fail as msg_sender != operator
    await expect(
      crowdfundingContract.withWallet(donorWallets[1]).methods.withdraw(donationAmount).send().wait(),
    ).rejects.toThrow('Assertion failed: Not an operator');

    // Instead, we construct a call and impersonate operator by skipping the usual account contract entrypoint...
    const call = crowdfundingContract.withWallet(donorWallets[1]).methods.withdraw(donationAmount).request();
    // ...using the withdraw fn as our entrypoint
    const entrypointPackedValues = PackedValues.fromValues(call.args);
    const request = new TxExecutionRequest(
      call.to,
      call.selector,
      entrypointPackedValues.hash,
      new TxContext(donorWallets[1].getChainId(), donorWallets[1].getVersion(), GasSettings.default()),
      [entrypointPackedValues],
      [],
    );
    // NB: Removing the msg_sender assertion from private_init will still result in a throw, as we are using
    // a non-entrypoint function (withdraw never calls context.end_setup()), meaning the min revertible counter will remain 0.
    // This does not protect fully against impersonation as the contract could just call context.end_setup() and the below would pass.
    // => the private_init msg_sender assertion is required (#7190, #7404)
    await expect(donorWallets[1].simulateTx(request, true, operatorWallet.getAddress())).rejects.toThrow(
      'Assertion failed: Users cannot set msg_sender in first call',
    );
  });

  it('cannot donate after a deadline', async () => {
    const donationAmount = 1000n;

    // 1) We add authwit so that the Crowdfunding contract can transfer donor's DNT
    {
      const action = donationToken
        .withWallet(donorWallets[1])
        .methods.transfer_from(donorWallets[1].getAddress(), crowdfundingContract.address, donationAmount, 0);
      const witness = await donorWallets[1].createAuthWit({ caller: crowdfundingContract.address, action });
      await donorWallets[1].addAuthWitness(witness);
    }

    // 2) We set next block timestamp to be after the deadline
    await cheatCodes.aztec.warp(deadline + 1);

    // 3) We donate to the crowdfunding contract
    await expect(
      crowdfundingContract.withWallet(donorWallets[1]).methods.donate(donationAmount).send().wait(),
    ).rejects.toThrow();
  });
});
