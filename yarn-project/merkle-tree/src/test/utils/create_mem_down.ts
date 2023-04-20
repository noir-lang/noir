import { default as memdown, type MemDown } from 'memdown';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;
