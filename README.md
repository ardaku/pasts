# Pasts

#### Minimal and simpler alternative to the futures crate.

[![tests](https://github.com/AldaronLau/pasts/workflows/tests/badge.svg)][2]
[![docs](https://docs.rs/pasts/badge.svg)][0]
[![crates.io](https://img.shields.io/crates/v/pasts.svg)][1]

[About][4] | [Source][5] | [Changelog][3] | [Tutorial][6]

# About
 - No required std (on no\_std, a single allocation is required)
 - No slow compiling proc macros (fast compile times)
 - No dependencies
 - No cost (True zero-cost abstractions!)
 - No pain (API super easy to learn & use!)
 - No unsafe code left for *you* to write for working with `Future`s (ability to
   `#[forbid(unsafe_code)]`)
 - No platform-specific API differences (code works everywhere!).
 - No worrying about pinning and fusing.

Check out the [documentation][0] for examples.

### Supported Platforms
Pasts targets all platforms that can run Rust.  The executor works
on at least the following platforms (may work on others):
 - All platforms that support threading (includes all tier 1 and some tier 2, 3)
 - Web Assembly In Browser (Tier 2)

## License
Licensed under either of
 - Apache License, Version 2.0 ([LICENSE_APACHE_2_0.txt][7]
   or [https://www.apache.org/licenses/LICENSE-2.0][8])
 - Boost License, Version 1.0 ([LICENSE_BOOST_1_0.txt][9]
   or [https://www.boost.org/LICENSE_1_0.txt][10])

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

Anyone is more than welcome to contribute!  Don't be shy about getting involved,
whether with a question, idea, bug report, bug fix, feature request, feature
implementation, or other enhancement.  Other projects have strict contributing
guidelines, but this project accepts any and all formats for pull requests and
issues.  For ongoing code contributions, if you wish to ensure your code is
used, open a draft PR so that I know not to write the same code.  If a feature
needs to be bumped in importance, I may merge an unfinished draft PR into it's
own branch and finish it (after a week's deadline for the person who openned
it).  Contributors will always be notified in this situation, and given a choice
to merge early.

All pull request contributors will have their username added in the contributors
section of the release notes of the next version after the merge, with a message
thanking them.  I always make time to fix bugs, so usually a patched version of
the library will be out a few days after a report.  Features requests will not
complete as fast.  If you have any questions, design critques, or want me to
find you something to work on based on your skill level, you can email me at
[jeronlau@plopgrizzly.com](mailto:jeronlau@plopgrizzly.com).  Otherwise,
[here's a link to the issues on GitHub](https://github.com/libcala/wavy/issues),
and, as always, make sure to read and follow the
[Code of Conduct](https://github.com/libcala/wavy/blob/main/CODE_OF_CONDUCT.md).

[0]: https://docs.rs/pasts
[1]: https://crates.io/crates/pasts
[2]: https://github.com/AldaronLau/pasts/actions?query=workflow%3Atests
[3]: https://github.com/AldaronLau/pasts/blob/master/CHANGELOG.md
[4]: https://github.com/AldaronLau/pasts/blob/master/README.md
[5]: https://github.com/AldaronLau/pasts
[6]: https://aldaronlau.com/
[7]: https://github.com/AldaronLau/pasts/blob/master/LICENSE-APACHE
[8]: https://www.apache.org/licenses/LICENSE-2.0
[9]: https://github.com/libcala/wavy/blob/main/LICENSE_BOOST_1_0.txt
[10]: https://www.boost.org/LICENSE_1_0.txt
