import React, { useState } from 'react';
import styles from './Footer.module.css';
import { isValidEmail } from '@site/src/utils/emailValidation';

// External link icon component
const ExternalLinkIcon = () => (
  <svg width="13.5" height="13.5" aria-hidden="true" viewBox="0 0 24 24" className="iconExternalLink_nPIU">
    <path fill="currentColor" d="M21 13v10h-21v-19h12v2h-10v15h17v-8h2zm3-12h-10.988l4.035 4-6.977 7.07 2.828 2.828 6.977-7.07 4.125 4.172v-11z"/>
  </svg>
);

export default function FooterWrapper(props) {
  const [email, setEmail] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isSubscribed, setIsSubscribed] = useState(false);
  const [successMessage, setSuccessMessage] = useState('');
  const [error, setError] = useState('');

  const handleSubmit = async (e) => {
    e.preventDefault();

    // Clear previous messages
    setError('');

    if (!email.trim()) {
      setError('Email address is required');
      return;
    }

    // Validate email
    if (!isValidEmail(email)) {
      setError('Please provide a valid email address');
      return;
    }

    setIsSubmitting(true);

    try {
      // Track subscription attempt
      if (window.analytics) {
        window.analytics.trackEvent('Email Subscription', 'Attempted', 'footer');
      }

      // Call the Brevo API endpoint
      const response = await fetch('/.netlify/functions/subscribe', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          email: email.trim(),
          source: 'noir-docs-footer'
        }),
      });

      const data = await response.json();

      if (response.ok) {
        if (data.alreadySubscribed) {
          setIsSubscribed(true);
          setSuccessMessage("It looks like you're already subscribed, good for you! 🎉");
          setEmail('');

          if (window.analytics) {
            window.analytics.trackEvent('Email Subscription', 'Already Subscribed', 'footer');
          }
        } else {
          setIsSubscribed(true);
          setSuccessMessage("Thanks for subscribing! 🎉");
          setEmail('');

          if (window.analytics) {
            window.analytics.trackEvent('Email Subscription', 'Successful', 'footer');
          }
        }

        console.log('✅ Subscription response:', data.message);
      } else if (response.status === 429) {
        const retryAfter = data.retryAfter || 60;
        const minutes = Math.ceil(retryAfter / 60);
        throw new Error(`Please wait ${minutes} minute${minutes > 1 ? 's' : ''} before trying again.`);
      } else {
        throw new Error(data.error || 'Subscription failed');
      }

    } catch (err) {
      console.error('Subscription error:', err);
      setError(err.message || 'Failed to subscribe. Please try again.');

      if (window.analytics) {
        window.analytics.trackEvent('Email Subscription', 'Failed', 'footer');
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <footer className="footer footer--dark">
      <div className="container">
        <div className={styles.footerGrid}>

          {/* Left side - Email subscription */}
          <div className={styles.footerLeft}>
            <h3 className={styles.footerTitle}>Stay up to date with Noir</h3>

            {isSubscribed ? (
              <div className={styles.successMessage}>
                <p>{successMessage}</p>
              </div>
            ) : (
              <form onSubmit={handleSubmit} className={styles.subscriptionForm}>
                <div className={styles.inputContainer}>
                  <input
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    placeholder="Enter email"
                    className={styles.emailInput}
                    disabled={isSubmitting}
                  />
                  <button
                    type="submit"
                    disabled={isSubmitting || !email.trim()}
                    className={styles.submitButton}
                  >
                    {isSubmitting ? (
                      <div className={styles.spinner} />
                    ) : (
                      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <path d="M5 12h14M12 5l7 7-7 7"/>
                      </svg>
                    )}
                  </button>
                </div>
                {error && (
                  <div className={styles.errorMessage}>
                    {error}
                  </div>
                )}
              </form>
            )}
          </div>

          {/* Right side - Original Noir footer content */}
          <div className={styles.footerRight}>
            <div className="row footer__links">
              <div className="col footer__col">
                <div className="footer__title">Community</div>
                <ul className="footer__items clean-list">
                  <li className="footer__item">
                    <a className="footer__link-item" href="https://forum.aztec.network/c/noir/7">
                      Noir Forum
                      <ExternalLinkIcon />
                    </a>
                  </li>
                  <li className="footer__item">
                    <a className="footer__link-item" href="https://discord.gg/JtqzkdeQ6G">
                      Discord
                      <ExternalLinkIcon />
                    </a>
                  </li>
                  <li className="footer__item">
                    <a className="footer__link-item" href="https://twitter.com/NoirLang">
                      Twitter
                      <ExternalLinkIcon />
                    </a>
                  </li>
                </ul>
              </div>

              <div className="col footer__col">
                <div className="footer__title">Code</div>
                <ul className="footer__items clean-list">
                  <li className="footer__item">
                    <a className="footer__link-item" href="https://github.com/noir-lang">
                      Noir GitHub
                      <ExternalLinkIcon />
                    </a>
                  </li>
                  <li className="footer__item">
                    <a className="footer__link-item" href="https://github.com/noir-lang/noir/tree/master/docs">
                      Docs GitHub
                      <ExternalLinkIcon />
                    </a>
                  </li>
                </ul>
              </div>
            </div>
          </div>
        </div>

        {/* Bottom copyright - using original Docusaurus styling */}
        <div className="footer__bottom text--center">
          <div className="footer__copyright">
            Noir will be dual licensed under MIT/Apache (Version 2.0).
          </div>
        </div>
      </div>
    </footer>
  );
}