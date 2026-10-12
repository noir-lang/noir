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

  // Additional Aztec Labs tracker (site 23) that receives the same events as
  // the Noir tracker, for cross-domain analytics across the Aztec and Noir
  // web properties.
  const azteclabsTrackerUrl = "https://azteclabs.matomo.cloud/matomo.php";
  const azteclabsSiteId = "23";
  const crossDomainList = [
    "*.aztec.network",
    "*.docs.aztec.network",
    "*.noir-lang.org",
    "*.play.aztec-labs.com",
    "*.testnet.aztec.network",
  ];

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
    // Gate all tracking and cookies behind explicit opt-in. Nothing is sent
    // and no cookies — including the cross-domain visitor id — are set until
    // consent is granted through the banner.
    pushInstruction("requireConsent");
    pushInstruction("setTrackerUrl", trackerUrl);
    pushInstruction("setSiteId", getSiteId(env));
    if (env !== "prod") {
      pushInstruction("setSecureCookie", false);
    }

    // Report to the shared Aztec Labs Matomo instance in addition to the Noir
    // instance. A single matomo.js dispatches every subsequent _paq command —
    // consent, trackPageView and the settings above — to both trackers, so the
    // Aztec Labs tracker is gated by the same consent as the Noir tracker.
    pushInstruction("addTracker", azteclabsTrackerUrl, azteclabsSiteId);
    pushInstruction("setDomains", crossDomainList);
    pushInstruction("enableCrossDomainLinking");
    pushInstruction("enableLinkTracking");

    // Re-apply a stored opt-in so returning visitors who already accepted are
    // tracked without seeing the banner again. Refusal or no prior choice
    // leaves tracking blocked by requireConsent.
    if (localStorage.getItem("matomoConsent") === "true") {
      pushInstruction("rememberConsentGiven");
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
    // Record the page the user opted in on; the initial page view was
    // suppressed by requireConsent before consent existed.
    pushInstruction("trackPageView");
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