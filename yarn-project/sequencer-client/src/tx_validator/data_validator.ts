import { Tx, type TxValidator } from '@aztec/circuit-types';
import { createDebugLogger } from '@aztec/foundation/log';

export class DataTxValidator implements TxValidator<Tx> {
  #log = createDebugLogger('aztec:sequencer:tx_validator:tx_data');

  validateTxs(txs: Tx[]): Promise<[validTxs: Tx[], invalidTxs: Tx[]]> {
    const validTxs: Tx[] = [];
    const invalidTxs: Tx[] = [];
    for (const tx of txs) {
      if (!this.#hasCorrectExecutionRequests(tx)) {
        invalidTxs.push(tx);
        continue;
      }

      validTxs.push(tx);
    }

    return Promise.resolve([validTxs, invalidTxs]);
  }

  #hasCorrectExecutionRequests(tx: Tx): boolean {
    const callRequests = [
      ...tx.data.getRevertiblePublicCallRequests(),
      ...tx.data.getNonRevertiblePublicCallRequests(),
    ];
    if (callRequests.length !== tx.enqueuedPublicFunctionCalls.length) {
      this.#log.warn(
        `Rejecting tx ${Tx.getHash(tx)} because of mismatch number of execution requests for public calls. Expected ${
          callRequests.length
        }. Got ${tx.enqueuedPublicFunctionCalls.length}.`,
      );
      return false;
    }

    const invalidExecutionRequestIndex = tx.enqueuedPublicFunctionCalls.findIndex(
      (execRequest, i) => !execRequest.isForCallRequest(callRequests[i]),
    );
    if (invalidExecutionRequestIndex !== -1) {
      this.#log.warn(
        `Rejecting tx ${Tx.getHash(
          tx,
        )} because of incorrect execution requests for public call at index ${invalidExecutionRequestIndex}.`,
      );
      return false;
    }

    const teardownCallRequest = tx.data.getTeardownPublicCallRequest();
    const isInvalidTeardownExecutionRequest =
      (!teardownCallRequest && !tx.publicTeardownFunctionCall.isEmpty()) ||
      (teardownCallRequest && !tx.publicTeardownFunctionCall.isForCallRequest(teardownCallRequest));
    if (isInvalidTeardownExecutionRequest) {
      this.#log.warn(`Rejecting tx ${Tx.getHash(tx)} because of incorrect teardown execution requests.`);
      return false;
    }

    return true;
  }

  // TODO: Check logs.
}
