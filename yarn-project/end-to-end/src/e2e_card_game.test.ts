import { AccountWallet, AztecAddress, Wallet, deployInitialSandboxAccounts } from '@aztec/aztec.js';
import { DebugLogger } from '@aztec/foundation/log';
import { CardGameContract } from '@aztec/noir-contracts/types';
import { AztecRPC } from '@aztec/types';

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
  address: bigint;
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

const GAME_ID = 42;

describe('e2e_card_game', () => {
  let aztecRpcServer: AztecRPC;
  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  let wallets: AccountWallet[];
  let firstPlayerWallet: Wallet;
  let secondPlayerWallet: Wallet;
  let thirdPlayerWallet: Wallet;

  let firstPlayer: AztecAddress;
  let secondPlayer: AztecAddress;
  let thirdPlayer: AztecAddress;

  let contract: CardGameContract;
  let contractAsSecondPlayer: CardGameContract;
  let contractAsThirdPlayer: CardGameContract;

  beforeEach(async () => {
    // Card stats are derived from the users' private keys, so to get consistent values, we set up the
    // initial sandbox accounts that always use the same private keys, instead of random ones.
    ({ aztecRpcServer, logger, teardown } = await setup(0));
    wallets = await Promise.all((await deployInitialSandboxAccounts(aztecRpcServer)).map(a => a.account.getWallet()));
    [firstPlayerWallet, secondPlayerWallet, thirdPlayerWallet] = wallets;
    [firstPlayer, secondPlayer, thirdPlayer] = wallets.map(a => a.getAddress());
    await deployContract();
  }, 100_000);

  afterEach(() => teardown());

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
    await contract.methods.buy_pack(27n).send().wait();
    const collection = await contract.methods.view_collection_cards(firstPlayer, 0).view({ from: firstPlayer });
    expect(unwrapOptions(collection)).toMatchInlineSnapshot(`
      [
        {
          "points": 18471n,
          "strength": 55863n,
        },
        {
          "points": 30024n,
          "strength": 10202n,
        },
        {
          "points": 47477n,
          "strength": 18471n,
        },
      ]
    `);
  }, 30_000);

  describe('game join', () => {
    let firstPlayerCollection: Card[];

    beforeEach(async () => {
      await Promise.all([
        contract.methods.buy_pack(27n).send().wait(),
        contractAsSecondPlayer.methods.buy_pack(27n).send().wait(),
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
      expect(unwrapOptions(collection)).toMatchInlineSnapshot(`
        [
          {
            "points": 30024n,
            "strength": 10202n,
          },
        ]
      `);

      expect((await contract.methods.view_game(GAME_ID).view({ from: firstPlayer })) as Game).toMatchObject({
        players: [
          {
            address: firstPlayer.toBigInt(),
            deck_strength: expect.anything(),
            points: 0n,
          },
          {
            address: 0n,
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
            address: firstPlayer.toBigInt(),
            deck_strength: expect.anything(),
            points: 0n,
          },
          {
            address: secondPlayer.toBigInt(),
            deck_strength: expect.anything(),
            points: 0n,
          },
        ]),
        started: true,
        finished: false,
        claimed: false,
        current_player: 0n,
      });
    }, 30_000);
  });

  describe('game play', () => {
    let firstPlayerCollection: Card[];
    let secondPlayerCollection: Card[];
    let thirdPlayerCOllection: Card[];

    beforeEach(async () => {
      await Promise.all([
        contract.methods.buy_pack(27n).send().wait(),
        contractAsSecondPlayer.methods.buy_pack(27n).send().wait(),
        contractAsThirdPlayer.methods.buy_pack(27n).send().wait(),
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
        player => playerDecks.find(playerDeckEntry => playerDeckEntry.address.toBigInt() === player)!.deck,
      );

      for (let roundIndex = 0; roundIndex < cards.length; roundIndex++) {
        for (let playerIndex = 0; playerIndex < players.length; playerIndex++) {
          const player = players[playerIndex];
          const card = cards[playerIndex][roundIndex];
          await contractFor(AztecAddress.fromBigInt(player)).methods.play_card(id, card).send().wait();
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

      const sotedByPoints = game.players.sort((a, b) => Number(b.points - a.points));
      const winner = AztecAddress.fromBigInt(sotedByPoints[0].address);
      const loser = AztecAddress.fromBigInt(sotedByPoints[1].address);

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
    }, 180_000);
  });
});
