{ pkgs, }:

pkgs.runCommandNoCC "check-format"
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
  treefmt \
    --allow-missing-formatter \
    --fail-on-change \
    --no-cache \
    --formatters \
      prettier \
      nix \
      shell \
      hcl \
      toml \
    -C ${./..}

  # it worked!
  touch $out
''
