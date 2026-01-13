// Netlify function for email subscription using Brevo API

const brevo = require('@getbrevo/brevo');

// Email validation function for serverless backend
function isValidEmail(email) {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

  if (!email || typeof email !== 'string') {
    return false;
  }

  if (!emailRegex.test(email)) {
    return false;
  }

  if (email.length > 254) {
    return false;
  }

  const [localPart, domain] = email.split('@');
  if (localPart.length > 64 || domain.length > 253) {
    return false;
  }

  return true;
}

const BREVO_LIST_ID = 101; // Noir docs mailing list ID

// Simple in-memory rate limiting (resets on cold starts)
const rateLimitMap = new Map();
const RATE_LIMIT_WINDOW = 60 * 1000; // 1 minute
const MAX_REQUESTS_PER_WINDOW = 5; // 5 requests per minute per IP
const MAX_EMAIL_REQUESTS_PER_HOUR = 3; // 3 submissions per hour per email

// Cleanup old entries periodically
setInterval(() => {
  const now = Date.now();
  for (const [key, data] of rateLimitMap.entries()) {
    if (now - data.firstRequest > RATE_LIMIT_WINDOW) {
      rateLimitMap.delete(key);
    }
  }
}, RATE_LIMIT_WINDOW);

function isRateLimited(identifier, maxRequests = MAX_REQUESTS_PER_WINDOW) {
  const now = Date.now();
  const key = identifier;

  if (!rateLimitMap.has(key)) {
    rateLimitMap.set(key, { count: 1, firstRequest: now });
    return false;
  }

  const data = rateLimitMap.get(key);

  // Reset if window expired
  if (now - data.firstRequest > RATE_LIMIT_WINDOW) {
    rateLimitMap.set(key, { count: 1, firstRequest: now });
    return false;
  }

  // Check if over limit
  if (data.count >= maxRequests) {
    return true;
  }

  // Increment counter
  data.count++;
  return false;
}

exports.handler = async (event, context) => {
  // Only allow POST requests
  if (event.httpMethod !== 'POST') {
    return {
      statusCode: 405,
      headers: {
        'Content-Type': 'application/json',
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Methods': 'POST',
        'Access-Control-Allow-Headers': 'Content-Type',
      },
      body: JSON.stringify({ error: 'Method not allowed' }),
    };
  }

  // Get client IP for rate limiting
  const clientIP = event.headers['client-ip'] || event.headers['x-forwarded-for'] || 'unknown';

  // Rate limit by IP
  if (isRateLimited(`ip:${clientIP}`)) {
    return {
      statusCode: 429,
      headers: {
        'Content-Type': 'application/json',
        'Access-Control-Allow-Origin': '*',
        'Retry-After': '60',
      },
      body: JSON.stringify({
        error: 'Too many requests. Please try again in a minute.',
        retryAfter: 60,
      }),
    };
  }

  try {
    // Basic request validation
    if (!event.body) {
      return {
        statusCode: 400,
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*',
        },
        body: JSON.stringify({ error: 'Request body is required' }),
      };
    }

    // Limit request body size (prevent large payload attacks)
    if (event.body.length > 1000) {
      return {
        statusCode: 413,
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*',
        },
        body: JSON.stringify({ error: 'Request too large' }),
      };
    }

    const { email, source } = JSON.parse(event.body);

    if (!email || typeof email !== 'string' || !isValidEmail(email)) {
      return {
        statusCode: 400,
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*',
        },
        body: JSON.stringify({ error: 'Please provide a valid email address' }),
      };
    }

    // Normalize and sanitize email
    const normalizedEmail = email.toLowerCase().trim();

    // Additional length check (RFC 5321 max email length)
    if (normalizedEmail.length > 254) {
      return {
        statusCode: 400,
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*',
        },
        body: JSON.stringify({ error: 'Email address is too long' }),
      };
    }

    // Rate limit by email (prevent spam submissions for same email)
    if (isRateLimited(`email:${normalizedEmail}`, MAX_EMAIL_REQUESTS_PER_HOUR)) {
      return {
        statusCode: 429,
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*',
          'Retry-After': '3600',
        },
        body: JSON.stringify({
          error: 'This email has been submitted recently. Please try again later.',
          retryAfter: 3600,
        }),
      };
    }

    // Initialize Brevo API client
    const apiInstance = new brevo.ContactsApi();
    apiInstance.setApiKey(brevo.ContactsApiApiKeys.apiKey, process.env.BREVO_API_KEY);

    try {
      // Create contact in Brevo
      const createContact = new brevo.CreateContact();
      createContact.email = normalizedEmail;
      createContact.listIds = [BREVO_LIST_ID];

      // Add source tracking
      if (source) {
        createContact.attributes = { SOURCE: source };
      }

      await apiInstance.createContact(createContact);
    } catch (brevoError) {
      // Check if contact already exists (Brevo returns 400 with duplicate_parameter code)
      if (
        brevoError &&
        (brevoError.status === 400 || brevoError.status === 409) &&
        brevoError.response?.data?.code === 'duplicate_parameter'
      ) {
        // Contact exists, try to add to list
        try {
          const addToList = new brevo.AddContactToList();
          addToList.emails = [normalizedEmail];

          await apiInstance.addContactToList(BREVO_LIST_ID, addToList);
        } catch (listError) {
          console.error('Error adding existing contact to list:', listError);
          // Contact exists and might already be in the list
          return {
            statusCode: 200,
            headers: {
              'Content-Type': 'application/json',
              'Access-Control-Allow-Origin': '*',
            },
            body: JSON.stringify({
              message: "You're already subscribed! Check your inbox for our latest updates.",
              alreadySubscribed: true,
            }),
          };
        }
      } else {
        // Other Brevo API error
        console.error('Brevo API error:', brevoError);
        throw brevoError;
      }
    }

    return {
      statusCode: 200,
      headers: {
        'Content-Type': 'application/json',
        'Access-Control-Allow-Origin': '*',
      },
      body: JSON.stringify({
        message: 'Successfully subscribed!',
        email: normalizedEmail,
      }),
    };
  } catch (error) {
    console.error('Subscription error:', error);
    return {
      statusCode: 500,
      headers: {
        'Content-Type': 'application/json',
        'Access-Control-Allow-Origin': '*',
      },
      body: JSON.stringify({ error: 'Failed to subscribe. Please try again.' }),
    };
  }
};
