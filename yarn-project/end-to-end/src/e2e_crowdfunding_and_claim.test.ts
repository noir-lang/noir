import {
  type AccountWallet,
  type AztecAddress,
  type CheatCodes,
  type DebugLogger,
  ExtendedNote,
  Fr,
  GrumpkinScalar,
  Note,
  type PXE,
  type TxHash,
  computeMessageSecretHash,
  generatePublicKey,
} from '@aztec/aztec.js';
import { computePartialAddress } from '@aztec/circuits.js';
import { InclusionProofsContract } from '@aztec/noir-contracts.js';
import { ClaimContract } from '@aztec/noir-contracts.js/Claim';
import { CrowdfundingContract } from '@aztec/noir-contracts.js/Crowdfunding';
import { TokenContract } from '@aztec/noir-contracts.js/Token';

import { jest } from '@jest/globals';

import { setup } from './fixtures/utils.js';

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

  let teardown: () => Promise<void>;
  let operatorWallet: AccountWallet;
  let donorWallets: AccountWallet[];
  let wallets: AccountWallet[];
  let logger: DebugLogger;

  let donationToken: TokenContract;
  let rewardToken: TokenContract;
  let crowdfundingContract: CrowdfundingContract;
  let claimContract: ClaimContract;

  let crowdfundingPrivateKey;
  let crowdfundingPublicKey;
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
    const storageSlot = new Fr(5); // The storage slot of `pending_shields` is 5.
    const noteTypeId = new Fr(84114971101151129711410111011678111116101n); // TransparentNote
    const note = new Note([new Fr(amount), secretHash]);
    const extendedNote = new ExtendedNote(note, wallet.getAddress(), address, storageSlot, noteTypeId, txHash);
    await wallet.addNote(extendedNote);
  };

  beforeAll(async () => {
    ({ cheatCodes, teardown, logger, pxe, wallets } = await setup(3));
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
    logger(`Donation Token deployed to ${donationToken.address}`);

    rewardToken = await TokenContract.deploy(
      operatorWallet,
      operatorWallet.getAddress(),
      rewardTokenMetadata.name,
      rewardTokenMetadata.symbol,
      rewardTokenMetadata.decimals,
    )
      .send()
      .deployed();
    logger(`Reward Token deployed to ${rewardToken.address}`);

    crowdfundingPrivateKey = GrumpkinScalar.random();
    crowdfundingPublicKey = generatePublicKey(crowdfundingPrivateKey);

    const crowdfundingDeployment = CrowdfundingContract.deployWithPublicKey(
      crowdfundingPublicKey,
      operatorWallet,
      donationToken.address,
      operatorWallet.getAddress(),
      deadline,
    );
    const crowdfundingInstance = crowdfundingDeployment.getInstance();
    await pxe.registerAccount(crowdfundingPrivateKey, computePartialAddress(crowdfundingInstance));
    crowdfundingContract = await crowdfundingDeployment.send().deployed();
    logger(`Crowdfunding contract deployed at ${crowdfundingContract.address}`);

    claimContract = await ClaimContract.deploy(operatorWallet, crowdfundingContract.address, rewardToken.address)
      .send()
      .deployed();
    logger(`Claim contract deployed at ${claimContract.address}`);

    await rewardToken.methods.set_minter(claimContract.address, true).send().wait();

    await mintDNTToDonors();
  });

  afterAll(() => teardown());

  const mintDNTToDonors = async () => {
    const secret = Fr.random();
    const secretHash = computeMessageSecretHash(secret);

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
        is_transient: false,
        nonce: noteNonces[0],
      },
      value: extendedNote.note.items[0],
      owner: extendedNote.note.items[1],
      randomness: extendedNote.note.items[2],
    };
  };

  it('full donor flow', async () => {
    const donationAmount = 1000n;

    // 1) We add authwit so that the Crowdfunding contract can transfer donor's DNT
    {
      const action = donationToken
        .withWallet(donorWallets[0])
        .methods.transfer(donorWallets[0].getAddress(), crowdfundingContract.address, donationAmount, 0);
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
      const notes = donateTxReceipt.debugInfo?.visibleNotes.filter(x =>
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

  it('cannot claim with a different address than the one that donated', async () => {
    const donationAmount = 1000n;
    {
      const action = donationToken
        .withWallet(donorWallets[1])
        .methods.transfer(donorWallets[1].getAddress(), crowdfundingContract.address, donationAmount, 0);
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
    const notes = donateTxReceipt.debugInfo?.visibleNotes.filter(x =>
      x.contractAddress.equals(crowdfundingContract.address),
    );
    expect(notes!.length).toEqual(1);

    // Set the value note in a format which can be passed to claim function
    const anotherDonationNote = await processExtendedNote(notes![0]);

    // 3) We claim the reward token via the Claim contract
    {
      await expect(
        claimContract
          .withWallet(donorWallets[0])
          .methods.claim(anotherDonationNote, donorWallets[1].getAddress())
          .send()
          .wait(),
      ).rejects.toThrow('Note does not belong to the sender');
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
      const { visibleNotes } = receipt.debugInfo!;
      expect(visibleNotes.length).toEqual(1);
      note = await processExtendedNote(visibleNotes![0]);
    }

    // 3) Test the note was included
    await inclusionsProofsContract.methods.test_note_inclusion(owner, false, 0n, true).send().wait();

    // 4) Finally, check that the claim process fails
    await expect(
      claimContract.withWallet(donorWallets[0]).methods.claim(note, donorWallets[0].getAddress()).send().wait(),
    ).rejects.toThrow();
  });

  it('cannot donate after a deadline', async () => {
    const donationAmount = 1000n;

    // 1) We add authwit so that the Crowdfunding contract can transfer donor's DNT
    {
      const action = donationToken
        .withWallet(donorWallets[1])
        .methods.transfer(donorWallets[1].getAddress(), crowdfundingContract.address, donationAmount, 0);
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
