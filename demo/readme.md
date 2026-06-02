
## Demo Apps — How to Run

  demo/usb-auth — the only runnable demo

  The server/ and client/ folders are empty (only node_modules, no source files — git only tracks demo/usb-auth). They appear to be remnants from an earlier architecture.

  The only working demo is demo/usb-auth. Run it with:

  ```bash
  cd demo\usb-auth
  npm run dev
  ```

  Then open http://127.0.0.1:5173 in your browser.

  Other useful commands from that directory:

  ┌──────────────────────────┬────────────────────────────────────────────────────────┐
  │         Command          │                      What it does                      │
  ├──────────────────────────┼────────────────────────────────────────────────────────┤
  │ npm run dev              │ Start Vite dev server (hot reload)                     │
  ├──────────────────────────┼────────────────────────────────────────────────────────┤
  │ npm run build            │ Production build to dist/                              │
  ├──────────────────────────┼────────────────────────────────────────────────────────┤
  │ npm test                 │ Unit tests (fields, providers, secret-file)            │
  ├──────────────────────────┼────────────────────────────────────────────────────────┤
  │ npm run test:proof       │ Full ZK proof generation + verification (slow, ~2 min) │
  ├──────────────────────────┼────────────────────────────────────────────────────────┤
  │ npm run generate:circuit │ Recompile main.nr → regenerate src/circuit-artifact.js │
  └──────────────────────────┴────────────────────────────────────────────────────────┘

  The CLI is also runnable directly after npm run generate:circuit:

  ```bash
  # Register a device secret
  node src/cli.js register --out secret.json --pin mypin123 --user alice

  # Generate a proof
  node src/cli.js prove --secret secret.json --pin mypin123 --user alice

  # Verify a saved proof.json (structural check)
  node src/cli.js verify --proof proof.json

  ```

  For the native Rust verifier built earlier:

  ```bash
  # From workspace root
  cargo build -p usb-verifier --release

  # Then:

  .\target\release\usb-verifier.exe --proof proof.json --serial 305441741 --json
  .\target\release\usb-verifier.exe --info   # show embedded circuit identity
  ```
