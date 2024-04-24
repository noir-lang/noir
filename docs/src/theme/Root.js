import React from 'react';
import useMatomo from '@site/src/components/Matomo/matomo';
import BrowserOnly from '@docusaurus/BrowserOnly';
import useIsBrowser from '@docusaurus/useIsBrowser';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';

function OptOutForm() {
  useMatomo();

  return (
    <div id="optout-form">
      <p>
        You may choose not to have a unique web analytics cookie identification number assigned to your computer to
        avoid the aggregation and analysis of data collected on this website.
      </p>
      <p>To make that choice, please click below to receive an opt-out cookie.</p>

      <p>
        <input type="checkbox" id="optout" />
        <label htmlFor="optout">
          <strong></strong>
        </label>
      </p>
    </div>
  );
}

export default function Root({ children }) {
  const useIsBrowserValue = useIsBrowser();
  if (!useIsBrowserValue) return <>{children}</>;

  return (
    <>
      <BrowserOnly>{() => <OptOutForm />}</BrowserOnly>
      {children}
    </>
  );
}
