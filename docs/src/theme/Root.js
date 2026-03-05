import React, { useEffect } from 'react';
import useMatomo from '@site/src/components/Matomo/matomo';
import BrowserOnly from '@docusaurus/BrowserOnly';
import useIsBrowser from '@docusaurus/useIsBrowser';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import { AnalyticsManager } from '@site/src/utils/analytics';
import NPSWidget from '@site/src/components/NPSWidget';

function OptOutForm() {
  const banner = useMatomo();

  return <>{banner}</>;
}

function AnalyticsProvider({ children }) {
  const { siteConfig } = useDocusaurusContext();
  const env = siteConfig.customFields.ENV;

  useEffect(() => {
    const analytics = new AnalyticsManager({ env });

    if (typeof window !== 'undefined') {
      window.analytics = {
        trackEvent: analytics.trackEvent.bind(analytics),
        trackCustomDimension: analytics.trackCustomDimension.bind(analytics),
        trackGoal: analytics.trackGoal.bind(analytics),
      };
    }
  }, [env]);

  return children;
}

export default function Root({ children }) {
  const useIsBrowserValue = useIsBrowser();
  if (!useIsBrowserValue) return <>{children}</>;

  return (
    <>
      <AnalyticsProvider>
        {children}
        <BrowserOnly>
          {() => (
            <>
              <OptOutForm />
              <NPSWidget />
            </>
          )}
        </BrowserOnly>
      </AnalyticsProvider>
    </>
  );
}
