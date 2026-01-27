# Changelog

## [0.1.4](https://github.com/panbanda/dna/compare/dna-v0.1.3...dna-v0.1.4) (2026-01-27)


### Bug Fixes

* use /mnt for swap file to avoid conflicts ([201d2a1](https://github.com/panbanda/dna/commit/201d2a1f4eab11c62649b498ddf6068fd4f9ccd1))

## [0.1.3](https://github.com/panbanda/dna/compare/dna-v0.1.2...dna-v0.1.3) (2026-01-27)


### Bug Fixes

* restore full LTO and add swap space for ARM Linux builds ([8a33b7c](https://github.com/panbanda/dna/commit/8a33b7cf5a1a5bf737c71508e44d842e91376d92))
* use macos-15-intel for x86_64 builds and thin LTO for faster CI ([8e7f1b0](https://github.com/panbanda/dna/commit/8e7f1b08efcbcb057b404c0338730c8c83677e02))


### Performance Improvements

* use arduino/setup-protoc for faster CI builds ([0df8a3b](https://github.com/panbanda/dna/commit/0df8a3ba886c432cf792d5e72c5ffe8701364500))

## [0.1.2](https://github.com/panbanda/dna/compare/dna-v0.1.1...dna-v0.1.2) (2026-01-27)


### Bug Fixes

* install protoc in release workflow ([3025a0b](https://github.com/panbanda/dna/commit/3025a0b61c2a66f8671b6f6e633fe22faba79e2c))

## [0.1.1](https://github.com/panbanda/dna/compare/dna-v0.1.0...dna-v0.1.1) (2026-01-27)


### Features

* implement DNA CLI with LanceDB and semantic search ([ab7c29d](https://github.com/panbanda/dna/commit/ab7c29d75f19538c57f4c8313ed4b2e7cf1b2201))
* implement MCP tool handlers using services ([e46033b](https://github.com/panbanda/dna/commit/e46033b2810299a66923278660174932751cf661))


### Bug Fixes

* allow dead_code for MCP service fields ([ce8cd45](https://github.com/panbanda/dna/commit/ce8cd4543c6b263833d84e59d272f1399e23efb0))
* CI fixes and add logo ([7e687fb](https://github.com/panbanda/dna/commit/7e687fb47632c4db1e81a88dae88cbfd9535cdc5))
* increase excessive-nesting-threshold to 10 ([0054e0d](https://github.com/panbanda/dna/commit/0054e0dc152755cedf6b39750a3ed92df866a2e1))
