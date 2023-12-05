# API

This provides a more modular api for importing parts of the library as needed.
The root `index.js` just exposes everything, which can have consequences for startup times and optimizations.
Here we can gradually build up a much more granular api to allow importing precisely what's needed.
This should adopt the opposite philosophy to "export all my child exports".
Every file should (usually) export one thing, and the file/directory structure should be reflected in package.json exports.
