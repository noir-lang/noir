import React from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';

import headerPic from '@site/static/img/homepage_header_pic.png';

export default function Landing() {
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

          <div className="homepage_cta_header_container">
            <div className="homepage_cta_container">
              <Link to="/docs" target="_blank" rel="noopener noreferrer">
                <button className="cta-button button button--primary button--lg homepage_cta">Read the Docs</button>
              </Link>
            </div>
          </div>

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
              <Link to="/docs/how_to/how-to-solidity-verifier" target="_blank" rel="noopener noreferrer">
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
        </div>
      </div>
    </Layout>
  );
}
