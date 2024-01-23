import React, { lazy, Suspense } from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';

import headerPic from '@site/static/img/homepage_header_pic.png';
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
            compatible proving system. Its design choices are influenced heavily by Rust and focuses on a simple,
            familiar syntax.
          </p>
          {!tryIt && (
            <div className="homepage_cta_header_container">
              <div className="homepage_cta_container">
                <Link to="/docs" target="_blank" rel="noopener noreferrer">
                  <button className="cta-button button button--primary button--lg homepage_cta">Read the Docs</button>
                </Link>
                <Link to="/">
                  <button
                    onClick={(e) => setTryIt(!tryIt)}
                    className="cta-button button button--secondary button--lg homepage_cta"
                  >
                    Go to Playground
                  </button>
                </Link>
              </div>
            </div>
          )}
          {tryIt && (
            <Suspense fallback={<Spinner />}>
              <Link to="/docs" target="_blank" rel="noopener noreferrer">
                <button className="cta-button button button--primary button--lg homepage_cta">Read the Docs</button>
              </Link>
              <NoirEditor style={{ width: '100%', height: '300px' }} baseUrl="https://play.noir-lang.org" />
            </Suspense>
          )}

          {!tryIt && (
            <div className="homepage_cta_lj_container">
              <div className="homepage_cta_container">
                <h2 className="homepage_h2">Learn</h2>
                <Link to="/docs/getting_started/installation" target="_blank" rel="noopener noreferrer">
                  <button className="cta-button button button--primary button--lg homepage_cta">Try Noir</button>
                </Link>
                <Link to="/docs" target="_blank" rel="noopener noreferrer">
                  <button className="cta-button button button--secondary button--lg homepage_cta">
                    Noir Cryptography
                  </button>
                </Link>
              </div>
              <div className="homepage_cta_container">
                <h2 className="homepage_h2">Coming from...</h2>
                <Link to="/docs/how_to/solidity_verifier" target="_blank" rel="noopener noreferrer">
                  <button className="cta-button button button--primary button--lg homepage_cta">Solidity</button>
                </Link>
                <Link to="/docs" target="_blank" rel="noopener noreferrer">
                  <button className="cta-button button button--secondary button--lg homepage_cta">Aztec</button>
                </Link>
              </div>
              <div className="homepage_cta_container">
                <h2 className="homepage_h2">New to Everything</h2>
                <Link to="/docs" target="_blank" rel="noopener noreferrer">
                  <button className="cta-button button button--primary button--lg homepage_cta">Noir Basics</button>
                </Link>
                <Link to="/docs/tutorials/noirjs_app" target="_blank" rel="noopener noreferrer">
                  <button className="cta-button button button--secondary button--lg homepage_cta">NoirJS</button>
                </Link>
              </div>
            </div>
          )}
        </div>
      </div>
    </Layout>
  );
}
