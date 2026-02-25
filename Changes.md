

## [v0.3.0] - 2026-02-25
### :wrench: Chores
- [`83bb3b3`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/83bb3b379cc2b6da560170524a175619a7c7b037) - **udev**: update udev rules, need to test on Pis *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`9eeae0b`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/9eeae0bc1f00b34098d86ba66acd48bc461b2cb2) - **udev and systemd**: update udev rules and systemd rules *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`6714ce8`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/6714ce801ce4fb7abde9811e7def2c97bdb1eb45) - **install script**: update install script *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`0dbfc29`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/0dbfc29df8023b9ad124314b107e032b1d6418d6) - **CI**: update github workflow *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`cc56eba`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/cc56eba27e071488ca93c4435885d6518d0eba50) - **cargo**: update cargo toml version *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*


## [v0.2.2] - 2026-01-21
### :wrench: Chores
- [`67b33b0`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/67b33b089f089d0c307fd816c9606aa80cf7696f) - **cargo**: fix alpine feature *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`af0dfe3`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/af0dfe39673fc2c977e1c3ab3b5e30fac54b24a3) - **CI**: bump versions *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*


## [v0.2.1] - 2026-01-21
### :wrench: Chores
- [`6a08143`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/6a08143bbe5cfb85b4d910a6b50ac75c4de41fde) - **CI**: fix config.toml, bump cargo version *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*


## [v0.2.0] - 2026-01-21
### :sparkles: New Features
- [`0452bac`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/0452bac6b51295a4c99e088d550cbd56069b1590) - **granular metrics**: add granular metrics as requested *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`90ea3f0`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/90ea3f022aaa700d15249861b266e9d8ccac9947) - **debug messaging**: remove more unwraps *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`11fb6e7`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/11fb6e781540bdbc479b202b98db09a4c4f6ca7d) - **packet_handler**: add packet timestamps? *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`607cf7a`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/607cf7a7f419c85f6c80b94ccd455c3ccf5f5732) - **types**: begin work simplifying types *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`8c3963e`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/8c3963e62541974db20feafee6c4615e53c43e6f) - **box channels**: reduce memory by boxing channel messages? *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`5d62367`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/5d62367990a88881268409ca7cf5e9de3f99b907) - **use raw sqlx**: not at feature parity yet *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`d92ee71`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/d92ee71c4e2042416e525b1d8d7f8d55cbe46a60) - **perf metrics**: add runtime performance monitoring *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`e048bc9`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/e048bc91c29f8091c0bdd42acadb5b9688ab40a7) - **state**: add state and debug messages *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`a1dc852`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/a1dc8527acf4dfdf57c0e7106d4af5daa45fac83) - **state**: keep track of serial node *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`6be4502`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/6be45026e2b635119d2a4e7d72e0c86fe2f9407d) - **workers**: use works for incoming FromRadio *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`8b14c84`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/8b14c8466e9b79fd85981d8d8a9bed534af71f24) - **tokio**: try channels again *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`e3060e5`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/e3060e5d30a296e0c060dd1253e6c86c6229e00a) - **tokio tracing**: add better tracing to tokio *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`30cd516`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/30cd516bf9bc4d1c496e3644f4d13da311ce803f) - **profiling**: cargo profiling profile *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`e258693`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/e258693f5c8a4815e08622f617b9e6fc143717a1) - **simple tokio**: remove async config use *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`b4137bd`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/b4137bdb2c70fb5994719729133dcdd8edccd437) - **state**: only use state messages in "debug" feature *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`488d4ab`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/488d4ab89f778d2831a5a715190c941dc73ac4d2) - **state**: only print nodes with > 0 packets received *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`c4cad9d`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/c4cad9d32b767069950abcabde3b649fa3335873) - **log**: add logging oncecell and formatting fixes *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`7b82222`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/7b82222f4c76d099ca33f4c24fd2e21ed665fee9) - **locks**: use tokio mutexes instead *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`8b4fae3`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/8b4fae36468e0c40a43550eaa7d9f0ee6eafd14c) - **logging**: use macro logging for less heap? *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`5dd822a`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/5dd822aff521dc32f20abf11e9fc6502ac0d35fb) - **log_perf**: gate performance logging of tokio runtime *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`111b384`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/111b3848fa827044f13440556b2f85ba4ce3873b) - **signal handling**: use tokio's instead *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`ace44d4`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/ace44d4d6e791c7c0fe6a52919ba999e608a4a72) - **config**: reduce heap usage in postgres connection url *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`a069d6f`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/a069d6fcc03225a5ea621415e2db49eb7f5adf5d) - **config**: read XDG config path for config file *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`6d5bda6`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/6d5bda6ef72c92e02a0076c9f68980dcdd19ff16) - **XdgApp static**: XDG APP global for use *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`e5fcde0`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/e5fcde056d510d730591509721e3c5647c0df48d) - **inserts**: add inserts for position packets *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`319c98c`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/319c98c31975f818c982470269d2f638dfd2c9a5) - **inserts**: directly insert from borrowed packet data *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`66b3fc7`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/66b3fc79fdfe720c5438217598a4a7e062095a09) - **inserts**: add all nodeinfo inserts for devicemetrics *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`5f4fe3c`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/5f4fe3cc12e4dea2300d937398ce13fdd2a838d7) - **inserts**: more direct borrowed data inserts *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`beb138d`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/beb138d964b57998a6f69ca3018f65bb0c40ca6b) - **dbops with borrows**: all database operations borrow data *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`a7fb14f`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/a7fb14fcfe44eaa21ed7ccddb94feb4b41c7788e) - **state**: use Display to reduce allocation in printing *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`6f0c9a7`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/6f0c9a70a93fc78774623f132b75e4a077ed30b3) - **serial**: reduce heap allocations *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`309b9a1`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/309b9a1b39aa9f7e903f870843fbf0ba03c9cc61) - **powermetrics**: add powermetrics inserts *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`554485f`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/554485f740d881c27623d3bf03a02492aacf957e) - **packet_handler**: add tracing to telemetry types *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`96340a1`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/96340a1cc33b9e07dbd6a5edc4c99a848e9ed836) - **updates**: add error messages before updates *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`2a81bcb`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/2a81bcbccf02044986e3354030c532f45dc3eb7a) - **debug**: error printing with contexts *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`98d0388`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/98d0388ef149eac31dcf93882d8c7cdb2f896ae8) - **trace**: vary error messaging with trace feature *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`2e90b60`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/2e90b60fff60c9039597d561c71e3e3b1f00bf8a) - **installer**: install script WIP for rpi4 *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*

### :bug: Bug Fixes
- [`ed3d132`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/ed3d1327645cc9851196f6eae6b7a518ab43433a) - **models**: Remove unnecessary derives *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`311e373`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/311e3732016daa9008e5a8f51f893b0193e1b719) - **log level**: colog lacks tracing *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`d94688a`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/d94688a47f9b952d13f76b8ffb8d2a6f99ca3145) - **packets**: check for telemetry channel before processing *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`88b70aa`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/88b70aaf8d47dcb2af44bf2888e18a85d06b340c) - **tokio channel**: clone sender *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`8875f69`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/8875f6980f725df9d6a6af5d228a4d5e6e8dd235) - **tokio tx**: spawn handle? *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`640f917`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/640f9178d9134dcf89fd8cd7e78bcf0428c6e5ff) - **panic**: remove panic for non-existent user *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`307a58c`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/307a58c414622f3f04cb0aa371569205451c38fc) - **insert panics**: only convert types if user is not none *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`e673b9f`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/e673b9fdda50017b0f8222279e266f36fe900a8a) - **tracing**: add back rustc-demangler *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`4afa22a`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/4afa22aae814645f973156eb1e0540bf7c954430) - **state logging**: remove duplicate date *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`479468e`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/479468ef9528df709839cf665d154c227ca04395) - **variables**: remove unused variables *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`9a0bea1`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/9a0bea11a97108ad540594b053f1f9cbef9161cf) - **imports**: remove unused imports *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`28fbd6d`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/28fbd6d2b7c309b84ccdecb17758168319b16b26) - **redo mpsc strat**: have packet_handler receive serial stream *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`898f5ce`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/898f5ce30dd5ea668ec001ea4f6d746a5197ca12) - **box FromRadio**: box to align with upstream *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`39ad57a`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/39ad57a598c4e5755ad06e3f46f65674b345c0c7) - **task**: packet_handler smaller task with refs? *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`694d197`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/694d1978737a15c88a80f56cbc70985fe5c5e4e3) - **imports**: align with features *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`770088a`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/770088aef150cb2b166cc7dae13b23e7ae2ca413) - **state**: remove unnecessary allocation *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`6562257`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/65622571fabd776235a2ffc94dd09daf8334b0b1) - **log**: Once instead of OnceCell, String with capacity, gate log_perf *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`2f30812`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/2f30812d6bcb1a191188ad79e500479bdceff45b) - **DEPLOYMENT_LOCATION**: update to static str, clippy fmt *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`9509804`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/9509804f5ebb341677210fd629f06bd62f1b785d) - **position packets**: use timestamp in packet *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`97c8443`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/97c844323c8f757000625c551893388fcf1b3dd1) - **serial connection**: close connection on disconnect *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`6094130`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/609413014de431fdd8a0564274a38967358be617) - **docs**: fix doc comments for AirQualityMetrics insert *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`d5b19a2`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/d5b19a2e543324b217158c52a3fe695132c67366) - **clones**: remove unnecessary clones *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`e463749`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/e463749d2bacf61b6e3e0136971751d12222dcc9) - **state**: insert to local state after db inserts *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`1ae170a`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/1ae170aa3c0e2a63d43718dfeb36c43503c53d78) - **powermetrics**: add powermetrics file *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`b1b5819`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/b1b5819ddc74c1617238023d6de85977173db5a9) - **exit**: do not print StreamApi on disconnect *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`4d91ab9`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/4d91ab9bc2c81dcf708d2f770ba0f0c25921cddf) - **clippy**: no unused variable *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*

### :recycle: Refactors
- [`92c2607`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/92c26076bb8572674ed3426b16506bdaefd4172c) - **util**: seperate state from types *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`f26649c`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/f26649cca381c55c821bcaf33c9da06e35e6ca84) - **process_packet**: reduce mem and complexity *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`5ddb4f1`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/5ddb4f118d7a34ac66e9c41e2119dbab608dfce4) - **use sqlx**: improve codebase *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`f6f450f`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/f6f450fc3c0a326a681a30500962002ef28b72f0) - **add default derive**: for sql rows *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`f784cd1`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/f784cd153ec69c204de8e3a9a51d95a61c5ab2c3) - **timestamp**: move function to more appropriate module *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*

### :wrench: Chores
- [`ba0133c`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/ba0133cc92aa893474fb4f9a39bcb51371a0705c) - **update typing**: use `Cow<'_, str>` instead of `String` *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`0ca6536`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/0ca6536a07322870dde926f1a1031c6b21816baa) - **strings**: change more strings to strs *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`d6b3443`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/d6b344365104dab2b51ff4866b8c5b0ae63e9e51) - **eliding, boxing**: box values and elide lifetimes *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`e71493a`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/e71493a1efefa6e6ad89679d34724df0364c2519) - **boxing**: Remove boxing for now, think more *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`b0974b0`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/b0974b014618ae0448c3bf603f74e28e7c75ab7e) - **add types**: forgot earlier *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`e2579f1`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/e2579f107e105020623984eefba5d2c6943c47bc) - **Cargo**: update dependencies and remove default features *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`3e48f01`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/3e48f012a51697184ffd2d060987c654f20b8f12) - **clippy**: override too many arguments warning *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`b4136fe`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/b4136fe17efe0c41f133444f4f29948cf5771d8e) - **default build**: add tracing to default build *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`fc0ea6d`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/fc0ea6d18e5014150b7c24175325117e1cba1e6b) - **cargo**: add profiling build and fmt *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`9e6db31`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/9e6db31a4d8ad806a80d58580024cf9351ca6930) - **remove mpsc**: try heaptrack without it *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`ebdf90e`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/ebdf90ec3064887d7a83d0801f9c69ccaefdbdd6) - **state**: User::default() in state function *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`4bd6846`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/4bd6846497e941d32d12060d627ad1830e2f5df5) - **cargo**: serde_json use std *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`024b363`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/024b363de0460418fc63312865e97b8b0121e27b) - **cargo**: re-add std to crates, add rt-multi-thread and parking_lot *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`1c4ae69`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/1c4ae69de7bf2f6515f385fb548c8f28a4b3ab2e) - **cargo**: fmt and remove rustc-demangle *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`beb8617`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/beb86177facca69a79d9a3f59e0cd622fb9705d4) - **cargo**: update features *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`a587d5f`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/a587d5f14102626bffd826ae9a87681af21b9031) - **feature colog**: implement colog repo wide *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`7538c2e`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/7538c2ef9e8144ed4b2e8d0c990fbabeecdc82aa) - **cargo**: format cargo.toml *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`65a586f`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/65a586f5b10e78638e974227c0805b24c78a350a) - **cargo**: no-default-features on deps again *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`eb66517`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/eb665175238ad4250addd1833b412ec95cb535d4) - **types**: delete types.rs *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`b1adbaa`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/b1adbaaa58a5cbd5a768784a607b533e67028294) - **state**: remove fake ids for packets since FromRadio has ids *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`569079e`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/569079e2187b52c53fbcd6b98d77672188fb421e) - **cargo**: update packages *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`eeac0a4`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/eeac0a482778ead13856a7fbfd113636cdea6b81) - **cargo**: update shorthands to have debug *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`c42012f`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/c42012f853f419955b0a2f196a5c3b5e053bcd6a) - **clippy**: fix warnings in packet_handler *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`07337b0`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/07337b075476167544ac746b807cb929c4cf9b76) - **sqlx**: update sqlx jsons *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`4962033`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/49620334d8c874234b85589af9d1b2c4bea24554) - **cargo config**: unset DATABASE_URL for offline builds *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`a2dcfd5`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/a2dcfd5846a6e71639ccf8832b802cf4689894c7) - **CI**: remove Zfmt-debug=none *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`b89580d`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/b89580d990d2a712fa5aeca9a3b0ca4dcfd95df6) - **CI**: add install script to builds *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*
- [`cd9a6ae`](https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/commit/cd9a6ae9477314a3db5978c3b5dbbaadf271c812) - **cargo**: update release number *(commit by [@gatlinnewhouse](https://github.com/gatlinnewhouse))*


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
[v0.2.0]: https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/compare/v0.1.12...v0.2.0
[v0.2.1]: https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/compare/v0.2.0...v0.2.1
[v0.2.2]: https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/compare/v0.2.1...v0.2.2
[v0.3.0]: https://github.com/coffee-and-telesense/meshtastic-telemetry-daemon-rs/compare/v0.2.2...v0.3.0
