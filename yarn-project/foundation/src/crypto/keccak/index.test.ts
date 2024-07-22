import { keccakf1600 } from './index.js';

describe('keccakf1600', () => {
  it('zero test vector should match', () => {
    const input = [...Array(25)].map(() => 0n);

    const out = keccakf1600(input);
    const asStrings = out?.map(x => x.toString(16).padStart(16, '0'));

    expect(asStrings).toEqual([
      'f1258f7940e1dde7',
      '84d5ccf933c0478a',
      'd598261ea65aa9ee',
      'bd1547306f80494d',
      '8b284e056253d057',
      'ff97a42d7f8e6fd4',
      '90fee5a0a44647c4',
      '8c5bda0cd6192e76',
      'ad30a6f71b19059c',
      '30935ab7d08ffc64',
      'eb5aa93f2317d635',
      'a9a6e6260d712103',
      '81a57c16dbcf555f',
      '43b831cd0347c826',
      '01f22f1a11a5569f',
      '05e5635a21d9ae61',
      '64befef28cc970f2',
      '613670957bc46611',
      'b87c5a554fd00ecb',
      '8c3ee88a1ccf32c8',
      '940c7922ae3a2614',
      '1841f924a2c509e4',
      '16f53526e70465c2',
      '75f644e97f30a13b',
      'eaf1ff7b5ceca249',
    ]);
  });

  it('test vector should match', () => {
    const input = [
      'f1258f7940e1dde7',
      '84d5ccf933c0478a',
      'd598261ea65aa9ee',
      'bd1547306f80494d',
      '8b284e056253d057',
      'ff97a42d7f8e6fd4',
      '90fee5a0a44647c4',
      '8c5bda0cd6192e76',
      'ad30a6f71b19059c',
      '30935ab7d08ffc64',
      'eb5aa93f2317d635',
      'a9a6e6260d712103',
      '81a57c16dbcf555f',
      '43b831cd0347c826',
      '01f22f1a11a5569f',
      '05e5635a21d9ae61',
      '64befef28cc970f2',
      '613670957bc46611',
      'b87c5a554fd00ecb',
      '8c3ee88a1ccf32c8',
      '940c7922ae3a2614',
      '1841f924a2c509e4',
      '16f53526e70465c2',
      '75f644e97f30a13b',
      'eaf1ff7b5ceca249',
    ].map(x => BigInt(`0x${x}`));

    const out = keccakf1600(input);
    const asStrings = out?.map(x => x.toString(16).padStart(16, '0'));

    expect(asStrings).toEqual([
      '2d5c954df96ecb3c',
      '6a332cd07057b56d',
      '093d8d1270d76b6c',
      '8a20d9b25569d094',
      '4f9c4f99e5e7f156',
      'f957b9a2da65fb38',
      '85773dae1275af0d',
      'faf4f247c3d810f7',
      '1f1b9ee6f79a8759',
      'e4fecc0fee98b425',
      '68ce61b6b9ce68a1',
      'deea66c4ba8f974f',
      '33c43d836eafb1f5',
      'e00654042719dbd9',
      '7cf8a9f009831265',
      'fd5449a6bf174743',
      '97ddad33d8994b40',
      '48ead5fc5d0be774',
      'e3b8c8ee55b7b03c',
      '91a0226e649e42e9',
      '900e3129e7badd7b',
      '202a9ec5faa3cce8',
      '5b3402464e1c3db6',
      '609f4e62a44c1059',
      '20d06cd26a8fbf5c',
    ]);
  });
});
