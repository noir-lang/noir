import { NestedProcessReturnValues } from '@aztec/circuit-types';

import type { ExecutionResult } from '../client/execution_result.js';
import type { PublicExecutionResult } from '../public/execution.js';

/**
 * Recursively accummulate the return values of a call result and its nested executions,
 * so they can be retrieved in order.
 * @param executionResult
 * @returns
 */
export function accumulateReturnValues(
  executionResult: PublicExecutionResult | ExecutionResult,
): NestedProcessReturnValues {
  const acc = new NestedProcessReturnValues(executionResult.returnValues);
  acc.nested = executionResult.nestedExecutions.map(nestedExecution => accumulateReturnValues(nestedExecution));
  return acc;
}
