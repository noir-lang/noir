import { Comparator, Fr, Wallet } from '@aztec/aztec.js';
import { DocsExampleContract } from '@aztec/noir-contracts';

import { setup } from './fixtures/utils.js';

interface NoirOption<T> {
  _is_some: boolean;
  _value: T;
}

function unwrapOptions<T>(options: NoirOption<T>[]): T[] {
  return options.filter((option: any) => option._is_some).map((option: any) => option._value);
}

describe('e2e_note_getter', () => {
  let wallet: Wallet;

  let teardown: () => Promise<void>;
  let contract: DocsExampleContract;

  beforeAll(async () => {
    ({ teardown, wallet } = await setup());
    contract = await DocsExampleContract.deploy(wallet).send().deployed();
    // sets card value to 1 and leader to sender.
    await contract.methods.initialize_private(Fr.random(), 1).send().wait();
  }, 25_000);

  afterAll(() => teardown());

  it('inserts notes from 0-9, then makes multiple queries specifying the total suite of comparators', async () => {
    const numbers = [...Array(10).keys()];
    await Promise.all(numbers.map(number => contract.methods.insert_note(number).send().wait()));
    await contract.methods.insert_note(5).send().wait();

    const [returnEq, returnNeq, returnLt, returnGt, returnLte, returnGte] = await Promise.all([
      contract.methods.read_note(5, Comparator.EQ).view(),
      contract.methods.read_note(5, Comparator.NEQ).view(),
      contract.methods.read_note(5, Comparator.LT).view(),
      contract.methods.read_note(5, Comparator.GT).view(),
      contract.methods.read_note(5, Comparator.LTE).view(),
      contract.methods.read_note(5, Comparator.GTE).view(),
    ]);

    expect(unwrapOptions(returnEq).map(({ points, randomness }: any) => ({ points, randomness }))).toStrictEqual([
      { points: 5n, randomness: 1n },
      { points: 5n, randomness: 1n },
    ]);

    expect(
      unwrapOptions(returnNeq)
        .map(({ points, randomness }: any) => ({ points, randomness }))
        .sort((a: any, b: any) => (a.points > b.points ? 1 : -1)),
    ).toStrictEqual(
      [
        { points: 0n, randomness: 1n },
        { points: 1n, randomness: 1n },
        { points: 7n, randomness: 1n },
        { points: 9n, randomness: 1n },
        { points: 2n, randomness: 1n },
        { points: 6n, randomness: 1n },
        { points: 8n, randomness: 1n },
        { points: 4n, randomness: 1n },
        { points: 3n, randomness: 1n },
      ].sort((a: any, b: any) => (a.points > b.points ? 1 : -1)),
    );

    expect(
      unwrapOptions(returnLt)
        .map(({ points, randomness }: any) => ({ points, randomness }))
        .sort((a: any, b: any) => (a.points > b.points ? 1 : -1)),
    ).toStrictEqual(
      [
        { points: 0n, randomness: 1n },
        { points: 1n, randomness: 1n },
        { points: 2n, randomness: 1n },
        { points: 4n, randomness: 1n },
        { points: 3n, randomness: 1n },
      ].sort((a: any, b: any) => (a.points > b.points ? 1 : -1)),
    );

    expect(
      unwrapOptions(returnGt)
        .map(({ points, randomness }: any) => ({ points, randomness }))
        .sort((a: any, b: any) => (a.points > b.points ? 1 : -1)),
    ).toStrictEqual(
      [
        { points: 7n, randomness: 1n },
        { points: 9n, randomness: 1n },
        { points: 6n, randomness: 1n },
        { points: 8n, randomness: 1n },
      ].sort((a: any, b: any) => (a.points > b.points ? 1 : -1)),
    );

    expect(
      unwrapOptions(returnLte)
        .map(({ points, randomness }: any) => ({ points, randomness }))
        .sort((a: any, b: any) => (a.points > b.points ? 1 : -1)),
    ).toStrictEqual(
      [
        { points: 5n, randomness: 1n },
        { points: 5n, randomness: 1n },
        { points: 0n, randomness: 1n },
        { points: 1n, randomness: 1n },
        { points: 2n, randomness: 1n },
        { points: 4n, randomness: 1n },
        { points: 3n, randomness: 1n },
      ].sort((a: any, b: any) => (a.points > b.points ? 1 : -1)),
    );

    expect(
      unwrapOptions(returnGte)
        .map(({ points, randomness }: any) => ({ points, randomness }))
        .sort((a: any, b: any) => (a.points > b.points ? 1 : -1)),
    ).toStrictEqual(
      [
        { points: 5n, randomness: 1n },
        { points: 5n, randomness: 1n },
        { points: 7n, randomness: 1n },
        { points: 9n, randomness: 1n },
        { points: 6n, randomness: 1n },
        { points: 8n, randomness: 1n },
      ].sort((a: any, b: any) => (a.points > b.points ? 1 : -1)),
    );
  }, 300_000);
});
