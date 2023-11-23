import "zx/globals";

// We perform any common setup for zx here to avoid repetition across test files.

if (process.platform == "win32") {
    $.shell = "powershell";
}
  
$.quote = (arg) => arg;

$.verbose = true;
