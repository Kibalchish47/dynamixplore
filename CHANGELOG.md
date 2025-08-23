# Changelog

## [0.6.0](https://github.com/Kibalchish47/dynamixplore/compare/dynamixplore-v0.5.0...dynamixplore-v0.6.0) (2025-08-23)


### Features

* 5 dynamical systems for benchmarking ([61e6024](https://github.com/Kibalchish47/dynamixplore/commit/61e60244c1993fd460a92bfd5278437b29b45f1c))
* fully implemented and generated benchmarks ([30901e8](https://github.com/Kibalchish47/dynamixplore/commit/30901e87737ab12ddc1dbcf0191b292c0034037f))
* helper functions in `integrators.rs` ([5b93e68](https://github.com/Kibalchish47/dynamixplore/commit/5b93e68cf3d92a7760ce610caca3aaae3826c6bd))
* implemented various integrator steppers ([e79a56e](https://github.com/Kibalchish47/dynamixplore/commit/e79a56ef443042d6778716ad1f661e813c2dfcc6))
* modified  (new architecture) ([d26d903](https://github.com/Kibalchish47/dynamixplore/commit/d26d9036eb28e18ef424f8139aeba9dc93b48466))
* PyO3 Class Definitions for Python API ([aa9389c](https://github.com/Kibalchish47/dynamixplore/commit/aa9389cee1bcad9245d94a3eb072e581cec49551))
* re-implemented  module ([90b3c96](https://github.com/Kibalchish47/dynamixplore/commit/90b3c9614d8918f76f3147315b32463f48f3da40))
* re-implemented  module ([28d91d9](https://github.com/Kibalchish47/dynamixplore/commit/28d91d9a3141d238879c91d92918868f82aa4ced))
* re-implemented `lyapunov.rs` module ([100fd3a](https://github.com/Kibalchish47/dynamixplore/commit/100fd3a4c5dcbfbc206a2547a18fadddf15f4603))
* re-implemented adaptive integration loop ([cdb0abd](https://github.com/Kibalchish47/dynamixplore/commit/cdb0abdc0d202b29c65f05e145471997fd559207))
* re-implemented explicit integration loop ([d70ef1d](https://github.com/Kibalchish47/dynamixplore/commit/d70ef1d68b629a2bbbd960232518f68f4b64d127))
* re-implemented implicit integration loop ([86498d4](https://github.com/Kibalchish47/dynamixplore/commit/86498d4a38d27bbf9f01cb044d04dbd82a6b0add))
* reworked program architecture ([1596aea](https://github.com/Kibalchish47/dynamixplore/commit/1596aea56448475474f80406168f69f035cd10c8))


### Bug Fixes

* `invariant_measure()` performance fix + new benchmark ([13e9ca9](https://github.com/Kibalchish47/dynamixplore/commit/13e9ca96afa0e4f281acda1284abc1da160c2efa))
* attempted to fix  issue ([8c327a5](https://github.com/Kibalchish47/dynamixplore/commit/8c327a51d4a3c5afd5efb92c60f10698540f4bb0))
* Deadlock caused by using GIL for calling`dynamics_func` ([2d5659e](https://github.com/Kibalchish47/dynamixplore/commit/2d5659e1f5e471ef5704ec2e4c29c8a61a65747e))
* dealing with  compilation errors ([ad17179](https://github.com/Kibalchish47/dynamixplore/commit/ad17179a1dfe3950cc5ec79c466787c27c9480ca))
* fixed Rust modules' compilation errors ([c74c6aa](https://github.com/Kibalchish47/dynamixplore/commit/c74c6aaa792a791c42eedf1caa10c3244d11af52))
* modified Python layer to fit with Rust ([294be8d](https://github.com/Kibalchish47/dynamixplore/commit/294be8d0f5358ace1b65fb5eaff9b77cc6853af5))
* re-implemented ([2a84731](https://github.com/Kibalchish47/dynamixplore/commit/2a847312c7e04a0d1f354aa7729a2ca9437aaf7f))
* re-implemented ([e892ee7](https://github.com/Kibalchish47/dynamixplore/commit/e892ee7168ad7f228af3eab67b363b1cb3efe857))
* re-implemented tests to align with new code ([05b066d](https://github.com/Kibalchish47/dynamixplore/commit/05b066d2af2be32b41279c93240016b7629548ba))


### Documentation

* implemented analysis benchmarks ([dcb32a9](https://github.com/Kibalchish47/dynamixplore/commit/dcb32a93f4eb63ac95019c9d8538831c157c06ba))
* implemented simulation benchmarks ([076b96e](https://github.com/Kibalchish47/dynamixplore/commit/076b96e0c23a4d9ecc2a745ea63d80f2758c1b9d))

## [0.5.0](https://github.com/Kibalchish47/dynamixplore/compare/dynamixplore-v0.4.0...dynamixplore-v0.5.0) (2025-08-13)


### Features

* implemented dx_core demo script ([a9ecade](https://github.com/Kibalchish47/dynamixplore/commit/a9ecade82399815543c8cbcbc0068252bd73e115))


### Documentation

* configured `tests.yml` ([c2c1d88](https://github.com/Kibalchish47/dynamixplore/commit/c2c1d88696274fa51328d2fa740e0c4100a9b923))
* configured Sphinx ([9cbd618](https://github.com/Kibalchish47/dynamixplore/commit/9cbd618e39cbaf1a2fc25e2d3d7e0f76525a5a81))
* created basic documentation ([03c827e](https://github.com/Kibalchish47/dynamixplore/commit/03c827eee083a2e5eb709969fa857ec2ba7f2dc0))
* customizing Sphinx setup ([ab67549](https://github.com/Kibalchish47/dynamixplore/commit/ab6754907e31158fc1806b88eca1d21ff4589d3f))
* fixed `CI.yml` + added benchmark files ([7a179a9](https://github.com/Kibalchish47/dynamixplore/commit/7a179a926af9dd7ce6dac6683966186e4433173a))
* updated GitHub workflow ([29d7614](https://github.com/Kibalchish47/dynamixplore/commit/29d761456f7d596f971363c7f8bc4c1f24a14512))
* wrote analysis tests ([c7683d6](https://github.com/Kibalchish47/dynamixplore/commit/c7683d6c86236598664423944fbb99678c807ddc))
* wrote simulation tests ([580ccc5](https://github.com/Kibalchish47/dynamixplore/commit/580ccc50caa6edcfa0222bdbe3492cb226319d04))

## [0.4.0](https://github.com/Kibalchish47/dynamixplore/compare/dynamixplore-v0.3.0...dynamixplore-v0.4.0) (2025-08-12)


### Features

* average results, convert, and return ([7802a0a](https://github.com/Kibalchish47/dynamixplore/commit/7802a0a253be4a9edc97d9e3f69f11ffff2d7fb7))
* implemented approximate entropy ([0d61281](https://github.com/Kibalchish47/dynamixplore/commit/0d612816ec6fc0762f10ff1dffbb0ccdcf3dc60a))
* implemented main loop ([bbaa96c](https://github.com/Kibalchish47/dynamixplore/commit/bbaa96c53fd46715825d36eb1a0bcd3fef721171))
* run transient phase + initialization for main loop ([29788bc](https://github.com/Kibalchish47/dynamixplore/commit/29788bc8bf91222bbcd685c7d14206b2418a5836))
* updated `lib.rs` with new functions ([01d21df](https://github.com/Kibalchish47/dynamixplore/commit/01d21df9e93975c19389716cfabe19b8c6bedcb9))


### Documentation

* `entropy.rs` code annotation ([d942e3e](https://github.com/Kibalchish47/dynamixplore/commit/d942e3e6dafe9406a743b50dcc27aa0a5a57d0fd))
* `lyapunov.rs` code annotation ([3339aba](https://github.com/Kibalchish47/dynamixplore/commit/3339abaf075c54fb482aa8587c8c8734ae0cfc9b))
* README.md improvement ([99f9e51](https://github.com/Kibalchish47/dynamixplore/commit/99f9e5190c22139d421950c5e81d8f96ae75b1a3))

## [0.3.0](https://github.com/Kibalchish47/dynamixplore/compare/dynamixplore-v0.2.0...dynamixplore-v0.3.0) (2025-08-05)


### Features

* (tentatively) reworked the integrators logic ([94c9014](https://github.com/Kibalchish47/dynamixplore/commit/94c90141a8a13eee4df2112e38be53651dc1b32d))
* created simple public functions ([f85e46d](https://github.com/Kibalchish47/dynamixplore/commit/f85e46d68d97de576e54c895afb4bdb10d6e975f))
* defined Python module wrapper in `/lib.rs` ([536dbf7](https://github.com/Kibalchish47/dynamixplore/commit/536dbf76a29d81a9445cc99f817c29839fdf5252))
* defined the package's public-facing API ([7fb9592](https://github.com/Kibalchish47/dynamixplore/commit/7fb95923d34046a8d286ab5aa606afb1ec9c8ffe))
* expanded the integrator architecture ([7dc45b7](https://github.com/Kibalchish47/dynamixplore/commit/7dc45b788df2be3274e9b098088948d649ac5346))
* expanded the public API ([9efbf75](https://github.com/Kibalchish47/dynamixplore/commit/9efbf75c2dcd088b709d872621fb3ae2ca523456))
* implemented `entropy.rs` module ([de330bc](https://github.com/Kibalchish47/dynamixplore/commit/de330bc21441a98e6b9959fdb434291ed4ed2710))
* implemented `stats.rs` module ([ce52697](https://github.com/Kibalchish47/dynamixplore/commit/ce526973f26c029331d4757c2607aa8ba093117c))
* implemented high-level plotting in `visualize.py` ([2aacaa4](https://github.com/Kibalchish47/dynamixplore/commit/2aacaa4759830ed6c5aff239c4d92becbae5b815))
* implemented RK4 and euler's methods properly ([c0590f8](https://github.com/Kibalchish47/dynamixplore/commit/c0590f8335006c8509a899190fe0dca90bdb2e83))
* implemented stateful `Analysis` class for post-simulation processing ([e8a3e64](https://github.com/Kibalchish47/dynamixplore/commit/e8a3e641456e18a62891dc82b66905237b3d08ea))
* implemented the  method belonging to the  class ([fa1a5e2](https://github.com/Kibalchish47/dynamixplore/commit/fa1a5e2867caa5a373987cdce95964045a53da36))
* implemented the integration loop ([72585b0](https://github.com/Kibalchish47/dynamixplore/commit/72585b04ea01ae5c2892103815d58e6d9ea6c176))
* implemented the RK45 Dormand-Prince 5(4) algorithmic integrator ([7625547](https://github.com/Kibalchish47/dynamixplore/commit/7625547a071c3355dd024f73b342c9110c7a3003))
* initial sketch for PyO3 integration in the solve_rk45() integrator ([0eb12af](https://github.com/Kibalchish47/dynamixplore/commit/0eb12af1242003c67d7d4d7990d9ebacd6949a2a))
* modified `lib.rs` to account for new functions ([fa1a2a9](https://github.com/Kibalchish47/dynamixplore/commit/fa1a2a9a868d7879ac8df844e46ff57f60cda67e))
* partially implemented  class (constructor) ([6cea758](https://github.com/Kibalchish47/dynamixplore/commit/6cea758d5f7e4d4c836ea8906dd7505debdcc3d5))
* re-implemented the solver architecture ([ae880bf](https://github.com/Kibalchish47/dynamixplore/commit/ae880bf8cb470b4de0d2db8babebe6c5c5904cf5))
* sketched out the RK4 and RK45 integrators implementation ([c04b1c3](https://github.com/Kibalchish47/dynamixplore/commit/c04b1c33ecac59c9f70672f16c301fb35ca73f17))


### Documentation

* basic code documentation for ([c30e477](https://github.com/Kibalchish47/dynamixplore/commit/c30e4777f255235a653879575e9f3fa68d7bd24f))
* basic README outline ([ac4e938](https://github.com/Kibalchish47/dynamixplore/commit/ac4e938bb1819c2877470c522ccdfa6cdb9074dd))
* finalized  code annotation and documentation (docstrings) ([1b5b2d3](https://github.com/Kibalchish47/dynamixplore/commit/1b5b2d3b90f7c80c7a5d5eb3f224ba153242f1fc))
* typo + removed unnecessary file ([7a90956](https://github.com/Kibalchish47/dynamixplore/commit/7a90956ac8014bb1f6ecb8126987a6c94a89dc00))
* wrote `DynamiXplore`'s summary ([eb5779b](https://github.com/Kibalchish47/dynamixplore/commit/eb5779bd915216ea404a23692f95bdc2ac3ac9b4))

## [0.2.0](https://github.com/Kibalchish47/dynamixplore/compare/dynamixplore-v0.1.0...dynamixplore-v0.2.0) (2025-07-05)


### Features

* (tentatively) reworked the integrators logic ([2e682c6](https://github.com/Kibalchish47/dynamixplore/commit/2e682c673d7f16f8ae3409e36efd4188fb38f29b))
* created simple public functions ([701713d](https://github.com/Kibalchish47/dynamixplore/commit/701713d51e3cc345288cf6c344cd6e2e4641451f))
* defined Python module wrapper ([08b22d6](https://github.com/Kibalchish47/dynamixplore/commit/08b22d63d29dcf542077e6e8d0263d33ef160571))
* expanded integrator architecture ([cba40aa](https://github.com/Kibalchish47/dynamixplore/commit/cba40aadd0f7392c95c3f304c53fae1c973dec7d))
* expanded the public API ([4355720](https://github.com/Kibalchish47/dynamixplore/commit/4355720a31fc382dbbdf13c81557f85b02339a87))
* implemented the integration loop ([0cf13b5](https://github.com/Kibalchish47/dynamixplore/commit/0cf13b5301d95adbb35ed065bf8ed634b11e0d69))
* implementing the RK45 Dormand-Prince 5(4) algorithmic integrator ([663c9e5](https://github.com/Kibalchish47/dynamixplore/commit/663c9e59f7fc9e349c7fe25a4c9d0752f283ddf8))
* initial sketch for PyO3 integration in the solve_rk45() integrator ([7fe2c24](https://github.com/Kibalchish47/dynamixplore/commit/7fe2c242f6e11e4aeee2c323eb3d1ed5a7bdcc97))
* sketch for RK4 and RK45 integrators implementation ([f31fceb](https://github.com/Kibalchish47/dynamixplore/commit/f31fcebd8c9dc9a864bcc37ac29b1ff94db94dc2))


### Documentation

* basic README outline ([33914e2](https://github.com/Kibalchish47/dynamixplore/commit/33914e29e56c2fc1110f8a74facee9518e6fd271))
