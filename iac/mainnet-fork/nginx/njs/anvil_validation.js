// Authorizes a request based on the method of the JSON-RPC request body, blocking all cheat codes.
// See https://github.com/nginx/njs-examples?tab=readme-ov-file#authorizing-requests-based-on-request-body-content-http-authorization-request-body
function authorize(r) {
  try {
    if (r.requestText) {
      const body = JSON.parse(r.requestText);
      if (body && body.method) {
        const method = body.method.replace(/\s+/g).toLowerCase();
        if (
          method.startsWith("evm_") ||
          method.startsWith("hardhat_") ||
          method.startsWith("anvil_")
        ) {
          const error = "Restricted method " + method;
          r.error(error);
          r.return(401, JSON.stringify({ error }));
          return;
        }
      }
    }
    r.internalRedirect("@anvil");
  } catch (e) {
    r.error("JSON.parse exception: " + e);
    r.return(400, JSON.stringify({ error: "Error parsing request" }));
  }
}

export default { authorize };
