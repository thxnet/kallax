{ pkgs, }:

pkgs.runCommand "check-format"
{
  buildInputs = with pkgs; [
    shellcheck

    hclfmt
    nixpkgs-fmt
    nodePackages.prettier
    shfmt
    taplo
    treefmt
  ];
} ''
  # Copy source to a writable directory since treefmt needs to write files
  cp -r ${./..} ./source
  chmod -R u+w ./source

  treefmt \
    --allow-missing-formatter \
    --fail-on-change \
    --no-cache \
    -C ./source

  # it worked!
  touch $out
''
