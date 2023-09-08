import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { AztecAddress, Wallet } from '@aztec/aztec.js';
import { DebugLogger } from '@aztec/foundation/log';
import { CardGameContract } from '@aztec/noir-contracts/types';
import { AztecRPC, CompleteAddress } from '@aztec/types';

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
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;
  let wallet: Wallet;
  let logger: DebugLogger;
  let firstPlayer: AztecAddress;
  let secondPlayer: AztecAddress;
  let thirdPlayer: AztecAddress;

  let contract: CardGameContract;

  beforeEach(async () => {
    let accounts: CompleteAddress[];
    ({ aztecNode, aztecRpcServer, accounts, wallet, logger } = await setup(3));
    firstPlayer = accounts[0].address;
    secondPlayer = accounts[1].address;
    thirdPlayer = accounts[2].address;
    await deployContract();
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
    if (aztecRpcServer instanceof AztecRPCServer) {
      await aztecRpcServer?.stop();
    }
  });

  const deployContract = async () => {
    logger(`Deploying L2 contract...`);
    contract = await CardGameContract.deploy(wallet).send().deployed();
    logger(`L2 contract deployed at ${contract.address}`);
  };

  const firstPlayerCollection: Card[] = [
    {
      points: 45778n,
      strength: 7074n,
    },
    {
      points: 60338n,
      strength: 53787n,
    },
    {
      points: 13035n,
      strength: 45778n,
    },
  ];

  it('should be able to buy packs', async () => {
    await contract.methods.buy_pack(27n).send({ origin: firstPlayer }).wait();
    const collection = await contract.methods.view_collection_cards(firstPlayer, 0).view({ from: firstPlayer });
    expect(unwrapOptions(collection)).toEqual(firstPlayerCollection);
  }, 30_000);

  describe('game join', () => {
    beforeEach(async () => {
      await Promise.all([
        contract.methods.buy_pack(27n).send({ origin: firstPlayer }).wait(),
        contract.methods.buy_pack(27n).send({ origin: secondPlayer }).wait(),
      ]);
    }, 30_000);

    it('should be able to join games', async () => {
      await contract.methods
        .join_game(GAME_ID, [cardToField(firstPlayerCollection[0]), cardToField(firstPlayerCollection[2])])
        .send({ origin: firstPlayer })
        .wait();

      await expect(
        contract.methods
          .join_game(GAME_ID, [cardToField(firstPlayerCollection[0]), cardToField(firstPlayerCollection[1])])
          .send({ origin: secondPlayer })
          .wait(),
      ).rejects.toThrow(/Card not found/);

      const collection = await contract.methods.view_collection_cards(firstPlayer, 0).view({ from: firstPlayer });
      expect(unwrapOptions(collection)).toEqual([
        {
          points: 60338n,
          strength: 53787n,
        },
      ]);

      expect((await contract.methods.view_game(GAME_ID).view({ from: firstPlayer })) as Game).toMatchObject({
        players: [
          {
            address: firstPlayer.toBigInt(),
            deck_strength: 52852n,
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
          .send({ origin: firstPlayer })
          .wait(),
        contract.methods
          .join_game(GAME_ID, [cardToField(secondPlayerCollection[0]), cardToField(secondPlayerCollection[2])])
          .send({ origin: secondPlayer })
          .wait(),
      ]);

      await contract.methods.start_game(GAME_ID).send({ origin: firstPlayer }).wait();

      expect((await contract.methods.view_game(GAME_ID).view({ from: firstPlayer })) as Game).toMatchObject({
        players: expect.arrayContaining([
          {
            address: firstPlayer.toBigInt(),
            deck_strength: 52852n,
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
    let secondPlayerCollection: Card[];
    let thirdPlayerCOllection: Card[];

    beforeEach(async () => {
      await Promise.all([
        contract.methods.buy_pack(27n).send({ origin: firstPlayer }).wait(),
        contract.methods.buy_pack(27n).send({ origin: secondPlayer }).wait(),
        contract.methods.buy_pack(27n).send({ origin: thirdPlayer }).wait(),
      ]);

      secondPlayerCollection = unwrapOptions(
        await contract.methods.view_collection_cards(secondPlayer, 0).view({ from: secondPlayer }),
      );

      thirdPlayerCOllection = unwrapOptions(
        await contract.methods.view_collection_cards(thirdPlayer, 0).view({ from: thirdPlayer }),
      );
    }, 60_000);

    async function joinGame(playerAddress: AztecAddress, cards: Card[], id = GAME_ID) {
      await contract.methods.join_game(id, cards.map(cardToField)).send({ origin: playerAddress }).wait();
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
          await contract.methods
            .play_card(id, card)
            .send({ origin: AztecAddress.fromBigInt(player) })
            .wait();
        }
      }

      const finalGameState = (await contract.methods.view_game(id).view({ from: firstPlayer })) as Game;

      expect(finalGameState.finished).toBe(true);
      return finalGameState;
    }

    it('should play a game, claim the winned cards and play another match with winned cards', async () => {
      const firstPlayerGameDeck = [firstPlayerCollection[0], firstPlayerCollection[2]];
      const secondPlayerGameDeck = [secondPlayerCollection[0], secondPlayerCollection[2]];
      await Promise.all([joinGame(firstPlayer, firstPlayerGameDeck), joinGame(secondPlayer, secondPlayerGameDeck)]);
      await contract.methods.start_game(GAME_ID).send({ origin: firstPlayer }).wait();

      let game = await playGame([
        { address: firstPlayer, deck: firstPlayerGameDeck },
        { address: secondPlayer, deck: secondPlayerGameDeck },
      ]);

      const sotedByPoints = game.players.sort((a, b) => Number(b.points - a.points));
      const winner = AztecAddress.fromBigInt(sotedByPoints[0].address);
      const loser = AztecAddress.fromBigInt(sotedByPoints[1].address);

      await expect(
        contract.methods.claim_cards(GAME_ID, game.rounds_cards.map(cardToField)).send({ origin: loser }).wait(),
      ).rejects.toThrow(/Not the winner/);

      await contract.methods.claim_cards(GAME_ID, game.rounds_cards.map(cardToField)).send({ origin: winner }).wait();

      const winnerCollection = unwrapOptions(
        (await contract.methods.view_collection_cards(winner, 0).view({ from: winner })) as NoirOption<Card>[],
      );

      const winnerGameDeck = [winnerCollection[0], winnerCollection[3]];
      const thirdPlayerGameDeck = [thirdPlayerCOllection[0], thirdPlayerCOllection[2]];

      await Promise.all([
        joinGame(winner, winnerGameDeck, GAME_ID + 1),
        joinGame(thirdPlayer, thirdPlayerGameDeck, GAME_ID + 1),
      ]);

      await contract.methods
        .start_game(GAME_ID + 1)
        .send({ origin: winner })
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
