import
  (builtins.fetchTarball {
    name = "nixos-25.05";
    url = "https://github.com/nixos/nixpkgs/archive/tags/25.05.tar.gz";
    sha256 = "sha256:1915r28xc4znrh2vf4rrjnxldw2imysz819gzhk9qlrkqanmfsxd";
  })
  {}
