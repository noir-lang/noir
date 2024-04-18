/**
 * Represents an error thrown when an operation is interrupted unexpectedly.
 * This custom error class extends the built-in Error class in JavaScript and
 * can be used to handle cases where a process or task is terminated before completion.
 */
export class InterruptError extends Error {}

/**
 * An error thrown when an action times out.
 */
export class TimeoutError extends Error {}
