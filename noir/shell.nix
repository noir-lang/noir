let
  lock = builtins.fromJSON (builtins.readFile ./flake.lock);
  flakeCompatRev = lock.nodes.flake-compat.locked.rev;
  flakeCompatHash = lock.nodes.flake-compat.locked.narHash;
  flakeCompat = fetchTarball {
    url = "https://github.com/edolstra/flake-compat/archive/${flakeCompatRev}.tar.gz";
    sha256 = flakeCompatHash;
  };
  compat = import flakeCompat {
    src = ./.;
  };
in
compat.shellNix
