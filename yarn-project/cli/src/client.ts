import { AztecRPC, createAztecRpcClient } from '@aztec/aztec.js';
import { DebugLogger } from '@aztec/foundation/log';
import { fileURLToPath } from '@aztec/foundation/url';

import { readFileSync } from 'fs';
import { dirname, resolve } from 'path';
import { gtr, ltr, satisfies, valid } from 'semver';

/**
 * Creates an Aztec RPC client with a given set of retries on non-server errors.
 * @param rpcUrl - URL of the RPC server.
 * @returns An RPC client.
 */
export function createClient(rpcUrl: string) {
  return createAztecRpcClient(rpcUrl);
}

/**
 * Creates an Aztec RPC client with a given set of retries on non-server errors.
 * Checks that the RPC server matches the expected version, and warns if not.
 * @param rpcUrl - URL of the RPC server.
 * @param logger - Debug logger to warn version incompatibilities.
 * @returns An RPC client.
 */
export async function createCompatibleClient(rpcUrl: string, logger: DebugLogger) {
  const client = createClient(rpcUrl);
  const packageJsonPath = resolve(dirname(fileURLToPath(import.meta.url)), '../package.json');
  const packageJsonContents = JSON.parse(readFileSync(packageJsonPath).toString());
  const expectedVersionRange = packageJsonContents.version; // During sandbox, we'll expect exact matches

  try {
    await checkServerVersion(client, expectedVersionRange);
  } catch (err) {
    if (err instanceof VersionMismatchError) {
      logger.warn(err.message);
    } else {
      throw err;
    }
  }

  return client;
}

/** Mismatch between server and client versions. */
class VersionMismatchError extends Error {}

/**
 * Checks that the RPC server version matches the expected one by this CLI. Throws if not.
 * @param rpc - RPC server connection.
 * @param expectedVersionRange - Expected version by CLI.
 */
export async function checkServerVersion(rpc: AztecRPC, expectedVersionRange: string) {
  const serverName = 'Aztec Sandbox';
  const { sandboxVersion } = await rpc.getNodeInfo();
  if (!sandboxVersion) {
    throw new VersionMismatchError(`Couldn't determine ${serverName} version. You may run into issues.`);
  }
  if (!sandboxVersion || !valid(sandboxVersion)) {
    throw new VersionMismatchError(
      `Missing or invalid version identifier for ${serverName} (${sandboxVersion ?? 'empty'}).`,
    );
  } else if (!satisfies(sandboxVersion, expectedVersionRange)) {
    if (gtr(sandboxVersion, expectedVersionRange)) {
      throw new VersionMismatchError(
        `${serverName} is running version ${sandboxVersion} which is newer than the expected by this CLI (${expectedVersionRange}). Consider upgrading your CLI to a newer version.`,
      );
    } else if (ltr(sandboxVersion, expectedVersionRange)) {
      throw new VersionMismatchError(
        `${serverName} is running version ${sandboxVersion} which is older than the expected by this CLI (${expectedVersionRange}). Consider upgrading your ${serverName} to a newer version.`,
      );
    } else {
      throw new VersionMismatchError(
        `${serverName} is running version ${sandboxVersion} which does not match the expected by this CLI (${expectedVersionRange}).`,
      );
    }
  }
}
