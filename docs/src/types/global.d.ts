// Global type declarations for Aztec Docs

declare global {
  interface Window {
    _paq?: Array<Array<string | number | boolean | null>>;
    analytics?: {
      trackEvent: (category: string, action: string, name?: string, value?: number) => void;
      trackCustomDimension: (index: number, value: string) => void;
      trackGoal: (goalId: number, customRevenue?: number) => void;
    };
  }
}

// Matomo tracking interface
export interface MatomoTracker {
  push: (instruction: Array<string | number | boolean | null>) => void;
  trackEvent: (category: string, action: string, name?: string, value?: number) => void;
  trackGoal: (goalId: number, customRevenue?: number) => void;
  trackCustomDimension: (index: number, value: string) => void;
}

export {};