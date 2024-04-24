import React from 'react';
import Footer from '@theme-original/Footer';
import useMatomo from '@site/src/components/Matomo/matomo';

export default function FooterWrapper(props) {
  useMatomo();

  return (
    <>
      <Footer {...props} />
    </>
  );
}
