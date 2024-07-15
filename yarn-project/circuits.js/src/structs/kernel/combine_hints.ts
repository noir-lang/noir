import { type FieldsOf } from '@aztec/foundation/array';
import { removeArrayPaddingEnd } from '@aztec/foundation/collection';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { inspect } from 'util';

import {
  MAX_ENCRYPTED_LOGS_PER_TX,
  MAX_NOTE_HASHES_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MAX_UNENCRYPTED_LOGS_PER_TX,
} from '../../constants.gen.js';
import {
  deduplicateSortedArray,
  getNonEmptyItems,
  mergeAccumulatedData,
  sortByCounterGetSortedHints,
  sortByPositionThenCounterGetSortedHints,
} from '../../utils/index.js';
import { ScopedLogHash } from '../log_hash.js';
import { NoteHash } from '../note_hash.js';
import { PublicDataUpdateRequest } from '../public_data_update_request.js';
import { type PublicAccumulatedData } from './public_accumulated_data.js';

export class CombineHints {
  constructor(
    public readonly sortedNoteHashes: Tuple<NoteHash, typeof MAX_NOTE_HASHES_PER_TX>,
    public readonly sortedNoteHashesIndexes: Tuple<number, typeof MAX_NOTE_HASHES_PER_TX>,
    public readonly sortedUnencryptedLogsHashes: Tuple<ScopedLogHash, typeof MAX_UNENCRYPTED_LOGS_PER_TX>,
    public readonly sortedUnencryptedLogsHashesIndexes: Tuple<number, typeof MAX_UNENCRYPTED_LOGS_PER_TX>,
    public readonly sortedPublicDataUpdateRequests: Tuple<
      PublicDataUpdateRequest,
      typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
    >,
    public readonly sortedPublicDataUpdateRequestsIndexes: Tuple<number, typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
    public readonly dedupedPublicDataUpdateRequests: Tuple<
      PublicDataUpdateRequest,
      typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
    >,
    public readonly dedupedPublicDataUpdateRequestsRuns: Tuple<number, typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
  ) {}

  static getFields(fields: FieldsOf<CombineHints>) {
    return [
      fields.sortedNoteHashes,
      fields.sortedNoteHashesIndexes,
      fields.sortedUnencryptedLogsHashes,
      fields.sortedUnencryptedLogsHashesIndexes,
      fields.sortedPublicDataUpdateRequests,
      fields.sortedPublicDataUpdateRequestsIndexes,
      fields.dedupedPublicDataUpdateRequests,
      fields.dedupedPublicDataUpdateRequestsRuns,
    ] as const;
  }

  static from(fields: FieldsOf<CombineHints>): CombineHints {
    return new CombineHints(...CombineHints.getFields(fields));
  }

  toBuffer() {
    return serializeToBuffer(...CombineHints.getFields(this));
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new CombineHints(
      reader.readArray(MAX_NOTE_HASHES_PER_TX, NoteHash),
      reader.readNumbers(MAX_NOTE_HASHES_PER_TX),
      reader.readArray(MAX_UNENCRYPTED_LOGS_PER_TX, ScopedLogHash),
      reader.readNumbers(MAX_UNENCRYPTED_LOGS_PER_TX),
      reader.readArray(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest),
      reader.readNumbers(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX),
      reader.readArray(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest),
      reader.readNumbers(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX),
    );
  }

  static fromPublicData({
    revertibleData,
    nonRevertibleData,
  }: {
    revertibleData: PublicAccumulatedData;
    nonRevertibleData: PublicAccumulatedData;
  }): CombineHints {
    const mergedNoteHashes = mergeAccumulatedData(
      nonRevertibleData.noteHashes,
      revertibleData.noteHashes,
      MAX_NOTE_HASHES_PER_TX,
    );

    const [sortedNoteHashes, sortedNoteHashesIndexes] = sortByCounterGetSortedHints(
      mergedNoteHashes,
      MAX_NOTE_HASHES_PER_TX,
    );

    const unencryptedLogHashes = mergeAccumulatedData(
      nonRevertibleData.unencryptedLogsHashes,
      revertibleData.unencryptedLogsHashes,
      MAX_ENCRYPTED_LOGS_PER_TX,
    );

    const [sortedUnencryptedLogsHashes, sortedUnencryptedLogsHashesIndexes] = sortByCounterGetSortedHints(
      unencryptedLogHashes,
      MAX_ENCRYPTED_LOGS_PER_TX,
    );

    const publicDataUpdateRequests = mergeAccumulatedData(
      nonRevertibleData.publicDataUpdateRequests,
      revertibleData.publicDataUpdateRequests,
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    );

    // Since we're using `check_permutation` in the circuit, we need index hints based on the original array.
    const [sortedPublicDataUpdateRequests, sortedPublicDataUpdateRequestsIndexes] =
      sortByPositionThenCounterGetSortedHints(publicDataUpdateRequests, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, {
        ascending: true,
        hintIndexesBy: 'original',
      });

    // further, we need to fill in the rest of the hints with an identity mapping
    for (let i = 0; i < MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX; i++) {
      if (publicDataUpdateRequests[i].isEmpty()) {
        sortedPublicDataUpdateRequestsIndexes[i] = i;
      }
    }

    const [dedupedPublicDataUpdateRequests, dedupedPublicDataUpdateRequestsRuns] = deduplicateSortedArray(
      sortedPublicDataUpdateRequests,
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
      () => PublicDataUpdateRequest.empty(),
    );

    return CombineHints.from({
      sortedNoteHashes,
      sortedNoteHashesIndexes,
      sortedUnencryptedLogsHashes,
      sortedUnencryptedLogsHashesIndexes,
      sortedPublicDataUpdateRequests,
      sortedPublicDataUpdateRequestsIndexes,
      dedupedPublicDataUpdateRequests,
      dedupedPublicDataUpdateRequestsRuns,
    });
  }

  [inspect.custom](): string {
    return `CombineHints {
  sortedNoteHashes: ${getNonEmptyItems(this.sortedNoteHashes)
    .map(h => inspect(h))
    .join(', ')},
  sortedNoteHashesIndexes: ${removeArrayPaddingEnd(this.sortedNoteHashesIndexes, n => n === 0)},
  sortedUnencryptedLogsHashes: ${getNonEmptyItems(this.sortedUnencryptedLogsHashes)
    .map(h => inspect(h))
    .join(', ')},
  sortedUnencryptedLogsHashesIndexes: ${removeArrayPaddingEnd(this.sortedUnencryptedLogsHashesIndexes, n => n === 0)},
  sortedPublicDataUpdateRequests: ${getNonEmptyItems(this.sortedPublicDataUpdateRequests)
    .map(h => inspect(h))
    .join(', ')},
  sortedPublicDataUpdateRequestsIndexes: ${this.sortedPublicDataUpdateRequestsIndexes},
  dedupedPublicDataUpdateRequests: ${getNonEmptyItems(this.dedupedPublicDataUpdateRequests)
    .map(h => inspect(h))
    .join(', ')},
  dedupedPublicDataUpdateRequestsRuns: ${this.dedupedPublicDataUpdateRequestsRuns},
}`;
  }
}
