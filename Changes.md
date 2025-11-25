

## [v0.1.12] - 2025-11-25
### :sparkles: New Features
- [`341d413`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/341d413688e46e543045d4a671cc708b28d7b76b) - **async_runtime**: add more async runtime config *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`092445e`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/092445e908e9a90cedce8b706269bb45da02d8cc) - **perf metrics**: add runtime perf monitoring? *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*

### :bug: Bug Fixes
- [`688f216`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/688f2165d683efea8da902a6a792a628179baadf) - **sqlite**: remove sqlite from daemon *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`d155329`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/d15532963a1c98b063ce88fee8d01da8fcb4c2cf) - **dep_loc**: change to &str from &String *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`2979c2a`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/2979c2aea347c2950f8e1d92e35c8b138357d3cd) - **log levels**: lower some log levels *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`902d65e`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/902d65e493211166507cb106fa9edc3ea26db9f3) - **info**: colog lacks trace macro *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`8c1d739`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/8c1d73946fbd39f691d72983b3cd3d558a9ff6ea) - **formatting**: node count formatting for log *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*

### :wrench: Chores
- [`5713b16`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/5713b1690906b0d1198ab6aba657d6213379fc47) - **warnings**: repress warnings on unread fields *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`d47b529`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/d47b5290095dd6bd22445d712838f5005b2601ae) - **CI**: add support for pi4 to CI and bump version *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*


## [v0.1.11] - 2025-10-01
### :sparkles: New Features
- [`3e471f3`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/3e471f335d29ffa931fc3583106b0cc3b202fc8d) - **udev**: add udev rule to disable autosuspend *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`6247f38`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/6247f381e1151ba503befe4fa8224741840929d6) - **udev**: add autostart to systemd service *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*

### :wrench: Chores
- [`38bb4e4`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/38bb4e4847d17ec1640c9fcbc0e4168eca3b6ff7) - **CI**: include udev rules in packaging *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`d8a44fb`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/d8a44fbe01f2f06ec55b73f28afa22c1c3f23e53) - **cargo**: bump version number *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*


## [v0.1.10] - 2025-09-10
### :sparkles: New Features
- [`746ad83`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/746ad8304ce89462d932cd07ad5f550ce487ac3f) - **main**: print out daemon version to log *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`5473117`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/5473117775c61a9de16a024cbec4f58c362aa915) - **mpsc_buffer_size**: use configuration value *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*

### :wrench: Chores
- [`7fccb50`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/7fccb50e32ea2ff79cc0928543713fcc1cdca0f3) - **example_config**: set conservative mpsc_buffer_size *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`fc3617a`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/fc3617ad43f3cf656e53d5225b759864cc65e313) - **main**: remove redundant clippy allow lint *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`43e739d`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/43e739d3b40755ed6eaa911eddb104fb36b1cefc) - **config**: rustfmt *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`4974396`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/4974396d3a4ad9ea1943b720ba2130c0e6e287a5) - **rustfmt**: run rustfmt on repo *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`4b34b8c`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/4b34b8cb15663e36efd3b638742e132526b8b89f) - **cargo**: update patch version *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*


## [v0.1.9] - 2025-09-03
### :bug: Bug Fixes
- [`e302123`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/e302123be1c8e00c1c8db1a77f73b376c5b02ddd) - **systemctl signals**: handle stop signal in main() *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*

### :wrench: Chores
- [`901495c`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/901495cdbb2af4e9bafc42f47f96970b5f65337c) - **cargo toml**: update to version 0.1.9 *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*


## [v0.1.8] - 2025-08-01
### :bug: Bug Fixes
- [`5b9c7da`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/5b9c7da1c0c01077214be573a9bbd23354e1bf00) - **systemd**: 30s delay on beaglebone *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*

### :wrench: Chores
- [`0b0114b`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/0b0114b0a174dcf69d8ae6ca88f5652c2a435154) - **systemd**: add systemd unit and path *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`57f50f5`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/57f50f54221cee57cd9152f69946839de349160a) - **systemd**: update service file *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`50f3e09`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/50f3e0971a13905d651b3beb8d23de9dcd3e6f02) - **systemd service**: just wait for docker daemon *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`a225940`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/a225940e4b9730380b537eabc0664450d4e294c9) - **systemd**: delay daemon, destroy path file *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`f75bae5`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/f75bae563b2d5f5e8b4534102d25cef9f203e774) - **systemd**: fix order of startpre *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`0de0a09`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/0de0a09ee2ff916af4a9610da8add32124cbd968) - **systemd**: set it to 120 *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`2f60fd0`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/2f60fd0ebb45e8f1ea10deda420c51e66fef2007) - **CI**: add files to releases *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`07e8a81`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/07e8a81265381d010c58ae1e99ec2249bb9cf072) - **cargo**: update cargo version *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*


## [v0.1.7] - 2025-07-30
### :bug: Bug Fixes
- [`4e4afc4`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/4e4afc4203a667396b41aa49421227ec17b223e3) - **clippy**: address clippy concerns and fix *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*

### :wrench: Chores
- [`58e8030`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/58e8030c133f8868a8dc24f2f772298f1817b7c6) - **features**: update feature names, default features, and CI *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`1363f6b`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/1363f6bdf0c48e37a0ee36add73f61c6aff6541e) - **README**: update README with build and CI notes *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`6a88af2`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/6a88af2ca7c789f34858584d69aa2b18296de12a) - **features/clippy**: update feature names and clippy lint fixes *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`9e45d6a`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/9e45d6a0da62c0877e86e9dcb0d5f95a1603c9cd) - **toolchain**: set nightly to be default toolchain *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`b7cb156`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/b7cb156b4c76706633da89159589a87c9e48b624) - **Cargo**: bump version number *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*


## [v0.1.6] - 2025-07-22
### :wrench: Chores
- [`6a0b632`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/6a0b632ce5cd53028a45165f23ce2ad919e47ad6) - **cargo**: bump version again for hotfix *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`4ab0310`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/4ab03101acee5331a44a2b4521a0d7a2082f302e) - **CI**: hopefully fix the CI errors on MUSL/ARMv7 *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`3703566`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/3703566b8947a7cac42503b7a3b09f58c9436e94) - **CI**: fix ci syntax error *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*


## [v0.1.5] - 2025-07-22
### :sparkles: New Features
- [`a442c5f`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/a442c5fa1407b378d5e46f7e0922ecb5b00ce88f) - add timestamp to packet count debug logging *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*

### :wrench: Chores
- [`49746c1`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/49746c120d55832f3a203f63db8d92670e901bb7) - **cargo**: rename features *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`91ca9a8`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/91ca9a84a82b6b0e9122bdcaf566195ae9700a12) - **cross**: update cross compilation targets *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`a43d306`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/a43d306d4cc33ae5380f6bfb4972bba6f6880537) - **actions**: update CI config *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`684f608`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/684f608856879ff258a35cfcadcf7129b269a9ec) - **cargo**: update version number *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*


## [v0.1.4] - 2025-06-05
### :sparkles: New Features
- [`1ca3994`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/1ca3994197ea5bc3c4e78e519dd1604738440435) - improve info level logging of db inserts *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`57805b6`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/57805b68646904656698b401fca29a4f019b05f2) - add timestamps to the logging messages *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`82c3bc2`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/82c3bc2d1b290126908ae3e4eddc1665e6dd42a1) - add node received counts for debug builds *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`fc7af24`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/fc7af24dd10100250953c69b1b042085d820fb17) - improve packet counts and timestamps in debug *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`8d81ad2`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/8d81ad29a277bcd562116c39186431a52605c5ed) - improve debug logging *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`d169139`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/d1691392123df39fcb4be325734697e70a4ccf93) - add node number to debug of rx packet count *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`872c665`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/872c665e25d5b546ceb31283299871aaa5c4c878) - add serial connection indicator to packet counts *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*

### :bug: Bug Fixes
- [`95e6b27`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/95e6b27c7644ed5a86e0c919f121940ded06d1a1) - fix sqlite logging integration *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`fd39e9e`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/fd39e9e345684aab6ec725556465e5c2069f4d8e) - lower logging for debug on postgres connections *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`13b6909`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/13b6909ce10f3b42d2e26bc4636b9c253faac84a) - fix borrow checking errors *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`5a5d9a3`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/5a5d9a352508b6cff8adddffa244fdaf4b8057a4) - dumb error with types in rx_count of gatewaystate *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*

### :wrench: Chores
- [`19fe5a3`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/19fe5a30c3e85e8447bbf27306e383745c51d684) - change aggressiveness of example_config *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*


## [v0.1.3] - 2025-05-17
### :bug: Bug Fixes
- [`0b701fe`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/0b701fe5408dd6d45a137d4d486c9212ff1d9aea) - **crash**: hotfix [#24](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/pull/24) bug that persisted in one spot *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*

### :wrench: Chores
- [`8feec00`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/8feec0029707cdbfe169baba5347960c8a5b46f2) - **cargo toml**: update version number for new tag *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*


## [v0.1.2] - 2025-05-17
### :sparkles: New Features
- [`91e8951`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/91e8951bbb821b5ca2f36872b476d98c6bfe6b83) - **GatewayState**: Added logging to GatewayState operations *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*

### :bug: Bug Fixes
- [`7d5a9c7`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/7d5a9c7d8a359c85fdbe5727444cc1ece2a2c59c) - **db inserts**: Fix foreign key insert errors for telemetry *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*

### :recycle: Refactors
- [`2b22440`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/2b224402e3a37b8209e8e01d7e3865ac49489e77) - **packet_handler**: remove channel check for node info serial pkts *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*

### :wrench: Chores
- [`220c6f8`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/220c6f8bd77385a6b401ebf51cea3a2a9affda2a) - **cargo toml**: ensure I do not strip debug builds *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`8b5dc70`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/8b5dc70372cd4d6ada107e86622ace2f5ae93a1e) - **cargo toml**: update version to reflect bugfix *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*


## [v0.1.1] - 2025-05-16
### :wrench: Chores
- [`9ae22e6`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/9ae22e6f1a2c5f0c4093c31e0539bafc409bfc47) - **cargo toml**: update version and add tag to re-run CI *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*

[v0.1.1]: https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/compare/v0.1.0...v0.1.1
[v0.1.2]: https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/compare/v0.1.1...v0.1.2
[v0.1.3]: https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/compare/v0.1.2...v0.1.3
[v0.1.4]: https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/compare/v0.1.3...v0.1.4
[v0.1.5]: https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/compare/v0.1.4...v0.1.5
[v0.1.6]: https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/compare/v0.1.5...v0.1.6
[v0.1.7]: https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/compare/v0.1.6...v0.1.7
[v0.1.8]: https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/compare/v0.1.7...v0.1.8
[v0.1.9]: https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/compare/v0.1.8...v0.1.9
[v0.1.10]: https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/compare/v0.1.9...v0.1.10
[v0.1.11]: https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/compare/v0.1.10...v0.1.11
[v0.1.12]: https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/compare/v0.1.11...v0.1.12
