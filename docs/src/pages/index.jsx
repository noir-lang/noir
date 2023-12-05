import React, { lazy, Suspense } from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';

import headerPic from '../../static/img/homepage_header_pic.png';
import { BeatLoader } from 'react-spinners';

const NoirEditor = lazy(() => import('@signorecello/noir_playground'));

const Spinner = () => {
  return (
    <div style={{ textAlign: 'center', marginTop: '4rem' }}>
      <BeatLoader color="#4F3C63" />
    </div>
  );
};

export default function Landing() {
  const [tryIt, setTryIt] = React.useState(false);

  return (
    <Layout>
      <div style={{ minHeight: '100vh' }}>
        <div
          style={{
            display: 'block',
            minHeight: '35vh',
            marginBottom: '4rem',
            backgroundPosition: 'center',
            backgroundSize: 'cover',
            backgroundImage: `url(${headerPic})`,
          }}
          alt="Revolutionizing SNARK proving systems"
        />
        <div className="homepage_layout" style={{ padding: '0 2rem' }}>
          <h1 style={{ textAlign: 'center', marginBottom: '2rem', fontWeight: '500' }} className="hero__title">
            Noir
          </h1>
          <p className="homepage_p">
            Noir is a Domain Specific Language for SNARK proving systems. It has been designed to use any ACIR
            compatible proving system. It's design choices are influenced heavily by Rust and focuses on a simple,
            familiar syntax.
          </p>

          {tryIt && (
            <Suspense fallback={<Spinner />}>
              <Link to="/docs" target="_blank" rel="noopener noreferrer">
                <button className="cta-button button button--primary button--lg homepage_cta">Read the Docs</button>
              </Link>
              <NoirEditor style={{ width: '100%', height: '300px' }} baseUrl="https://play.noir-lang.org" />
            </Suspense>
          )}

          {!tryIt && (
            <div className="homepage_cta_container">
              <Link to="/docs" target="_blank" rel="noopener noreferrer">
                <button className="cta-button button button--primary button--lg homepage_cta">Read the Docs</button>
              </Link>
              <button
                onClick={(e) => setTryIt(!tryIt)}
                className="cta-button button button--secondary button--lg homepage_cta"
              >
                Try it now!
              </button>
            </div>
          )}
        </div>
      </div>
    </Layout>
  );
}
