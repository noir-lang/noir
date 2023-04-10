import { sleep } from '../../sleep/index.js';

/**
 * Contrived example for JSON RPC tests.
 */
export class TestNote {
  constructor(private data: string) {}
  /**
   * Create a string representation of this class.
   * @returns The string representation.
   */
  toString(): string {
    return this.data;
  }
  /**
   * Creates a string representation of this class.
   * @param data - The data.
   * @returns The string representation.
   */
  static fromString(data: string): TestNote {
    return new TestNote(data);
  }
}

/**
 * Represents a simple state management for TestNote instances.
 * Provides functionality to get a note by index and add notes asynchronously.
 * Primarily used for testing JSON RPC-related functionalities.
 */
export class TestState {
  constructor(private notes: TestNote[]) {}
  /**
   * Retrieve the TestNote instance at the specified index from the notes array.
   * This method allows getting a desired TestNote from the collection of notes
   * maintained by the TestState instance using the provided index value.
   *
   * @param index - The index of the TestNote to be retrieved from the notes array.
   * @returns The TestNote instance corresponding to the given index.
   */
  getNote(index: number): TestNote {
    return this.notes[index];
  }
  /**
   * Add an array of TestNote instances to the current TestState's notes.
   * This function simulates asynchronous behavior by waiting for a duration
   * equal to the number of notes being added. It then returns the updated
   * list of TestNote instances in the TestState.
   *
   * @param notes - An array of TestNote instances to be added.
   * @returns A Promise that resolves to an array of TestNote instances, including the newly added notes.
   */
  async addNotes(notes: TestNote[]): Promise<TestNote[]> {
    for (const note of notes) {
      this.notes.push(note);
    }
    await sleep(notes.length);
    return this.notes;
  }
}
