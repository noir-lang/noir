import { type PXE, createPXEClient } from '@aztec/aztec.js';
import { type DebugLogger } from '@aztec/foundation/log';
import { NoRetryError } from '@aztec/foundation/retry';

import axios, { type AxiosError, type AxiosResponse } from 'axios';
import { gtr, ltr, satisfies, valid } from 'semver';

export async function axiosFetch(
  host: string,
  rpcMethod: string,
  body: any,
  useApiEndpoints: boolean,
  _noRetry = true,
) {
  let resp: AxiosResponse;
  if (useApiEndpoints) {
    resp = await axios
      .post(`${host}/${rpcMethod}`, body, {
        headers: { 'content-type': 'application/json' },
      })
      .catch((error: AxiosError) => {
        if (error.response) {
          return error.response;
        }
        throw error;
      });
  } else {
    resp = await axios
      .post(
        host,
        { ...body, method: rpcMethod },
        {
          headers: { 'content-type': 'application/json' },
        },
      )
      .catch((error: AxiosError) => {
        if (error.response) {
          return error.response;
        }
        throw error;
      });
  }

  const isOK = resp.status >= 200 && resp.status < 300;
  if (isOK) {
    return resp.data;
  } else if (resp.status >= 400 && resp.status < 500) {
    throw new NoRetryError('(JSON-RPC PROPAGATED) ' + resp.data.error.message);
  } else {
    throw new Error('(JSON-RPC PROPAGATED) ' + resp.data.error.message);
  }
}

/**
 * Creates a PXE client with a given set of retries on non-server errors.
 * Checks that PXE matches the expected version, and warns if not.
 * @param rpcUrl - URL of the RPC server wrapping the PXE.
 * @param _logger - Debug logger to warn version incompatibilities.
 * @returns A PXE client.
 */
export function createCompatibleClient(rpcUrl: string, _logger: DebugLogger): Promise<PXE> {
  // Use axios due to timeout issues with fetch when proving TXs.
  const pxe = createPXEClient(rpcUrl, axiosFetch);
  return Promise.resolve(pxe);
}

/** Mismatch between server and client versions. */
class VersionMismatchError extends Error {}

/**
 * Checks that Private eXecution Environment (PXE) version matches the expected one by this CLI. Throws if not.
 * @param pxe - PXE client.
 * @param expectedVersionRange - Expected version by CLI.
 */
export async function checkServerVersion(pxe: PXE, expectedVersionRange: string) {
  const serverName = 'Aztec Node';
  const { nodeVersion } = await pxe.getNodeInfo();
  if (!nodeVersion) {
    throw new VersionMismatchError(`Couldn't determine ${serverName} version. You may run into issues.`);
  }
  if (!nodeVersion || !valid(nodeVersion)) {
    throw new VersionMismatchError(
      `Missing or invalid version identifier for ${serverName} (${nodeVersion ?? 'empty'}).`,
    );
  } else if (!satisfies(nodeVersion, expectedVersionRange)) {
    if (gtr(nodeVersion, expectedVersionRange)) {
      throw new VersionMismatchError(
        `${serverName} is running version ${nodeVersion} which is newer than the expected by this CLI (${expectedVersionRange}). Consider upgrading your CLI to a newer version.`,
      );
    } else if (ltr(nodeVersion, expectedVersionRange)) {
      throw new VersionMismatchError(
        `${serverName} is running version ${nodeVersion} which is older than the expected by this CLI (${expectedVersionRange}). Consider upgrading your ${serverName} to a newer version.`,
      );
    } else {
      throw new VersionMismatchError(
        `${serverName} is running version ${nodeVersion} which does not match the expected by this CLI (${expectedVersionRange}).`,
      );
    }
  }
}
