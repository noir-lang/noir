import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { INITIAL_TEST_SECRET_KEYS } from '@aztec/accounts/testing';
import {
  type AccountWallet,
  AztecAddress,
  type DebugLogger,
  GrumpkinScalar,
  type PXE,
  type Wallet,
  computeAppNullifierSecretKey,
  deriveKeys,
  deriveMasterNullifierSecretKey,
} from '@aztec/aztec.js';
import { toBufferLE } from '@aztec/foundation/bigint-buffer';
import { sha256 } from '@aztec/foundation/crypto';
import { CardGameContract } from '@aztec/noir-contracts.js/CardGame';

import { jest } from '@jest/globals';

import { setup } from './fixtures/utils.js';

/* eslint-disable camelcase */

interface Card {
  points: bigint;
  strength: bigint;
}

const cardToField = (card: Card): bigint => {
  return card.strength + card.points * 65536n;
};

interface PlayerGameEntry {
  address: AztecAddress;
  deck_strength: bigint;
  points: bigint;
}

interface Game {
  players: PlayerGameEntry[];
  rounds_cards: Card[];
  started: boolean;
  finished: boolean;
  claimed: boolean;
  current_player: bigint;
  current_round: bigint;
}

interface NoirBoundedVec<T> {
  storage: T[];
  len: bigint;
}

function boundedVecToArray<T>(boundedVec: NoirBoundedVec<T>): T[] {
  return boundedVec.storage.slice(0, Number(boundedVec.len));
}

// Game settings.
const PACK_CARDS = 3;
const GAME_ID = 42;

const PLAYER_SECRET_KEYS = INITIAL_TEST_SECRET_KEYS;

const TIMEOUT = 600_000;

describe('e2e_card_game', () => {
  jest.setTimeout(TIMEOUT);

  let pxe: PXE;
  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  let wallets: AccountWallet[];
  let masterNullifierSecretKeys: GrumpkinScalar[];

  let firstPlayerWallet: Wallet;
  let secondPlayerWallet: Wallet;
  let thirdPlayerWallet: Wallet;

  let firstPlayer: AztecAddress;
  let secondPlayer: AztecAddress;
  let thirdPlayer: AztecAddress;

  let contract: CardGameContract;
  let contractAsSecondPlayer: CardGameContract;
  let contractAsThirdPlayer: CardGameContract;

  const getPackedCards = (accountIndex: number, seed: bigint): Card[] => {
    // First we get the app nullifier secret key for the account
    const masterNullifierSecretKey = masterNullifierSecretKeys[accountIndex];
    const appNullifierSecretKey = computeAppNullifierSecretKey(masterNullifierSecretKey, contract.address);
    // Then we compute the mix from it and hash it to get the random bytes the same way as in the contract
    const mix = appNullifierSecretKey.toBigInt() + seed;
    const randomBytes = sha256(toBufferLE(mix, 32));
    const cards: Card[] = [];
    for (let i = 0; i < PACK_CARDS; ++i) {
      cards.push({
        strength: BigInt(randomBytes.readUint8(i) + randomBytes.readUint8(i + 1) * 256),
        points: BigInt(randomBytes.readUint8(i + 2) + randomBytes.readUint8(i + 3) * 256),
      });
    }
    return cards;
  };

  beforeAll(async () => {
    ({ pxe, logger, teardown, wallets } = await setup(0));

    const preRegisteredAccounts = await pxe.getRegisteredAccounts();

    const secretKeysToRegister = INITIAL_TEST_SECRET_KEYS.filter(key => {
      const publicKey = deriveKeys(key).publicKeys.masterIncomingViewingPublicKey;
      return (
        preRegisteredAccounts.find(preRegisteredAccount => {
          return preRegisteredAccount.publicKeys.masterIncomingViewingPublicKey.equals(publicKey);
        }) == undefined
      );
    });

    for (let i = 0; i < secretKeysToRegister.length; i++) {
      logger.info(`Deploying account contract ${i}/${secretKeysToRegister.length}...`);
      const encryptionPrivateKey = secretKeysToRegister[i];
      const account = getSchnorrAccount(pxe, encryptionPrivateKey, GrumpkinScalar.random());
      const wallet = await account.waitSetup({ interval: 0.1 });
      wallets.push(wallet);
    }
    logger.info('Account contracts deployed');

    [firstPlayerWallet, secondPlayerWallet, thirdPlayerWallet] = wallets;
    [firstPlayer, secondPlayer, thirdPlayer] = wallets.map(a => a.getAddress());

    masterNullifierSecretKeys = PLAYER_SECRET_KEYS.map(sk => deriveMasterNullifierSecretKey(sk));
  });

  beforeEach(async () => {
    await deployContract();
  });

  afterAll(() => teardown());

  const deployContract = async () => {
    logger.debug(`Deploying L2 contract...`);
    contract = await CardGameContract.deploy(firstPlayerWallet).send().deployed();
    contractAsSecondPlayer = contract.withWallet(secondPlayerWallet);
    contractAsThirdPlayer = contract.withWallet(thirdPlayerWallet);
    logger.info(`L2 contract deployed at ${contract.address}`);
  };

  const getWallet = (address: AztecAddress) => wallets.find(w => w.getAddress().equals(address))!;
  const contractFor = (address: AztecAddress) => contract.withWallet(getWallet(address))!;

  it('should be able to buy packs', async () => {
    const seed = 27n;
    // docs:start:send_tx
    await contract.methods.buy_pack(seed).send().wait();
    // docs:end:send_tx
    const collection = await contract.methods.view_collection_cards(firstPlayer, 0).simulate({ from: firstPlayer });
    const expected = getPackedCards(0, seed);
    expect(boundedVecToArray(collection)).toMatchObject(expected);
  });

  describe('game join', () => {
    const seed = 27n;
    let firstPlayerCollection: Card[];

    beforeEach(async () => {
      await Promise.all([
        contract.methods.buy_pack(seed).send().wait(),
        contractAsSecondPlayer.methods.buy_pack(seed).send().wait(),
      ]);
      firstPlayerCollection = boundedVecToArray(
        await contract.methods.view_collection_cards(firstPlayer, 0).simulate({ from: firstPlayer }),
      );
    });

    it('should be able to join games', async () => {
      await contract.methods
        .join_game(GAME_ID, [cardToField(firstPlayerCollection[0]), cardToField(firstPlayerCollection[2])])
        .send()
        .wait();

      await expect(
        contractAsSecondPlayer.methods
          .join_game(GAME_ID, [cardToField(firstPlayerCollection[0]), cardToField(firstPlayerCollection[1])])
          .send()
          .wait(),
      ).rejects.toThrow(`Assertion failed: Cannot return zero notes`);

      const collection = await contract.methods.view_collection_cards(firstPlayer, 0).simulate({ from: firstPlayer });
      expect(boundedVecToArray(collection)).toHaveLength(1);
      expect(boundedVecToArray(collection)).toMatchObject([firstPlayerCollection[1]]);

      expect((await contract.methods.view_game(GAME_ID).simulate({ from: firstPlayer })) as Game).toMatchObject({
        players: [
          {
            address: firstPlayer,
            deck_strength: expect.anything(),
            points: 0n,
          },
          {
            address: AztecAddress.ZERO,
            deck_strength: 0n,
            points: 0n,
          },
        ],
        started: false,
        finished: false,
        claimed: false,
        current_player: 0n,
      });
    });

    it('should start games', async () => {
      const secondPlayerCollection = boundedVecToArray(
        (await contract.methods
          .view_collection_cards(secondPlayer, 0)
          .simulate({ from: secondPlayer })) as NoirBoundedVec<Card>,
      );

      await Promise.all([
        contract.methods
          .join_game(GAME_ID, [cardToField(firstPlayerCollection[0]), cardToField(firstPlayerCollection[2])])
          .send()
          .wait(),
        contractAsSecondPlayer.methods
          .join_game(GAME_ID, [cardToField(secondPlayerCollection[0]), cardToField(secondPlayerCollection[2])])
          .send()
          .wait(),
      ]);

      await contract.methods.start_game(GAME_ID).send().wait();

      expect((await contract.methods.view_game(GAME_ID).simulate({ from: firstPlayer })) as Game).toMatchObject({
        players: expect.arrayContaining([
          {
            address: firstPlayer,
            deck_strength: expect.anything(),
            points: 0n,
          },
          {
            address: secondPlayer,
            deck_strength: expect.anything(),
            points: 0n,
          },
        ]),
        started: true,
        finished: false,
        claimed: false,
        current_player: 0n,
      });
    });
  });

  describe('game play', () => {
    let firstPlayerCollection: Card[];
    let secondPlayerCollection: Card[];
    let thirdPlayerCOllection: Card[];

    beforeEach(async () => {
      const seed = 27n;
      await Promise.all([
        contract.methods.buy_pack(seed).send().wait(),
        contractAsSecondPlayer.methods.buy_pack(seed).send().wait(),
        contractAsThirdPlayer.methods.buy_pack(seed).send().wait(),
      ]);

      firstPlayerCollection = boundedVecToArray(
        await contract.methods.view_collection_cards(firstPlayer, 0).simulate({ from: firstPlayer }),
      );

      secondPlayerCollection = boundedVecToArray(
        await contract.methods.view_collection_cards(secondPlayer, 0).simulate({ from: secondPlayer }),
      );

      thirdPlayerCOllection = boundedVecToArray(
        await contract.methods.view_collection_cards(thirdPlayer, 0).simulate({ from: thirdPlayer }),
      );
    });

    async function joinGame(playerWallet: Wallet, cards: Card[], id = GAME_ID) {
      await contract.withWallet(playerWallet).methods.join_game(id, cards.map(cardToField)).send().wait();
    }

    async function playGame(playerDecks: { address: AztecAddress; deck: Card[] }[], id = GAME_ID) {
      const initialGameState = (await contract.methods.view_game(id).simulate({ from: firstPlayer })) as Game;
      const players = initialGameState.players.map(player => player.address);
      const cards = players.map(
        player => playerDecks.find(playerDeckEntry => playerDeckEntry.address.equals(player))!.deck,
      );

      for (let roundIndex = 0; roundIndex < cards.length; roundIndex++) {
        for (let playerIndex = 0; playerIndex < players.length; playerIndex++) {
          const player = players[playerIndex];
          const card = cards[playerIndex][roundIndex];
          await contractFor(player).methods.play_card(id, card).send().wait();
        }
      }

      const finalGameState = (await contract.methods.view_game(id).simulate({ from: firstPlayer })) as Game;

      expect(finalGameState.finished).toBe(true);
      return finalGameState;
    }

    it('should play a game, claim the winned cards and play another match with winned cards', async () => {
      const firstPlayerGameDeck = [firstPlayerCollection[0], firstPlayerCollection[2]];
      const secondPlayerGameDeck = [secondPlayerCollection[0], secondPlayerCollection[2]];
      await Promise.all([
        joinGame(firstPlayerWallet, firstPlayerGameDeck),
        joinGame(secondPlayerWallet, secondPlayerGameDeck),
      ]);
      await contract.methods.start_game(GAME_ID).send().wait();

      let game = await playGame([
        { address: firstPlayer, deck: firstPlayerGameDeck },
        { address: secondPlayer, deck: secondPlayerGameDeck },
      ]);

      const sortedByPoints = game.players.sort((a, b) => Number(b.points - a.points));
      const winner = sortedByPoints[0].address;
      const loser = sortedByPoints[1].address;

      await expect(
        contractFor(loser).methods.claim_cards(GAME_ID, game.rounds_cards.map(cardToField)).send().wait(),
      ).rejects.toThrow(/Not the winner/);

      await contractFor(winner).methods.claim_cards(GAME_ID, game.rounds_cards.map(cardToField)).send().wait();

      const winnerCollection = boundedVecToArray(
        (await contract.methods.view_collection_cards(winner, 0).simulate({ from: winner })) as NoirBoundedVec<Card>,
      );

      const winnerGameDeck = [winnerCollection[0], winnerCollection[3]];
      const thirdPlayerGameDeck = [thirdPlayerCOllection[0], thirdPlayerCOllection[2]];

      await Promise.all([
        joinGame(getWallet(winner), winnerGameDeck, GAME_ID + 1),
        joinGame(thirdPlayerWallet, thirdPlayerGameDeck, GAME_ID + 1),
      ]);

      await contractFor(winner)
        .methods.start_game(GAME_ID + 1)
        .send()
        .wait();

      game = await playGame(
        [
          { address: winner, deck: winnerGameDeck },
          { address: thirdPlayer, deck: thirdPlayerGameDeck },
        ],
        GAME_ID + 1,
      );

      expect(game.finished).toBe(true);
    });
  });
});
