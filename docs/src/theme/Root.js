import React, { useEffect } from 'react';
import Head from '@docusaurus/Head';
import { useLocation } from '@docusaurus/router';
import useMatomo from '@site/src/components/Matomo/matomo';
import BrowserOnly from '@docusaurus/BrowserOnly';
import useIsBrowser from '@docusaurus/useIsBrowser';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import { AnalyticsManager } from '@site/src/utils/analytics';
import { markdownSiblingForPathname } from '@site/src/llmsTxt';
import NPSWidget from '@site/src/components/NPSWidget';

// Advertise the clean markdown sibling of the current page to agents via a
// `<link rel="alternate" type="text/markdown">` in <head>. Rendered during static site
// generation so the link is present in the served HTML; only emitted for pages that
// actually have a markdown sibling, so the href is never broken.
function MarkdownAlternateLink() {
  const { siteConfig } = useDocusaurusContext();
  const { pathname } = useLocation();
  const href = markdownSiblingForPathname(pathname, siteConfig.baseUrl);
  if (!href) return null;
  return (
    <Head>
      <link rel="alternate" type="text/markdown" href={href} />
    </Head>
  );
}

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
  if (!useIsBrowserValue)
    return (
      <>
        <MarkdownAlternateLink />
        {children}
      </>
    );

  return (
    <>
      <MarkdownAlternateLink />
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
