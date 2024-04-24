import { useEffect, useState } from 'react';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';

function getSiteId(siteConfig) {
  const env = siteConfig.customFields.MATOMO_ENV;
  if (env == 'dev') {
    return '3';
  } else if (env == 'staging') {
    return '2';
  } else {
    return '1';
  }
}

export default function useMatomo() {
  const { siteConfig } = useDocusaurusContext();
  const [isOptedOut, setIsOptedOut] = useState(false);

  useEffect(() => {
    window._paq = window._paq || [];
    console.log('effect');
    console.log(window);

    var u = 'https://noirlang.matomo.cloud/';
    const siteId = getSiteId(siteConfig);
    // const secureCookie = siteId === '3' ? false : true;
    // console.log('secureCookie', secureCookie);
    // window._paq.push(['setSecureCookie', secureCookie]);
    window._paq.push(['setTrackerUrl', u + 'matomo.php']);
    window._paq.push(['setSiteId', siteId]);

    var d = document,
      g = d.createElement('script'),
      s = d.getElementsByTagName('script')[0];
    g.async = true;
    g.src = 'https://cdn.matomo.cloud/noirlang.matomo.cloud/matomo.js';
    s.parentNode.insertBefore(g, s);
  }, []);

  useEffect(() => {
    window._paq = window._paq || [];
    window._paq.push(['trackPageView']);
  }, [window.location.href]);

  useEffect(() => {
    window._paq = window._paq || [];
    function setOptOutText(element) {
      console.log(element.checked);
      window._paq.push([
        function () {
          element.checked = !this.isUserOptedOut();
          document.querySelector('label[for=optout] strong').innerText = this.isUserOptedOut()
            ? 'You are currently opted out. Click here to opt in.'
            : 'You are currently opted in. Click here to opt out.';
        },
      ]);
    }

    var optOut = document.getElementById('optout');
    optOut.addEventListener('click', function () {
      if (this.checked) {
        window._paq.push(['forgetUserOptOut']);
      } else {
        window._paq.push(['optUserOut']);
      }
      setOptOutText(optOut);
    });
    setOptOutText(optOut);
  }, [isOptedOut]);

  return <></>;
}
