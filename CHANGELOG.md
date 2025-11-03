# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/compare/v0.4.0...v0.4.1) - 2025-11-03

### Other

- *(deps)* bump crate-ci/typos from 1.38.1 to 1.39.0
- *(deps)* update tokio-modbus requirement from 0.16 to 0.17

## [0.4.0](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/compare/v0.3.1...v0.4.0) - 2025-10-20

### Fixed

- fix build `tokio_modbus::prelude::sync::Context` doesn't implement `Debug`

### Other

- update cocogitto action to version 4
- *(deps)* bump crate-ci/typos from 1.37.2 to 1.38.1
- *(deps)* bump crate-ci/typos from 1.36.3 to 1.37.2
- *(deps)* bump crate-ci/typos from 1.36.2 to 1.36.3
- *(deps)* bump crate-ci/typos from 1.35.7 to 1.36.2
- *(deps)* bump crate-ci/typos from 1.35.5 to 1.35.7
- improve feature description in lib.rs

## [0.3.1](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/compare/v0.3.0...v0.3.1) - 2025-08-28

### Other

- Enable doc_cfg feature for docs.rs builds

## [0.3.0](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/compare/v0.2.1...v0.3.0) - 2025-08-27

### Other

- update README.md
- update README.md
- update README.md
- improve readme
- changed features
- refactor
- refactor
- update docs
- fix incorrect error
- update README.md
- change errors
- merge duplicate tokio code
- refactored the code for better clarity and maintainability
- *(deps)* bump crate-ci/typos from 1.35.4 to 1.35.5
- *(deps)* update dialoguer requirement from 0.11 to 0.12
- *(deps)* bump crate-ci/typos from 1.35.3 to 1.35.4
- *(deps)* bump actions/checkout from 4 to 5
- refactored the code for better clarity and maintainability
- *(deps)* bump crate-ci/typos from 1.34.0 to 1.35.3
- cleanup
- *(deps)* bump crate-ci/typos from 1.33.1 to 1.34.0
# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -

## [0.2.1](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/compare/v0.2.0...v0.2.1) - 2025-06-24

### Other

- rename variable
- *(deps)* update flexi_logger requirement from 0.30 to 0.31
- *(deps)* bump crate-ci/typos from 1.32.0 to 1.33.1
- Refactor mqtt config cli defaults ([#47](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/pull/47))

## [v0.2.0](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/compare/b7fd111cca59ccbc2bfb0b2197a6855c4af2a15a..v0.2.0) - 2025-05-28
#### Build system
- create release-plz.yml - ([4b0dc9d](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/4b0dc9d3d5696c35a9826e68090c72912f740f39)) - acpiccolo
#### Miscellaneous Chores
- **(deps)** bump crate-ci/typos from 1.31.1 to 1.32.0 (#46) - ([b7fd111](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/b7fd111cca59ccbc2bfb0b2197a6855c4af2a15a)) - dependabot[bot]
- fix typo - ([2d71dae](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/2d71daed738ff7a7d6ac23d5701044afef68763d)) - acpiccolo
- improved protocol - ([c0abda3](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/c0abda3770dfda2e29ecaaa23ce8f93cba933189)) - acpiccolo

- - -

## [v0.1.1](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/compare/fa3fe37de957f422b54547abaf678a4f9984ea11..v0.1.1) - 2025-04-29
#### Bug Fixes
- fixed wrong protocol sizes - ([9050b06](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/9050b06b2f4fd1be2591839efb6b06eafc45d448)) - acpiccolo
#### Build system
- exclude CHANGELOG.md for typos - ([f95d2d0](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/f95d2d0dadd543431eaa35d653434a72353122f9)) - acpiccolo
- add missing Cargo.toml metadata and change package name - ([11e4000](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/11e4000cae2193065bde4f409d42b7dcc6035339)) - acpiccolo
#### Documentation
- fix API examples - ([74b7fac](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/74b7facd833c2b83fbe39a32a1fc17106c66ecae)) - acpiccolo
- add protocol document - ([66b4b9b](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/66b4b9b2a86982362652aa92d9fc64011a2247a4)) - acpiccolo
#### Features
- add daemon mode with MQTT support - ([384b760](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/384b760d30701e971f6db7cdbfee0591ef604117)) - acpiccolo
#### Miscellaneous Chores
- **(deps)** bump crate-ci/typos from 1.31.0 to 1.31.1 - ([006a3b9](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/006a3b9ceafdb3d456a0774c243617f5a4f07b75)) - dependabot[bot]
- **(deps)** update flexi_logger requirement from 0.29 to 0.30 - ([d40cf0e](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/d40cf0e2ef62b7fd2f5b9fe52196e3e8df8442be)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.30.2 to 1.31.0 - ([e976b16](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/e976b164a5326f9bb795cdef26cbc5b920e5be55)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.30.1 to 1.30.2 - ([d84306a](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/d84306a4cdc55fd61a5997e4509262eb970c76d5)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.30.0 to 1.30.1 - ([e73445b](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/e73445b291a47fdca7abb1fe696704a0319974d9)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.29.9 to 1.30.0 - ([ebc1eb3](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/ebc1eb3fd06c6802acb54876f4fbe9bf3012e440)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.29.7 to 1.29.9 - ([0b52aac](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/0b52aace60a1168dbead8acfde43bfffc5f1da82)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.29.5 to 1.29.7 - ([fa8e342](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/fa8e342999b83c43d4f4a810882de25c30780a20)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.29.4 to 1.29.5 - ([6e21613](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/6e21613e82e5d3f51e1bb1e1fb21bde402c01927)) - dependabot[bot]
- **(deps)** update paho-mqtt requirement from 0.12 to 0.13 - ([86de3c9](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/86de3c9c5c9dfa70aa1de8923fc105abe054e7a6)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.28.3 to 1.29.4 - ([543ec01](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/543ec015999108bab4418a0bc3e0f0de6dfae2f2)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.28.2 to 1.28.3 - ([caeb2db](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/caeb2db7ada522e2150c6659f0189b6e5c02ddfc)) - dependabot[bot]
- **(deps)** update tokio-modbus requirement from 0.15 to 0.16 - ([b240eaa](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/b240eaa209cd2c4e7a971f8fada8274335fa0a39)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.28.1 to 1.28.2 - ([9b96ffb](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/9b96ffb83da3dd75bb7df5382d87db6cfecf9697)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.27.3 to 1.28.1 - ([c642d33](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/c642d33743fe5a6fee99aa6474987262ea42c759)) - dependabot[bot]
- **(deps)** update clap-verbosity-flag requirement from 2 to 3 - ([59b171a](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/59b171a339d7e44bfa02885aa85ae7ff5daf8675)) - dependabot[bot]
- **(deps)** update thiserror requirement from 1 to 2 - ([779c476](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/779c476bd4c3050dc7f506aa5567bff39a3afedd)) - dependabot[bot]
- **(deps)** update tokio-modbus requirement from 0.14 to 0.15 - ([aea2a55](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/aea2a55c3ce65c18a9d9616516703d452a6d5995)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.27.0 to 1.27.3 - ([5ee7da7](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/5ee7da72a3791191a0c6320365e602004eaa8b73)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.26.8 to 1.27.0 - ([5018686](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/5018686b0e4df4bfbc659c4088a6d255a81bfcc2)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.26.0 to 1.26.8 - ([f16016b](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/f16016b38eebba2c729b51275290bbb914984bb0)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.25.0 to 1.26.0 - ([a8ddc1c](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/a8ddc1c6d150b31ee8ed51254900594ced9d69a0)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.24.6 to 1.25.0 - ([bc2985f](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/bc2985fa4cd09418eb591080bac75dbe9844df5f)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.24.5 to 1.24.6 - ([32fdeef](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/32fdeefd834418db326389188cca6a7a333e3e6e)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.24.3 to 1.24.5 - ([8ff50a2](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/8ff50a2a94a82919bfcbc91f2866f210f83f3012)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.24.1 to 1.24.3 - ([5373aaf](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/5373aaf4c661836ef19d93d2ceecc6f3f5319461)) - dependabot[bot]
- **(deps)** update flexi_logger requirement from 0.28 to 0.29 - ([269162e](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/269162e8e77511e68cb2dfb856e00dcbb089fe72)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.23.6 to 1.24.1 - ([020b2f8](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/020b2f8080c71444843d7412f00fed1d2f029016)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.23.5 to 1.23.6 - ([53f2768](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/53f27680e4e51197917979d2b2858371d6818fea)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.23.2 to 1.23.5 - ([4647af2](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/4647af220af2d402b290895b65e02f8932fffdac)) - dependabot[bot]
- **(deps)** update tokio-modbus requirement from 0.13 to 0.14 - ([78e2f44](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/78e2f446ce7b33f8c8c0fe5d1f8a8d92c73486dc)) - acpiccolo
- **(deps)** update tokio-modbus requirement from 0.11 to 0.13 - ([db4e587](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/db4e587ce538aabebd66541151b274b7ecd8fae8)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.23.1 to 1.23.2 - ([7e17661](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/7e176612c90738ea09e220cd105cdf29add5a10c)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.22.9 to 1.23.1 - ([3f9a6a4](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/3f9a6a4edd64e5da83f4d6abb14cbcf23ee510d2)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.22.7 to 1.22.9 - ([f7dc0f3](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/f7dc0f3717eb95239dc1a69fe045a28706799e0b)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.22.3 to 1.22.7 - ([b6fd8aa](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/b6fd8aa7c06183a3ba8666710597040048781212)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.21.0 to 1.22.3 - ([1b83f26](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/1b83f26c958c0469a0adb481d84560eb696203c4)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.20.10 to 1.21.0 - ([110e008](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/110e008f4047f05dd509f5523ac01f64e9d266a1)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.20.9 to 1.20.10 - ([0198f48](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/0198f48646587be0efee39f807d9d8125a93b1c5)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.20.8 to 1.20.9 - ([22acd88](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/22acd88ca19a2f9352ed1b54de15aa58ebcab4c9)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.20.4 to 1.20.8 - ([25d898b](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/25d898b4e7d2c19cc0ebca6b6931c16a1efb1b2b)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.19.0 to 1.20.4 - ([95105b6](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/95105b6f153d6b73b4f47a33c20baec3b785e0da)) - dependabot[bot]
- **(deps)** update tokio-modbus requirement from 0.11 to 0.12 - ([44e05bb](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/44e05bb2ca470518a2d13af221f560ad9343e94e)) - dependabot[bot]
- **(deps)** update flexi_logger requirement from 0.27 to 0.28 - ([f89c3c9](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/f89c3c98f7ce206edaf78e9a7e8d81ff872e0122)) - dependabot[bot]
- **(deps)** bump crate-ci/typos from 1.18.2 to 1.19.0 - ([613ce8e](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/613ce8ee348c74473673704c1a17cebe96a3849d)) - dependabot[bot]
- Validate address decoding - ([24cf0a2](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/24cf0a2588f612f3d9f1875fce8742c498e0bb2c)) - acpiccolo
- improved protocol description (#45) - ([87f81cf](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/87f81cfe3c66aa11b5442bc264b34b6489c6b723)) - acpiccolo
- update edition from 2021 to 2024 - ([5166145](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/5166145413a757041374da404f320d4c5ca085cc)) - acpiccolo
- change positional arguments to named arguments - ([5fea4b6](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/5fea4b61bd380d7a6910f50a5d1846c3f095663f)) - acpiccolo
- changed tokio error from Exception to ExceptionCode - ([a8fb92f](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/a8fb92fe235e2dbcb2841969db9e010f35127113)) - acpiccolo
- update README.md - ([517dd05](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/517dd056592a7afa4983296bb32779b738a23155)) - acpiccolo
- fix typo - ([4fda437](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/4fda437e14dd3525a642e8011b20225cba6d32e8)) - acpiccolo
- update README.md - ([f106624](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/f10662415c15fce9e7a2f141593d801ccaa31d35)) - acpiccolo
- change argument description - ([f847c10](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/f847c1055c09bc39dd47b4c66821bc5b0e6f4db7)) - acpiccolo
- removed obsolete examples - ([e2eca8c](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/e2eca8ce44eb97977f3bf241d2844ffa8decd77a)) - acpiccolo
- changed command line arguments "timeout" and "delay" - ([990a5f1](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/990a5f10fed1136850beb95a7832c26e909b7d36)) - acpiccolo
- improved command line parsing - ([1416616](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/1416616c0550b1fb7e2d9201c72bd0e46ea300a6)) - acpiccolo
- fixed typo - ([f3089ab](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/f3089ab615609738fa52c1915f3fafe28e306a03)) - acpiccolo
- improved error handling - ([9f08bbe](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/9f08bbe3e66603c44629409b2fb4024eaafd63f6)) - acpiccolo
- change error - ([11cb700](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/11cb700ab42098ccdaeabac53a656dbf60f6b804)) - acpiccolo
- implement changed tokio-modbus results - ([f626429](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/f6264295c014217c6bed2c20d4259393f6568c32)) - acpiccolo
- Revert "chore(deps): update tokio-modbus requirement from 0.11 to 0.12" - ([e701227](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/e70122737bc16dee91f2279a1ed2cc2abfabb514)) - acpiccolo
- first commit - ([fa3fe37](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/commit/fa3fe37de957f422b54547abaf678a4f9984ea11)) - acpiccolo

- - -

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).