import React from 'react';
import useMatomo from '@site/src/components/Matomo/matomo';
import BrowserOnly from '@docusaurus/BrowserOnly';
import useIsBrowser from '@docusaurus/useIsBrowser';
import AskCookbook from '@cookbookdev/docsbot/react';

function OptOutForm() {
  const banner = useMatomo();

  return <>{banner}</>;
}

/** It's a public API key, so it's safe to expose it here. */
const COOKBOOK_PUBLIC_API_KEY = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI2NmIyOWRmZDM3ZWIwYzRiMTVlZGYzMDAiLCJpYXQiOjE3MjI5ODE4ODUsImV4cCI6MjAzODU1Nzg4NX0.19rsZIFNmgVEbpmDjEJ8oPnL7uHYePytf-Ex1pjm2_8';

export default function Root({ children }) {
  const useIsBrowserValue = useIsBrowser();
  if (!useIsBrowserValue) return <>{children}</>;

  return (
    <>
      {children}
      <BrowserOnly>{() => <OptOutForm />}</BrowserOnly>
      <BrowserOnly>{() => <AskCookbook apiKey={COOKBOOK_PUBLIC_API_KEY} />}</BrowserOnly>
    </>
  );
}
