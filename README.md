# Pasts

#### [Changelog][3] | [Source][4] | [Getting Started][5]

[![tests](https://github.com/AldaronLau/pasts/workflows/tests/badge.svg)][2]
[![docs](https://docs.rs/pasts/badge.svg)][0]
[![crates.io](https://img.shields.io/crates/v/pasts.svg)][1]

Minimal and simpler alternative to the futures crate.

Check out the [documentation][0] for examples.

# Goals
 - No required std (on no\_std, a single allocation is required)
 - No slow compiling proc macros (fast compile times)
 - No dependencies
 - No cost (True zero-cost abstractions!)
 - No pain (API super easy to learn & use!)
 - No unsafe code left for *you* to write for working with `Future`s (ability to
   `#[forbid(unsafe_code)]`)
 - No platform-specific API differences (code works everywhere!).
 - No worrying about pinning and fusing.

### Supported Platforms
Pasts targets all platforms that can run Rust.  The executor works
on at least the following platforms (may work on others):
 - All platforms that support threading (includes all tier 1 and some tier 2, 3)
 - Web Assembly In Browser (Tier 2)

## License
Licensed under any of
 - Apache License, Version 2.0, ([LICENSE_APACHE_2_0.txt][7]
   or [https://www.apache.org/licenses/LICENSE-2.0][8])
 - MIT License, ([LICENSE_MIT.txt][9] or [https://mit-license.org/][10])
 - Boost Software License, Version 1.0, ([LICENSE_BOOST_1_0.txt][11]
   or [https://www.boost.org/LICENSE_1_0.txt][12])

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as described above, without any additional terms or conditions.

## Help
If you want help using or contributing to this library, feel free to send me an
email at [aldaronlau@gmail.com][13].

[0]: https://docs.rs/pasts
[1]: https://crates.io/crates/pasts
[2]: https://github.com/AldaronLau/pasts/actions?query=workflow%3Atests
[3]: https://github.com/AldaronLau/pasts/blob/main/CHANGELOG.md
[4]: https://github.com/AldaronLau/pasts
[5]: https://docs.rs/pasts#getting-started
[6]: https://aldaronlau.com/
[7]: https://github.com/AldaronLau/pasts/blob/main/LICENSE_APACHE_2_0.txt
[8]: https://www.apache.org/licenses/LICENSE-2.0
[9]: https://github.com/AldaronLau/pasts/blob/main/LICENSE_MIT.txt
[10]: https://mit-license.org/
[11]: https://github.com/AldaronLau/pasts/blob/main/LICENSE_BOOST_1_0.txt
[12]: https://www.boost.org/LICENSE_1_0.txt
[13]: mailto:aldaronlau@gmail.com
