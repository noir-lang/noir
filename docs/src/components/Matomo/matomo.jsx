import { useEffect, useState } from "react";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import Link from "@docusaurus/Link";
import { useLocation } from "@docusaurus/router";

function getSiteId(env) {
  // Noir site IDs
  if (env == "dev") {
    return "3"; // Keep existing Noir dev ID
  } else if (env == "staging") {
    return "2"; // Keep existing Noir staging ID
  } else {
    return "1"; // Keep existing Noir production ID
  }
}

function pushInstruction(name, ...args) {
  return window._paq.push([name, ...args]);
}

export default function useMatomo() {
  const { siteConfig } = useDocusaurusContext();
  const [showBanner, setShowBanner] = useState(false);
  const location = useLocation();

  const env = siteConfig.customFields.ENV || 'dev';
  const urlBase = "https://noirlang.matomo.cloud/";
  const trackerUrl = `${urlBase}matomo.php`;
  const srcUrl = `${urlBase}matomo.js`;

  window._paq = window._paq || [];

  // Debug logging
  if (typeof window !== 'undefined' && env !== 'prod') {
    console.log('🔍 Matomo Debug:', {
      env,
      siteId: getSiteId(env),
      trackerUrl,
      consentGiven: localStorage.getItem("matomoConsent")
    });
  }

  useEffect(() => {
    const storedConsent = localStorage.getItem("matomoConsent");
    if (storedConsent === null) {
      setShowBanner(true);
    }
  }, []);

  useEffect(() => {
    pushInstruction("setTrackerUrl", trackerUrl);
    pushInstruction("setSiteId", getSiteId(env));
    if (env !== "prod") {
      pushInstruction("setSecureCookie", false);
    }

    const doc = document;
    const scriptElement = doc.createElement("script");
    const scripts = doc.getElementsByTagName("script")[0];

    scriptElement.type = "text/javascript";
    scriptElement.async = true;
    scriptElement.defer = true;
    scriptElement.src = srcUrl;

    if (scripts && scripts.parentNode) {
      scripts.parentNode.insertBefore(scriptElement, scripts);
    }
  }, []);

  useEffect(() => {
    pushInstruction("trackPageView");
  }, [location.pathname]);

  const optIn = () => {
    pushInstruction("rememberConsentGiven");
    localStorage.setItem("matomoConsent", true);
    setShowBanner(false);
  };

  const optOut = () => {
    pushInstruction("forgetConsentGiven");
    localStorage.setItem("matomoConsent", false);
    setShowBanner(false);
  };

  // Add global debug function for console access
  useEffect(() => {
    if (env !== "prod" && typeof window !== 'undefined') {
      window.forceNPS = () => {
        const event = new CustomEvent('forceShowNPS');
        window.dispatchEvent(event);
        console.log('🔧 Forcing NPS widget to show');
      };

      // Clean up on unmount
      return () => {
        delete window.forceNPS;
      };
    }
  }, [env]);

  if (!showBanner) {
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
        </div>
      </div>
    </div>
  );
}