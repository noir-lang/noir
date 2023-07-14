import { type MemDown, default as memdown } from 'memdown';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;
