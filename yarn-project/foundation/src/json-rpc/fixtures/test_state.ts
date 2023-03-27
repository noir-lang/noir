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

export class TestState {
  constructor(private notes: TestNote[]) {}
  getNote(index: number): TestNote {
    return this.notes[index];
  }
  async addNotes(notes: TestNote[]): Promise<TestNote[]> {
    for (const note of notes) {
      this.notes.push(note);
    }
    await sleep(notes.length);
    return this.notes;
  }
}
