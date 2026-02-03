# Changelog

## [0.3.0](https://github.com/panbanda/dna/compare/dna-v0.2.1...dna-v0.3.0) (2026-02-03)


### ⚠ BREAKING CHANGES

* **cli:** `dna kind add` now requires description as positional argument `--meta` renamed to `--label` (short: `-l`)

### Features

* add ai-safety template and documentation ([0985ba3](https://github.com/panbanda/dna/commit/0985ba3e86b55a0b2ca23e84fbaa0c8c61d4dd9f))
* add constraint and requirement kinds to intent template ([5fcbbf9](https://github.com/panbanda/dna/commit/5fcbbf9605a2272213b6af3553cae1740619c465))
* add versioning and prune commands for storage management ([ced2ba1](https://github.com/panbanda/dna/commit/ced2ba1ccd3c3f97d4d74b5b7e5c5d44a7fc4a4b))
* add versioning and prune commands for storage management ([b7b4608](https://github.com/panbanda/dna/commit/b7b4608be4725b7dfb2d167cdba8ba6bd90dfb79))
* **cli:** add --label, --context flags and improve CLI documentation ([e68131e](https://github.com/panbanda/dna/commit/e68131edac6ce23c9a4b5300b720f811226f9356))
* implement reindex filtering and improve documentation ([8f69ff3](https://github.com/panbanda/dna/commit/8f69ff3d81d7c40fa2ac94f16d31fe9c8e6f924c))
* **kind:** implement dynamic artifact kind system ([0d82a20](https://github.com/panbanda/dna/commit/0d82a20cce87074e023816c8d2c25f9e118c6b57))
* **kind:** implement dynamic artifact kind system ([c471b6f](https://github.com/panbanda/dna/commit/c471b6f5a1fff144439e8f289250889a570c35f7))
* replace --intent-flow with generic --template system ([7452ed2](https://github.com/panbanda/dna/commit/7452ed2e46e8fdf457fb276458a18b4ac4abf967))
* replace --intent-flow with generic --template system ([bb447c1](https://github.com/panbanda/dna/commit/bb447c1dc13cfc85c04f8a687bfa84a483d5dbc1))


### Bug Fixes

* add missing context parameter to all call sites ([2aa7bc4](https://github.com/panbanda/dna/commit/2aa7bc41c1d8e2fb36adaa63732b479141ead6d8))
* address clippy warning for literal format string ([18baf13](https://github.com/panbanda/dna/commit/18baf136341433e82e56881313dadeb315e7ecea))
* address clippy warnings ([55633a0](https://github.com/panbanda/dna/commit/55633a0c8312763960374bfc1e74cabc9f26c039))
* **ci:** remove release build from CI to stay within runner disk limits ([f30eadc](https://github.com/panbanda/dna/commit/f30eadcd2e51ee82a156e021a300b08971489ef5))
* **kind:** address code review feedback ([6edb6d7](https://github.com/panbanda/dna/commit/6edb6d71a396351cc959d01c43e6194a666b51be))
* move SearchResult import to test module ([0ca7b19](https://github.com/panbanda/dna/commit/0ca7b19f802f41f67dfe6167a0aa29bf76e01302))
* **test:** update CLI tests for new kind add positional syntax ([1730a3f](https://github.com/panbanda/dna/commit/1730a3f5dfd39185f505bb3956fba1541f011caf))
* **test:** update list assertions to match actual CLI output ([f55939d](https://github.com/panbanda/dna/commit/f55939d8a3dab78118b3df50df68398a10e48bd4))
* update CLI tests to use --template instead of --intent-flow ([a160e6d](https://github.com/panbanda/dna/commit/a160e6d9b174fbc95d7edc93d4096423fce2b31d))
* use clap ArgGroup for reindex and add missing import ([40f279a](https://github.com/panbanda/dna/commit/40f279a9af1cf5fa8ceedf1ebe625a96ba690305))

## [0.2.1](https://github.com/panbanda/dna/compare/dna-v0.2.0...dna-v0.2.1) (2026-01-29)


### Bug Fixes

* upgrade hf-hub from 0.3 to 0.4 ([83fdd1e](https://github.com/panbanda/dna/commit/83fdd1e1f900e7fe649c127a4c14c5a68ad709eb))
* upgrade hf-hub from 0.3 to 0.4 ([4198403](https://github.com/panbanda/dna/commit/4198403851efc96eccab66d2827dd1651d075d2c))

## [0.2.0](https://github.com/panbanda/dna/compare/dna-v0.1.7...dna-v0.2.0) (2026-01-29)


### ⚠ BREAKING CHANGES

* replace rigid ArtifactType enum with flexible kind string

### Features

* add HTTP server with REST API, MCP, and Lambda support ([37359ad](https://github.com/panbanda/dna/commit/37359ad5bbce02c218b19360350c4a850f36b7e8))
* implement dna-server with REST API, MCP HTTP, and auth ([1d67145](https://github.com/panbanda/dna/commit/1d67145e69fb15d3f817f2da2a844edc9ad9468c))
* integrate rmcp SDK for MCP server ([a5bc05e](https://github.com/panbanda/dna/commit/a5bc05e403d31eb1d7bd2702ba519fdf099146ef))
* replace rigid ArtifactType enum with flexible kind string ([6a75f25](https://github.com/panbanda/dna/commit/6a75f25900a3ab352cf9f7973c0ff6cf3c406b6c))
* replace rigid ArtifactType enum with flexible kind string ([b3b2b9b](https://github.com/panbanda/dna/commit/b3b2b9b1b1497f0b4e449b5050c1726c576decc1))


### Bug Fixes

* address PR [#10](https://github.com/panbanda/dna/issues/10) review findings ([cb3f26b](https://github.com/panbanda/dna/commit/cb3f26b63eabcebb62f2d993d2741d8b7d8e8743))
* **ci:** add handler tests, exclude only local.rs from coverage ([721dc37](https://github.com/panbanda/dna/commit/721dc372c6f033ac8fb93b3a4c76728fc85fcb89))
* **ci:** exclude untestable files from coverage threshold ([a33a352](https://github.com/panbanda/dna/commit/a33a352ce1176d5baa55c80e7aa53782cb574886))
* **ci:** switch release-please to simple strategy for workspace support ([65a2912](https://github.com/panbanda/dna/commit/65a291280972ac3530ab9948fb692a47963f61e6))

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
