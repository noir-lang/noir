import { ArchiverConfig } from '@aztec/archiver';
import { SequencerClientConfig } from '@aztec/sequencer-client';

/**
 * The configuration the aztec node.
 */
export type AztecNodeConfig = ArchiverConfig & SequencerClientConfig;
