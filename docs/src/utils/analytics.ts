// Generic Analytics utilities for Aztec Docs
// Provides type-safe Matomo integration with fallback handling

interface AnalyticsConfig {
  enableConsoleLogging?: boolean;
  enableMatomo?: boolean;
  requireConsent?: boolean;
  env?: string; // Environment from siteConfig
}

class AnalyticsManager {
  private config: AnalyticsConfig;
  
  constructor(config: AnalyticsConfig = {}) {
    this.config = {
      enableConsoleLogging: config.env !== 'prod',
      enableMatomo: true,
      requireConsent: true,
      ...config
    };
  }

  /**
   * Check if Matomo is available and user has consented
   */
  private isMatomoAvailable(): boolean {
    if (!this.config.enableMatomo) return false;
    
    // Check if _paq exists
    if (typeof window === 'undefined' || !window._paq) return false;
    
    // Check consent if required
    if (this.config.requireConsent) {
      const consent = localStorage.getItem("matomoConsent");
      return consent === "true";
    }
    
    return true;
  }

  /**
   * Track custom dimension in Matomo
   */
  trackCustomDimension(index: number, value: string): void {
    if (this.config.enableConsoleLogging) {
      console.log(`📊 Custom Dimension: ${index} = ${value}`);
    }

    if (this.isMatomoAvailable()) {
      try {
        window._paq!.push(['setCustomDimension', index, value]);
      } catch (error) {
        console.warn('Matomo custom dimension tracking failed:', error);
      }
    } else {
      if (this.config.enableConsoleLogging) {
        console.warn('Analytics not available (no consent or blocked by adblocker)');
      }
    }
  }

  /**
   * Track goal completion in Matomo
   */
  trackGoal(goalId: number, customRevenue?: number): void {
    if (this.config.enableConsoleLogging) {
      console.log(`🎯 Goal: ${goalId}`, customRevenue ? `(customRevenue: ${customRevenue})` : '');
    }

    if (this.isMatomoAvailable()) {
      try {
        window._paq!.push(['trackGoal', goalId, customRevenue || undefined]);
      } catch (error) {
        console.warn('Matomo goal tracking failed:', error);
      }
    } else {
      if (this.config.enableConsoleLogging) {
        console.warn('Analytics not available (no consent or blocked by adblocker)');
      }
    }
  }

  /**
   * Generic event tracking method
   */
  trackEvent(category: string, action: string, name?: string, value?: number): void {
    if (this.config.enableConsoleLogging) {
      console.log(`📊 Event: ${category} > ${action}`, { name, value });
    }

    if (this.isMatomoAvailable()) {
      try {
        window._paq!.push([
          'trackEvent',
          category,
          action,
          name || undefined,
          value || undefined
        ]);
      } catch (error) {
        console.warn('Matomo event tracking failed:', error);
      }
    } else {
      if (this.config.enableConsoleLogging) {
        console.warn('Analytics not available (no consent or blocked by adblocker)');
      }
    }
  }
}

// Export class and singleton instance
export { AnalyticsManager };
export const analytics = new AnalyticsManager();

// Export types
export type { AnalyticsConfig };