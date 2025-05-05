import { BrilligFunctionId, DebugFileMap, DebugInfo, LocationNodeDebugInfo, OpcodeLocation } from '@noir-lang/types';
import { inflate } from 'pako';
import { base64Decode } from './base64_decode';
import { ExecutionError } from '@noir-lang/acvm_js';

/**
 * A stack of calls, resolved or not
 */
type CallStack = SourceCodeLocation[] | OpcodeLocation[];

/**
 * A resolved pointer to a failing section of the noir source code.
 */
interface SourceCodeLocation {
  /**
   * The path to the source file.
   */
  filePath: string;
  /**
   * The line number of the location.
   */
  line: number;
  /**
   * The column number of the location.
   */
  column: number;
  /**
   * The source code text of the location.
   */
  locationText: string;
}

export function parseDebugSymbols(debugSymbols: string): DebugInfo[] {
  return JSON.parse(inflate(base64Decode(debugSymbols), { to: 'string', raw: true })).debug_infos;
}

/**
 * Extracts the call stack from an thrown by the acvm.
 * @param error - The error to extract from.
 * @param debug - The debug metadata of the program called.
 * @param files - The files used for compilation of the program.
 * @returns The call stack, if available.
 */
export function extractCallStack(error: ExecutionError, debug: DebugInfo, files: DebugFileMap): CallStack | undefined {
  if (!('callStack' in error) || !error.callStack) {
    return undefined;
  }
  const { callStack, brilligFunctionId } = error;
  if (!debug) {
    return callStack;
  }

  try {
    return resolveOpcodeLocations(callStack, debug, files, brilligFunctionId);
  } catch (_err) {
    return callStack;
  }
}

/**
 * Resolves the source code locations from an array of opcode locations
 */
function resolveOpcodeLocations(
  opcodeLocations: OpcodeLocation[],
  debug: DebugInfo,
  files: DebugFileMap,
  brilligFunctionId?: BrilligFunctionId,
): SourceCodeLocation[] {
  let locations = opcodeLocations.flatMap((opcodeLocation) =>
    getSourceCodeLocationsFromOpcodeLocation(opcodeLocation, debug, files, brilligFunctionId),
  );
  // Adds the acir call stack if the last location is a brillig opcode
  if (locations.length > 0) {
    const decomposedOpcodeLocation = opcodeLocations[opcodeLocations.length - 1].split('.');
    if (decomposedOpcodeLocation.length === 2) {
      const acirCallstackId = debug.acir_locations[decomposedOpcodeLocation[0]];
      if (acirCallstackId !== undefined) {
        const callStack = debug.location_tree.locations[acirCallstackId];
        const acirCallstack = getCallStackFromLocationNode(callStack, debug.location_tree.locations, files);
        locations = acirCallstack.concat(locations);
      }
    }
  }
  return locations;
}

function getCallStackFromLocationNode(
  callStack: LocationNodeDebugInfo,
  location_tree: LocationNodeDebugInfo[],
  files: DebugFileMap,
): SourceCodeLocation[] {
  const result: SourceCodeLocation[] = [];
  while (callStack.parent !== null) {
    const { file: fileId, span } = callStack.value;
    const { path, source } = files[fileId];

    const locationText = source.substring(span.start, span.end);
    const precedingText = source.substring(0, span.start);
    const previousLines = precedingText.split('\n');
    // Lines and columns in stacks are one indexed.
    const line = previousLines.length;
    const column = previousLines[previousLines.length - 1].length + 1;

    result.push({
      filePath: path,
      line,
      column,
      locationText,
    });

    callStack = location_tree[callStack.parent];
  }
  // Reverse since we explored the child nodes first
  return result.reverse();
}
/**
 * Extracts the call stack from the location of a failing opcode and the debug metadata.
 * One opcode can point to multiple calls due to inlining.
 */
function getSourceCodeLocationsFromOpcodeLocation(
  opcodeLocation: string,
  debug: DebugInfo,
  files: DebugFileMap,
  brilligFunctionId?: BrilligFunctionId,
): SourceCodeLocation[] {
  let callstack_id = debug.acir_locations[opcodeLocation];
  const brilligLocation = extractBrilligLocation(opcodeLocation);
  if (brilligFunctionId !== undefined && brilligLocation !== undefined) {
    callstack_id = debug.brillig_locations[brilligFunctionId][brilligLocation];
    if (callstack_id === undefined) {
      return [];
    }
  }

  if (callstack_id === undefined) {
    return [];
  }
  const callStack = debug.location_tree.locations[callstack_id];
  return getCallStackFromLocationNode(callStack, debug.location_tree.locations, files);
}

/**
 * Extracts a brillig location from an opcode location.
 */
function extractBrilligLocation(opcodeLocation: string): string | undefined {
  const splitted = opcodeLocation.split('.');
  if (splitted.length === 2) {
    return splitted[1];
  }
  return undefined;
}
