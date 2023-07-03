{ name
, version
, dockerTools
, kallax
, buildEnv
, ...
}:

dockerTools.buildImage {
  inherit name;
  tag = "v${version}";

  copyToRoot = buildEnv {
    name = "image-root";
    paths = [ kallax ];
    pathsToLink = [ "/bin" ];
  };

  config = {
    Entrypoint = [ "${kallax}/bin/kallax" ];
  };
}
