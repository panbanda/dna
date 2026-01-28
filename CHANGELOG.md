# Changelog

## [0.1.7](https://github.com/panbanda/dna/compare/dna-v0.1.6...dna-v0.1.7) (2026-01-28)


### Features

* add S3 storage support and figment-based config ([2cd9877](https://github.com/panbanda/dna/commit/2cd9877b85c50156b092acf9a3e15047bec0e34f))
* add S3 storage support and figment-based config ([881e74b](https://github.com/panbanda/dna/commit/881e74bdcd5b60cf179a69601263e9322943c496))

## [0.1.6](https://github.com/panbanda/dna/compare/dna-v0.1.5...dna-v0.1.6) (2026-01-28)


### Features

* implement local Candle embeddings and CLI improvements ([af5275f](https://github.com/panbanda/dna/commit/af5275f6e369161eb7f3784d6d8267f906d8ea15))
* implement local Candle embeddings and CLI improvements ([fb55f1d](https://github.com/panbanda/dna/commit/fb55f1d09bcaa7c31165110b878a3577564c433f))

## [0.1.5](https://github.com/panbanda/dna/compare/dna-v0.1.4...dna-v0.1.5) (2026-01-27)


### Bug Fixes

* limit cargo jobs to reduce memory usage during LTO linking ([ac03ae3](https://github.com/panbanda/dna/commit/ac03ae3a02ac9384a04274f9a41f627b0b479989))


### Performance Improvements

* use thin LTO for faster builds with lower memory usage ([aeef7b8](https://github.com/panbanda/dna/commit/aeef7b844eab9154adbfeec3dd752225febc299f))

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
