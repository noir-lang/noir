import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { INITIAL_TEST_ENCRYPTION_KEYS } from '@aztec/accounts/testing';
import {
  AccountWallet,
  AztecAddress,
  DebugLogger,
  GrumpkinScalar,
  PXE,
  Wallet,
  generatePublicKey,
} from '@aztec/aztec.js';
import { computeNullifierSecretKey, computeSiloedNullifierSecretKey } from '@aztec/circuits.js';
import { toBufferLE } from '@aztec/foundation/bigint-buffer';
import { sha256 } from '@aztec/foundation/crypto';
import { CardGameContract } from '@aztec/noir-contracts/CardGame';

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

interface NoirOption<T> {
  _is_some: boolean;
  _value: T;
}

function unwrapOptions<T>(options: NoirOption<T>[]): T[] {
  return options.filter((option: any) => option._is_some).map((option: any) => option._value);
}

// Game settings.
const PACK_CARDS = 3;
const GAME_ID = 42;

const PLAYER_ENCRYPTION_KEYS = INITIAL_TEST_ENCRYPTION_KEYS;

describe('e2e_card_game', () => {
  let pxe: PXE;
  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  let wallets: AccountWallet[];
  let nullifierSecretKeys: GrumpkinScalar[];

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
    const nullifierKey = nullifierSecretKeys[accountIndex];
    const secret = computeSiloedNullifierSecretKey(nullifierKey, contract.address);
    const mix = secret.high.add(secret.low).toBigInt() + seed;
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

    const toRegister = PLAYER_ENCRYPTION_KEYS.filter(key => {
      const publicKey = generatePublicKey(key);
      return (
        preRegisteredAccounts.find(preRegisteredAccount => {
          return preRegisteredAccount.publicKey.equals(publicKey);
        }) == undefined
      );
    });

    for (let i = 0; i < toRegister.length; i++) {
      logger(`Deploying account contract ${i}/${toRegister.length}...`);
      const encryptionPrivateKey = toRegister[i];
      const account = getSchnorrAccount(pxe, encryptionPrivateKey, GrumpkinScalar.random());
      const wallet = await account.waitDeploy({ interval: 0.1 });
      wallets.push(wallet);
    }
    logger('Account contracts deployed');

    [firstPlayerWallet, secondPlayerWallet, thirdPlayerWallet] = wallets;
    [firstPlayer, secondPlayer, thirdPlayer] = wallets.map(a => a.getAddress());

    nullifierSecretKeys = PLAYER_ENCRYPTION_KEYS.map(pk => computeNullifierSecretKey(pk));
  }, 100_000);

  beforeEach(async () => {
    await deployContract();
  });

  afterAll(() => teardown());

  const deployContract = async () => {
    logger(`Deploying L2 contract...`);
    contract = await CardGameContract.deploy(firstPlayerWallet).send().deployed();
    contractAsSecondPlayer = contract.withWallet(secondPlayerWallet);
    contractAsThirdPlayer = contract.withWallet(thirdPlayerWallet);
    logger(`L2 contract deployed at ${contract.address}`);
  };

  const getWallet = (address: AztecAddress) => wallets.find(w => w.getAddress().equals(address))!;
  const contractFor = (address: AztecAddress) => contract.withWallet(getWallet(address))!;

  it('should be able to buy packs', async () => {
    const seed = 27n;
    await contract.methods.buy_pack(seed).send().wait();
    const collection = await contract.methods.view_collection_cards(firstPlayer, 0).view({ from: firstPlayer });
    const expected = getPackedCards(0, seed);
    expect(unwrapOptions(collection)).toMatchObject(expected);
  }, 30_000);

  describe('game join', () => {
    const seed = 27n;
    let firstPlayerCollection: Card[];

    beforeEach(async () => {
      await Promise.all([
        contract.methods.buy_pack(seed).send().wait(),
        contractAsSecondPlayer.methods.buy_pack(seed).send().wait(),
      ]);
      firstPlayerCollection = unwrapOptions(
        await contract.methods.view_collection_cards(firstPlayer, 0).view({ from: firstPlayer }),
      );
    }, 30_000);

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
      ).rejects.toThrow(/Card not found/);

      const collection = await contract.methods.view_collection_cards(firstPlayer, 0).view({ from: firstPlayer });
      expect(unwrapOptions(collection)).toHaveLength(1);
      expect(unwrapOptions(collection)).toMatchObject([firstPlayerCollection[1]]);

      expect((await contract.methods.view_game(GAME_ID).view({ from: firstPlayer })) as Game).toMatchObject({
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
    }, 30_000);

    it('should start games', async () => {
      const secondPlayerCollection = unwrapOptions(
        (await contract.methods
          .view_collection_cards(secondPlayer, 0)
          .view({ from: secondPlayer })) as NoirOption<Card>[],
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

      expect((await contract.methods.view_game(GAME_ID).view({ from: firstPlayer })) as Game).toMatchObject({
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
    }, 360_000);
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

      firstPlayerCollection = unwrapOptions(
        await contract.methods.view_collection_cards(firstPlayer, 0).view({ from: firstPlayer }),
      );

      secondPlayerCollection = unwrapOptions(
        await contract.methods.view_collection_cards(secondPlayer, 0).view({ from: secondPlayer }),
      );

      thirdPlayerCOllection = unwrapOptions(
        await contract.methods.view_collection_cards(thirdPlayer, 0).view({ from: thirdPlayer }),
      );
    }, 60_000);

    async function joinGame(playerWallet: Wallet, cards: Card[], id = GAME_ID) {
      await contract.withWallet(playerWallet).methods.join_game(id, cards.map(cardToField)).send().wait();
    }

    async function playGame(playerDecks: { address: AztecAddress; deck: Card[] }[], id = GAME_ID) {
      const initialGameState = (await contract.methods.view_game(id).view({ from: firstPlayer })) as Game;
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

      const finalGameState = (await contract.methods.view_game(id).view({ from: firstPlayer })) as Game;

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

      const winnerCollection = unwrapOptions(
        (await contract.methods.view_collection_cards(winner, 0).view({ from: winner })) as NoirOption<Card>[],
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
    }, 360_000);
  });
});
