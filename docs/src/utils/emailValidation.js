// Email validation utility for frontend (ES modules)
export function isValidEmail(email) {
  // RFC 5322 compliant email regex (simplified but robust)
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  
  if (!email || typeof email !== 'string') {
    return false;
  }
  
  // Basic format check
  if (!emailRegex.test(email)) {
    return false;
  }
  
  // Length checks (RFC 5321)
  if (email.length > 254) {
    return false;
  }
  
  // Local part (before @) should be max 64 chars
  const [localPart, domain] = email.split('@');
  if (localPart.length > 64) {
    return false;
  }
  
  // Domain part checks
  if (domain.length > 253) {
    return false;
  }
  
  return true;
}