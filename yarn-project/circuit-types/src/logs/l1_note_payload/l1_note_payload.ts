import {
  AztecAddress,
  type GrumpkinPrivateKey,
  type KeyValidationRequest,
  type PublicKey,
  computeIvpkApp,
  computeIvskApp,
  computeOvskApp,
  derivePublicKeyFromSecretKey,
} from '@aztec/circuits.js';
import { Fr, Point } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { EncryptedLogHeader } from './encrypted_log_header.js';
import { EncryptedLogIncomingBody } from './encrypted_log_incoming_body.js';
import { EncryptedLogOutgoingBody } from './encrypted_log_outgoing_body.js';
import { Note } from './note.js';

// Both the incoming and the outgoing header are 48 bytes.
// 32 bytes for the address, and 16 bytes padding to follow PKCS#7
const HEADER_SIZE = 48;

// The outgoing body is constant size of 176 bytes.
// 160 bytes for the secret key, address, and public key, and 16 bytes padding to follow PKCS#7
const OUTGOING_BODY_SIZE = 176;
/**
 * A class which wraps note data which is pushed on L1.
 * @remarks This data is required to compute a nullifier/to spend a note. Along with that this class contains
 * the necessary functionality to encrypt and decrypt the data.
 */
export class L1NotePayload {
  constructor(
    /**
     * A note as emitted from Noir contract. Can be used along with private key to compute nullifier.
     */
    public note: Note,
    /**
     * Address of the contract this tx is interacting with.
     */
    public contractAddress: AztecAddress,
    /**
     * Storage slot of the underlying note.
     */
    public storageSlot: Fr,
    /**
     * Type identifier for the underlying note, required to determine how to compute its hash and nullifier.
     */
    public noteTypeId: Fr,
  ) {}

  /**
   * Deserializes the L1NotePayload object from a Buffer.
   * @param buffer - Buffer or BufferReader object to deserialize.
   * @returns An instance of L1NotePayload.
   */
  static fromBuffer(buffer: Buffer | BufferReader): L1NotePayload {
    const reader = BufferReader.asReader(buffer);
    return new L1NotePayload(
      reader.readObject(Note),
      reader.readObject(AztecAddress),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
    );
  }

  /**
   * Serializes the L1NotePayload object into a Buffer.
   * @returns Buffer representation of the L1NotePayload object.
   */
  toBuffer() {
    return serializeToBuffer([this.note, this.contractAddress, this.storageSlot, this.noteTypeId]);
  }

  /**
   * Create a random L1NotePayload object (useful for testing purposes).
   * @param contract - The address of a contract the note was emitted from.
   * @returns A random L1NotePayload object.
   */
  static random(contract = AztecAddress.random()) {
    return new L1NotePayload(Note.random(), contract, Fr.random(), Fr.random());
  }

  /**
   * Encrypts a note payload for a given recipient and sender.
   * Creates an incoming log the the recipient using the recipient's ivsk, and
   * an outgoing log for the sender using the sender's ovsk.
   *
   * @param ephSk - An ephemeral secret key used for the encryption
   * @param recipient - The recipient address, retrievable by the sender for his logs
   * @param ivpk - The incoming viewing public key of the recipient
   * @param ovKeys - The outgoing viewing keys of the sender
   * @returns A buffer containing the encrypted log payload
   * @throws If the ivpk is zero.
   */
  public encrypt(ephSk: GrumpkinPrivateKey, recipient: AztecAddress, ivpk: PublicKey, ovKeys: KeyValidationRequest) {
    if (ivpk.isZero()) {
      throw new Error(
        `Attempting to encrypt with a zero ivpk. You have probably passed a zero value in your Noir code somewhere thinking that the note won't broadcasted... but it was.`,
      );
    }

    const ephPk = derivePublicKeyFromSecretKey(ephSk);

    const header = new EncryptedLogHeader(this.contractAddress);

    const incomingHeaderCiphertext = header.computeCiphertext(ephSk, ivpk);
    const outgoingHeaderCiphertext = header.computeCiphertext(ephSk, ovKeys.pkM);

    const ivpkApp = computeIvpkApp(ivpk, this.contractAddress);

    const incomingBodyCiphertext = new EncryptedLogIncomingBody(
      this.storageSlot,
      this.noteTypeId,
      this.note,
    ).computeCiphertext(ephSk, ivpkApp);

    const outgoingBodyCiphertext = new EncryptedLogOutgoingBody(ephSk, recipient, ivpkApp).computeCiphertext(
      ovKeys.skAppAsGrumpkinPrivateKey,
      ephPk,
    );

    return Buffer.concat([
      ephPk.toBuffer(),
      incomingHeaderCiphertext,
      outgoingHeaderCiphertext,
      outgoingBodyCiphertext,
      incomingBodyCiphertext,
    ]);
  }

  /**
   * Decrypts a ciphertext as an incoming log.
   *
   * This is executable by the recipient of the note, and uses the ivsk to decrypt the payload.
   * The outgoing parts of the log are ignored entirely.
   *
   * Produces the same output as `decryptAsOutgoing`.
   *
   * @param ciphertext - The ciphertext for the log
   * @param ivsk - The incoming viewing secret key, used to decrypt the logs
   * @returns The decrypted log payload
   */
  public static decryptAsIncoming(ciphertext: Buffer | bigint[], ivsk: GrumpkinPrivateKey) {
    const input = Buffer.isBuffer(ciphertext) ? ciphertext : Buffer.from(ciphertext.map((x: bigint) => Number(x)));
    const reader = BufferReader.asReader(input);

    const ephPk = reader.readObject(Point);

    const incomingHeader = EncryptedLogHeader.fromCiphertext(reader.readBytes(HEADER_SIZE), ivsk, ephPk);

    // Skipping the outgoing header and body
    reader.readBytes(HEADER_SIZE);
    reader.readBytes(OUTGOING_BODY_SIZE);

    // The incoming can be of variable size, so we read until the end
    const incomingBodySlice = reader.readToEnd();

    const ivskApp = computeIvskApp(ivsk, incomingHeader.address);
    const incomingBody = EncryptedLogIncomingBody.fromCiphertext(incomingBodySlice, ivskApp, ephPk);

    return new L1NotePayload(
      incomingBody.note,
      incomingHeader.address,
      incomingBody.storageSlot,
      incomingBody.noteTypeId,
    );
  }

  /**
   * Decrypts a ciphertext as an outgoing log.
   *
   * This is executable by the sender of the note, and uses the ovsk to decrypt the payload.
   * The outgoing parts are decrypted to retrieve information that allows the sender to
   * decrypt the incoming log, and learn about the note contents.
   *
   * Produces the same output as `decryptAsIncoming`.
   *
   * @param ciphertext - The ciphertext for the log
   * @param ovsk - The outgoing viewing secret key, used to decrypt the logs
   * @returns The decrypted log payload
   */
  public static decryptAsOutgoing(ciphertext: Buffer | bigint[], ovsk: GrumpkinPrivateKey) {
    const input = Buffer.isBuffer(ciphertext) ? ciphertext : Buffer.from(ciphertext.map((x: bigint) => Number(x)));
    const reader = BufferReader.asReader(input);

    const ephPk = reader.readObject(Point);

    // Skip the incoming header
    reader.readBytes(HEADER_SIZE);

    const outgoingHeader = EncryptedLogHeader.fromCiphertext(reader.readBytes(HEADER_SIZE), ovsk, ephPk);

    const ovskApp = computeOvskApp(ovsk, outgoingHeader.address);
    const outgoingBody = EncryptedLogOutgoingBody.fromCiphertext(reader.readBytes(OUTGOING_BODY_SIZE), ovskApp, ephPk);

    // The incoming can be of variable size, so we read until the end
    const incomingBodySlice = reader.readToEnd();

    const incomingBody = EncryptedLogIncomingBody.fromCiphertext(
      incomingBodySlice,
      outgoingBody.ephSk,
      outgoingBody.recipientIvpkApp,
    );

    return new L1NotePayload(
      incomingBody.note,
      outgoingHeader.address,
      incomingBody.storageSlot,
      incomingBody.noteTypeId,
    );
  }

  public equals(other: L1NotePayload) {
    return (
      this.note.equals(other.note) &&
      this.contractAddress.equals(other.contractAddress) &&
      this.storageSlot.equals(other.storageSlot) &&
      this.noteTypeId.equals(other.noteTypeId)
    );
  }
}
