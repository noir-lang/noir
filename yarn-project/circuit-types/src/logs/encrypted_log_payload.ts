import {
  AztecAddress,
  Fr,
  type GrumpkinPrivateKey,
  Point,
  type PublicKey,
  computeIvpkApp,
  computeIvskApp,
  computeOvskApp,
  derivePublicKeyFromSecretKey,
} from '@aztec/circuits.js';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { EncryptedLogHeader } from './encrypted_log_header.js';
import { EncryptedLogIncomingBody } from './encrypted_log_incoming_body.js';
import { EncryptedLogOutgoingBody } from './encrypted_log_outgoing_body.js';
import { type L1NotePayload } from './l1_note_payload/l1_note_payload.js';
import { Note } from './l1_note_payload/note.js';

// A placeholder tag until we have a proper tag system in place.
const PLACEHOLDER_TAG = new Fr(33);

// Both the incoming and the outgoing header are 48 bytes.
// 32 bytes for the address, and 16 bytes padding to follow PKCS#7
const HEADER_SIZE = 48;

// The outgoing body is constant size of 176 bytes.
// 160 bytes for the secret key, address, and public key, and 16 bytes padding to follow PKCS#7
const OUTGOING_BODY_SIZE = 176;

export class EncryptedLogPayload {
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

  toBuffer() {
    return serializeToBuffer([this.note, this.contractAddress, this.storageSlot, this.noteTypeId]);
  }

  static fromBuffer(buffer: Buffer | BufferReader): EncryptedLogPayload {
    const reader = BufferReader.asReader(buffer);
    return new EncryptedLogPayload(
      reader.readObject(Note),
      reader.readObject(AztecAddress),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
    );
  }

  static fromL1NotePayload(l1NotePayload: L1NotePayload) {
    return new EncryptedLogPayload(
      l1NotePayload.note,
      l1NotePayload.contractAddress,
      l1NotePayload.storageSlot,
      l1NotePayload.noteTypeId,
    );
  }

  /**
   * Encrypts a note payload for a given recipient and sender.
   * Creates an incoming log the the recipient using the recipient's ivsk, and
   * an outgoing log for the sender using the sender's ovsk.
   *
   * @param ephSk - An ephemeral secret key used for the encryption
   * @param recipient - The recipient address, retrievable by the sender for his logs
   * @param ivpk - The incoming viewing public key of the recipient
   * @param ovsk - The outgoing viewing secret key of the sender
   * @returns A buffer containing the encrypted log payload
   */
  public encrypt(ephSk: GrumpkinPrivateKey, recipient: AztecAddress, ivpk: PublicKey, ovsk: GrumpkinPrivateKey) {
    const ephPk = derivePublicKeyFromSecretKey(ephSk);
    const ovpk = derivePublicKeyFromSecretKey(ovsk);

    const header = new EncryptedLogHeader(this.contractAddress);

    const incomingHeaderCiphertext = header.computeCiphertext(ephSk, ivpk);
    const outgoingHeaderCiphertext = header.computeCiphertext(ephSk, ovpk);

    const ivpkApp = computeIvpkApp(ivpk, this.contractAddress);

    const incomingBodyCiphertext = new EncryptedLogIncomingBody(
      this.storageSlot,
      this.noteTypeId,
      this.note,
    ).computeCiphertext(ephSk, ivpkApp);

    const ovskApp = computeOvskApp(ovsk, this.contractAddress);

    const outgoingBodyCiphertext = new EncryptedLogOutgoingBody(ephSk, recipient, ivpkApp).computeCiphertext(
      ovskApp,
      ephPk,
    );

    return Buffer.concat([
      PLACEHOLDER_TAG.toBuffer(),
      PLACEHOLDER_TAG.toBuffer(),
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

    // We don't use the tags as part of the decryption here, we just gotta read to skip them.
    reader.readObject(Fr); // incoming tag
    reader.readObject(Fr); // outgoing tag

    const ephPk = reader.readObject(Point);

    const incomingHeader = EncryptedLogHeader.fromCiphertext(reader.readBytes(HEADER_SIZE), ivsk, ephPk);

    // Skipping the outgoing header and body
    reader.readBytes(HEADER_SIZE);
    reader.readBytes(OUTGOING_BODY_SIZE);

    // The incoming can be of variable size, so we read until the end
    const incomingBodySlice = reader.readToEnd();

    const ivskApp = computeIvskApp(ivsk, incomingHeader.address);
    const incomingBody = EncryptedLogIncomingBody.fromCiphertext(incomingBodySlice, ivskApp, ephPk);

    return new EncryptedLogPayload(
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

    // We don't use the tags as part of the decryption here, we just gotta read to skip them.
    reader.readObject(Fr); // incoming tag
    reader.readObject(Fr); // outgoing tag

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

    return new EncryptedLogPayload(
      incomingBody.note,
      outgoingHeader.address,
      incomingBody.storageSlot,
      incomingBody.noteTypeId,
    );
  }
}
