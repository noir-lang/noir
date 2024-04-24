import { useEffect } from 'react';

import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import useIsBrowser from '@docusaurus/useIsBrowser';

export default function useMatomo() {
  const { siteConfig } = useDocusaurusContext();
  const useIsBrowserValue = useIsBrowser();
  if (!useIsBrowserValue) return null;

  useEffect(() => {
    console.log(siteConfig.customFields.MATOMO_CONTAINER);
    var _mtm = (window._mtm = window._mtm || []);
    _mtm.push({ 'mtm.startTime': new Date().getTime(), event: 'mtm.Start' });
    var d = document,
      g = d.createElement('script'),
      s = d.getElementsByTagName('script')[0];
    g.async = true;
    g.src = `https://cdn.matomo.cloud/noirlang.matomo.cloud/container_${siteConfig.customFields.MATOMO_CONTAINER}.js`;
    s.parentNode.insertBefore(g, s);
  }, [window.location.href]);

  return null;
}
