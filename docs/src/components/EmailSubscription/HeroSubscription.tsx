import React, { useState } from 'react';
import styles from './HeroSubscription.module.css';
import { analytics } from '@site/src/utils/analytics';

interface HeroSubscriptionProps {
  title?: string;
  subtitle?: string;
  placeholder?: string;
  source?: string;
}

export default function HeroSubscription({
  title = "Stay up to date with Noir",
  subtitle = "",
  placeholder = "Enter email",
  source = "hero-footer"
}: HeroSubscriptionProps) {
  const [email, setEmail] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isSubscribed, setIsSubscribed] = useState(false);
  const [successMessage, setSuccessMessage] = useState('');
  const [error, setError] = useState('');

  const validateEmail = (email: string): boolean => {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!email.trim()) {
      setError('Email address is required');
      return;
    }

    if (!validateEmail(email)) {
      setError('Please enter a valid email address');
      return;
    }

    setError('');
    setIsSubmitting(true);

    try {
      // Track subscription attempt
      analytics.trackEvent('Email Subscription', 'Attempted', source);

      // Call the real Brevo API endpoint
      const response = await fetch('/.netlify/functions/subscribe', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          email: email.trim(),
          source: source,
          timestamp: Date.now(),
          url: window.location.href
        }),
      });

      const data = await response.json();

      if (response.ok) {
        if (data.alreadySubscribed) {
          // Handle already subscribed case - show as success with different message
          setIsSubscribed(true);
          setSuccessMessage("It looks like you're already subscribed, good for you!");
          setEmail('');

          // Track already subscribed event
          analytics.trackEvent('Email Subscription', 'Already Subscribed', source);
        } else {
          // Handle new subscription success
          setIsSubscribed(true);
          setSuccessMessage("You'll hear from us soon with the latest updates.");
          setEmail('');

          // Track successful subscription
          analytics.trackEvent('Email Subscription', 'Successful', source);
        }

        console.log('✅ Subscription response:', data.message);
      } else if (response.status === 429) {
        // Rate limited
        const retryAfter = data.retryAfter || 60;
        const minutes = Math.ceil(retryAfter / 60);
        throw new Error(`Please wait ${minutes} minute${minutes > 1 ? 's' : ''} before trying again.`);
      } else {
        throw new Error(data.error || 'Subscription failed');
      }

    } catch (err: any) {
      console.error('Subscription error:', err);
      setError(err.message || 'Failed to subscribe. Please try again.');

      // Track subscription error
      analytics.trackEvent('Email Subscription', 'Failed', source);
    } finally {
      setIsSubmitting(false);
    }
  };

  if (isSubscribed) {
    return (
      <div className={styles.heroContainer}>
        <div className={styles.heroContent}>
          <div className={styles.successState}>
            <h2 className={styles.heroTitle}>Thank you!</h2>
            <p className={styles.successMessage}>{successMessage}</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.heroContainer}>
      <div className={styles.heroContent}>
        <h2 className={styles.heroTitle}>{title}</h2>
        {subtitle && <p className={styles.heroSubtitle}>{subtitle}</p>}
        
        <form onSubmit={handleSubmit} className={styles.heroForm}>
          <div className={styles.inputContainer}>
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder={placeholder}
              className={styles.heroInput}
              disabled={isSubmitting}
              aria-label="Email address"
            />
            <button
              type="submit"
              disabled={isSubmitting || !email.trim()}
              className={styles.heroButton}
              aria-label="Subscribe"
            >
              {isSubmitting ? (
                <div className={styles.loadingSpinner} />
              ) : (
                <svg 
                  width="24" 
                  height="24" 
                  viewBox="0 0 24 24" 
                  fill="none" 
                  stroke="currentColor" 
                  strokeWidth="2"
                >
                  <path d="M5 12h14M12 5l7 7-7 7"/>
                </svg>
              )}
            </button>
          </div>
          
          {error && (
            <div className={styles.errorMessage} role="alert">
              {error}
            </div>
          )}
        </form>
      </div>
    </div>
  );
}