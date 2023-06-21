variable "TAG" {
  default = "develop"
}

variable "REPOSITORY" {
  default = "ghcr.io"
}

group "default" {
  targets = [
    "kallax",
  ]
}

target "kallax" {
  dockerfile = "dev-support/containers/debian/Containerfile"
  target     = "kallax"
  tags       = ["${REPOSITORY}/thxnet/kallax:${TAG}"]
  platforms  = ["linux/amd64"]
  contexts = {
    substrate-based = "docker-image://ghcr.io/thxnet/ci-containers/substrate-based:build-2023.05.20-41956af"
    ubuntu          = "docker-image://docker.io/library/ubuntu:22.04"
  }
  labels = {
    "description"                     = "Container image for Kallax"
    "io.thxnet.image.type"            = "final"
    "io.thxnet.image.authors"         = "contact@thxlab.io"
    "io.thxnet.image.vendor"          = "thxlab.io"
    "io.thxnet.image.description"     = "Utilities for setting up and managing Substrate-based blockchains"
    "org.opencontainers.image.source" = "https://github.com/thxnet/kallax"
  }
}
