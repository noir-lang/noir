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
              <Link to="/docs">
                <button className="cta-button button button--primary button--lg homepage_cta">Read the Docs</button>
              </Link>
            </div>
          </div>
        </div>
      </div>
    </Layout>
  );
}
