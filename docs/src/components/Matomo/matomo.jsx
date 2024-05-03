import { useEffect, useState } from 'react';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Link from '@docusaurus/Link';

function getSiteId(env) {
  if (env == 'dev') {
    return '3';
  } else if (env == 'staging') {
    return '2';
  } else {
    return '1';
  }
}
function pushInstruction(name, ...args) {
  return window._paq.push([name, ...args]);
}

export default function useMatomo() {
  const { siteConfig } = useDocusaurusContext();
  const [showBanner, setShowBanner] = useState(false);

  const env = siteConfig.customFields.MATOMO_ENV;
  const urlBase = 'https://noirlang.matomo.cloud/';
  const trackerUrl = `${urlBase}matomo.php`;
  const srcUrl = `${urlBase}matomo.js`;

  window._paq = window._paq || [];

  useEffect(() => {
    const storedConsent = localStorage.getItem('matomoConsent');
    if (storedConsent === null) {
      setShowBanner(true);
    }
  }, []);

  useEffect(() => {
    pushInstruction('setTrackerUrl', trackerUrl);
    pushInstruction('setSiteId', getSiteId(env));
    if (env !== 'prod') {
      pushInstruction('setSecureCookie', false);
    }

    const doc = document;
    const scriptElement = doc.createElement('script');
    const scripts = doc.getElementsByTagName('script')[0];

    scriptElement.type = 'text/javascript';
    scriptElement.async = true;
    scriptElement.defer = true;
    scriptElement.src = srcUrl;

    if (scripts && scripts.parentNode) {
      scripts.parentNode.insertBefore(scriptElement, scripts);
    }
  }, []);

  useEffect(() => {
    pushInstruction('trackPageView');
  }, [window.location.href]);

  const optIn = () => {
    pushInstruction('rememberConsentGiven');
    localStorage.setItem('matomoConsent', true);
    setShowBanner(false);
  };

  const optOut = () => {
    pushInstruction('forgetConsentGiven');
    localStorage.setItem('matomoConsent', false);
    setShowBanner(false);
  };

  const debug = () => {
    pushInstruction(function () {
      console.log(this.getRememberedConsent());
      console.log(localStorage.getItem('matomoConsent'));
    });
  };

  const reset = () => {
    pushInstruction('forgetConsentGiven');
    localStorage.clear('matomoConsent');
  };

  if (!showBanner && env === 'dev') {
    return (
      <div id="optout-form">
        <div className="homepage_footer">
          <p>Debugging analytics</p>
          <div className="homepage_cta_footer_container">
            <button className="cta-button button button--secondary button--sm" onClick={debug}>
              Debug
            </button>
            <button className="cta-button button button--secondary button--sm" onClick={reset}>
              Reset
            </button>
          </div>
        </div>
      </div>
    );
  } else if (!showBanner) {
    return null;
  }

  return (
    <div id="optout-form">
      <div className="homepage_footer">
        <p>
          We value your privacy and we only collect statistics and essential cookies. If you'd like to help us improve
          our websites, you can allow cookies for tracking page views, time on site, and other analytics.
          <br />
          <br />
          <Link to="https://aztec.network/privacy-policy/">
            Find out how we use cookies and how you can change your settings.
          </Link>
        </p>
        <div className="homepage_cta_footer_container">
          <button className="cta-button button button--primary button--sm" onClick={optIn}>
            I accept cookies
          </button>
          <button className="cta-button button button--secondary button--sm" onClick={optOut}>
            I refuse cookies
          </button>
          {env === 'dev' && (
            <button className="cta-button button button--secondary button--sm" onClick={debug}>
              Debug
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
