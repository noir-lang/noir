import React from 'react';
import useMatomo from '@site/src/components/Matomo/matomo';
import BrowserOnly from '@docusaurus/BrowserOnly';
import useIsBrowser from '@docusaurus/useIsBrowser';

function OptOutForm() {
  const banner = useMatomo();

  return <>{banner}</>;
}

export default function Root({ children }) {
  const useIsBrowserValue = useIsBrowser();
  if (!useIsBrowserValue) return <>{children}</>;

  return (
    <>
      {children}
      <BrowserOnly>{() => <OptOutForm />}</BrowserOnly>
    </>
  );
}
