import { Keccak } from 'sha3';

/**
 * Computes the Keccak-256 hash of the given input buffer.
 *
 * @param input - The input buffer to be hashed.
 * @returns The computed Keccak-256 hash as a Buffer.
 */
export function keccak256(input: Buffer) {
  const hash = new Keccak(256);
  return hash.update(input).digest();
}

/**
 * Computes the keccak-256 hash of a given input string and returns the result as a hexadecimal string.
 */
export function keccak256String(input: string) {
  const hash = new Keccak(256);
  hash.reset();
  hash.update(input);
  return hash.digest('hex');
}

/**
 * Computes the Keccak-224 hash of the given input buffer.
 *
 * @param input - The input buffer to be hashed.
 * @returns The computed Keccak-224 hash as a Buffer.
 */
export function keccak224(input: Buffer) {
  const hash = new Keccak(224);
  return hash.update(input).digest();
}

/**
 * Computes the Keccak-f1600 permutation of the input.
 * @param state 25 64-bit words.
 * @returns The permuted state.
 */
export function keccakf1600(state: bigint[]): bigint[] {
  // Ideally we'd assert the size of the state and its constituent elements here.
  // But since this gets included in the browser bundle, we can't use Node's assert module.
  /* The implementation based on the "simple" implementation by Ronny Van Keer. */
  /* Adapted from Barretenberg's CPP implementation. */
  let Eba, Ebe, Ebi, Ebo, Ebu;
  let Ega, Ege, Egi, Ego, Egu;
  let Eka, Eke, Eki, Eko, Eku;
  let Ema, Eme, Emi, Emo, Emu;
  let Esa, Ese, Esi, Eso, Esu;

  let Ba, Be, Bi, Bo, Bu;

  let Da, De, Di, Do, Du;

  let Aba = state[0];
  let Abe = state[1];
  let Abi = state[2];
  let Abo = state[3];
  let Abu = state[4];
  let Aga = state[5];
  let Age = state[6];
  let Agi = state[7];
  let Ago = state[8];
  let Agu = state[9];
  let Aka = state[10];
  let Ake = state[11];
  let Aki = state[12];
  let Ako = state[13];
  let Aku = state[14];
  let Ama = state[15];
  let Ame = state[16];
  let Ami = state[17];
  let Amo = state[18];
  let Amu = state[19];
  let Asa = state[20];
  let Ase = state[21];
  let Asi = state[22];
  let Aso = state[23];
  let Asu = state[24];

  for (let round = 0; round < 24; round += 2) {
    /* Round (round + 0): Axx -> Exx */
    Ba = Aba ^ Aga ^ Aka ^ Ama ^ Asa;
    Be = Abe ^ Age ^ Ake ^ Ame ^ Ase;
    Bi = Abi ^ Agi ^ Aki ^ Ami ^ Asi;
    Bo = Abo ^ Ago ^ Ako ^ Amo ^ Aso;
    Bu = Abu ^ Agu ^ Aku ^ Amu ^ Asu;

    Da = Bu ^ rol(Be, 1n);
    De = Ba ^ rol(Bi, 1n);
    Di = Be ^ rol(Bo, 1n);
    Do = Bi ^ rol(Bu, 1n);
    Du = Bo ^ rol(Ba, 1n);

    Ba = Aba ^ Da;
    Be = rol(Age ^ De, 44n);
    Bi = rol(Aki ^ Di, 43n);
    Bo = rol(Amo ^ Do, 21n);
    Bu = rol(Asu ^ Du, 14n);
    Eba = Ba ^ (~Be & Bi) ^ roundConstants[round];
    Ebe = Be ^ (~Bi & Bo);
    Ebi = Bi ^ (~Bo & Bu);
    Ebo = Bo ^ (~Bu & Ba);
    Ebu = Bu ^ (~Ba & Be);

    Ba = rol(Abo ^ Do, 28n);
    Be = rol(Agu ^ Du, 20n);
    Bi = rol(Aka ^ Da, 3n);
    Bo = rol(Ame ^ De, 45n);
    Bu = rol(Asi ^ Di, 61n);
    Ega = Ba ^ (~Be & Bi);
    Ege = Be ^ (~Bi & Bo);
    Egi = Bi ^ (~Bo & Bu);
    Ego = Bo ^ (~Bu & Ba);
    Egu = Bu ^ (~Ba & Be);

    Ba = rol(Abe ^ De, 1n);
    Be = rol(Agi ^ Di, 6n);
    Bi = rol(Ako ^ Do, 25n);
    Bo = rol(Amu ^ Du, 8n);
    Bu = rol(Asa ^ Da, 18n);
    Eka = Ba ^ (~Be & Bi);
    Eke = Be ^ (~Bi & Bo);
    Eki = Bi ^ (~Bo & Bu);
    Eko = Bo ^ (~Bu & Ba);
    Eku = Bu ^ (~Ba & Be);

    Ba = rol(Abu ^ Du, 27n);
    Be = rol(Aga ^ Da, 36n);
    Bi = rol(Ake ^ De, 10n);
    Bo = rol(Ami ^ Di, 15n);
    Bu = rol(Aso ^ Do, 56n);
    Ema = Ba ^ (~Be & Bi);
    Eme = Be ^ (~Bi & Bo);
    Emi = Bi ^ (~Bo & Bu);
    Emo = Bo ^ (~Bu & Ba);
    Emu = Bu ^ (~Ba & Be);

    Ba = rol(Abi ^ Di, 62n);
    Be = rol(Ago ^ Do, 55n);
    Bi = rol(Aku ^ Du, 39n);
    Bo = rol(Ama ^ Da, 41n);
    Bu = rol(Ase ^ De, 2n);
    Esa = Ba ^ (~Be & Bi);
    Ese = Be ^ (~Bi & Bo);
    Esi = Bi ^ (~Bo & Bu);
    Eso = Bo ^ (~Bu & Ba);
    Esu = Bu ^ (~Ba & Be);

    /* Round (round + 1): Exx -> Axx */

    Ba = Eba ^ Ega ^ Eka ^ Ema ^ Esa;
    Be = Ebe ^ Ege ^ Eke ^ Eme ^ Ese;
    Bi = Ebi ^ Egi ^ Eki ^ Emi ^ Esi;
    Bo = Ebo ^ Ego ^ Eko ^ Emo ^ Eso;
    Bu = Ebu ^ Egu ^ Eku ^ Emu ^ Esu;

    Da = Bu ^ rol(Be, 1n);
    De = Ba ^ rol(Bi, 1n);
    Di = Be ^ rol(Bo, 1n);
    Do = Bi ^ rol(Bu, 1n);
    Du = Bo ^ rol(Ba, 1n);

    Ba = Eba ^ Da;
    Be = rol(Ege ^ De, 44n);
    Bi = rol(Eki ^ Di, 43n);
    Bo = rol(Emo ^ Do, 21n);
    Bu = rol(Esu ^ Du, 14n);
    Aba = Ba ^ (~Be & Bi) ^ roundConstants[round + 1];
    Abe = Be ^ (~Bi & Bo);
    Abi = Bi ^ (~Bo & Bu);
    Abo = Bo ^ (~Bu & Ba);
    Abu = Bu ^ (~Ba & Be);

    Ba = rol(Ebo ^ Do, 28n);
    Be = rol(Egu ^ Du, 20n);
    Bi = rol(Eka ^ Da, 3n);
    Bo = rol(Eme ^ De, 45n);
    Bu = rol(Esi ^ Di, 61n);
    Aga = Ba ^ (~Be & Bi);
    Age = Be ^ (~Bi & Bo);
    Agi = Bi ^ (~Bo & Bu);
    Ago = Bo ^ (~Bu & Ba);
    Agu = Bu ^ (~Ba & Be);

    Ba = rol(Ebe ^ De, 1n);
    Be = rol(Egi ^ Di, 6n);
    Bi = rol(Eko ^ Do, 25n);
    Bo = rol(Emu ^ Du, 8n);
    Bu = rol(Esa ^ Da, 18n);
    Aka = Ba ^ (~Be & Bi);
    Ake = Be ^ (~Bi & Bo);
    Aki = Bi ^ (~Bo & Bu);
    Ako = Bo ^ (~Bu & Ba);
    Aku = Bu ^ (~Ba & Be);

    Ba = rol(Ebu ^ Du, 27n);
    Be = rol(Ega ^ Da, 36n);
    Bi = rol(Eke ^ De, 10n);
    Bo = rol(Emi ^ Di, 15n);
    Bu = rol(Eso ^ Do, 56n);
    Ama = Ba ^ (~Be & Bi);
    Ame = Be ^ (~Bi & Bo);
    Ami = Bi ^ (~Bo & Bu);
    Amo = Bo ^ (~Bu & Ba);
    Amu = Bu ^ (~Ba & Be);

    Ba = rol(Ebi ^ Di, 62n);
    Be = rol(Ego ^ Do, 55n);
    Bi = rol(Eku ^ Du, 39n);
    Bo = rol(Ema ^ Da, 41n);
    Bu = rol(Ese ^ De, 2n);
    Asa = Ba ^ (~Be & Bi);
    Ase = Be ^ (~Bi & Bo);
    Asi = Bi ^ (~Bo & Bu);
    Aso = Bo ^ (~Bu & Ba);
    Asu = Bu ^ (~Ba & Be);
  }

  state[0] = Aba;
  state[1] = Abe;
  state[2] = Abi;
  state[3] = Abo;
  state[4] = Abu;
  state[5] = Aga;
  state[6] = Age;
  state[7] = Agi;
  state[8] = Ago;
  state[9] = Agu;
  state[10] = Aka;
  state[11] = Ake;
  state[12] = Aki;
  state[13] = Ako;
  state[14] = Aku;
  state[15] = Ama;
  state[16] = Ame;
  state[17] = Ami;
  state[18] = Amo;
  state[19] = Amu;
  state[20] = Asa;
  state[21] = Ase;
  state[22] = Asi;
  state[23] = Aso;
  state[24] = Asu;

  return state;
}

function rol(x: bigint, s: bigint) {
  return BigInt.asUintN(64, x << s) | (x >> (64n - s));
}

const roundConstants: bigint[] = [
  0x0000000000000001n,
  0x0000000000008082n,
  0x800000000000808an,
  0x8000000080008000n,
  0x000000000000808bn,
  0x0000000080000001n,
  0x8000000080008081n,
  0x8000000000008009n,
  0x000000000000008an,
  0x0000000000000088n,
  0x0000000080008009n,
  0x000000008000000an,
  0x000000008000808bn,
  0x800000000000008bn,
  0x8000000000008089n,
  0x8000000000008003n,
  0x8000000000008002n,
  0x8000000000000080n,
  0x000000000000800an,
  0x800000008000000an,
  0x8000000080008081n,
  0x8000000000008080n,
  0x0000000080000001n,
  0x8000000080008008n,
];
