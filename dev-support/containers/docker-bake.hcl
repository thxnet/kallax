variable "TAG" {
  default = "develop"
}

variable "CONTAINER_REGISTRY" {
  default = "ghcr.io/thxnet"
}

group "default" {
  targets = [
    "kallax",
  ]
}

target "kallax" {
  dockerfile = "dev-support/containers/debian/Containerfile"
  target     = "kallax"
  tags       = ["${CONTAINER_REGISTRY}/kallax:${TAG}"]
  platforms  = ["linux/amd64"]
  args = {
    RUSTC_WRAPPER         = "/usr/bin/sccache"
    AWS_ACCESS_KEY_ID     = null
    AWS_SECRET_ACCESS_KEY = null
    SCCACHE_BUCKET        = null
    SCCACHE_ENDPOINT      = null
    SCCACHE_S3_USE_SSL    = null
  }
  contexts = {
    sccache         = "docker-image://ghcr.io/thxnet/ci-containers/sccache:0.5.4"
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
