# Contributing To Trow

Firstly, big thanks for considering contributing to the project. We really hope to make this into a
community project and to do that we need your help!

## Code of Conduct

We are serious about making this a welcoming, happy project. We will not tolerate discrimination,
aggressive or insulting behaviour.

To this end, the project and everyone participating in it is bound by the [Code of
Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report
unacceptable behaviour to any of the project admins or adrian.mouat@container-solutions.com.

## Bugs

If you find a bug, please [open an issue](https://github.com/trow-registry/trow/issues)! Do try
to include all the details needed to recreate your problem. This is likely to include:

 - The version of Trow being used
 - How Trow was installed
 - The exact platform and version of the platform that Trow is running on
 - The steps taken to cause the bug

## Building Features and Documentation

If you're looking for something to work on, take look at the issue tracker, in particular any items
labelled [good first issue](https://github.com/trow-registry/trow/labels/good%20first%20issue).
Please leave a comment on the issue to mention that you have started work, in order to avoid
multiple people working on the same issue.

If you have an idea for a feature - whether or not you have time to work on it - please also open an
issue describing your feature and label it "enhancement". We can then discuss it as a community and
see what can be done. Please be aware that some features may not align with the project goals and
might therefore be closed. In particular, please don't start work on a new feature without
discussing it first to avoid wasting effort. We do commit to listening to all proposals and will do
our best to work something out!

Once you've got the go ahead to work on a feature, you can start work. See
[DEVELOPING.md](DEVELOPING.md) for advice on building and testing Trow. Feel free to communicate
with team via updates on the issue tracker and ask for feedback, pointers etc. Once you're happy
with your code, go ahead and open a Pull Request.

## Pull Request Process

Most of the code is written in Rust and uses the standard style guidelines - please run your code
through `rustfmt` prior to opening a PR.

On opening a PR, a GitHub action will execute the test suite against the new code. All code is
required to pass the tests, and new code must be accompanied by new tests.

All PRs have to be reviewed and signed off by another developer before being merged to the main
branch. This review will likely ask for some changes to the code - please don't be alarmed or upset
at this; it is expected that all PRs will need tweaks and a normal part of the process.

Be aware that all Trow code is released under the [Apache 2.0 licence](LICENSE).

## Thanks

Thanks to [Container Solutions](https://www.container-solutions.com/) for creating the project and to [Extrality](https://www.extrality.ai/) for further developing the project.

Contribution guidelines take inspiration from
[Atom](https://github.com/atom/atom/blob/master/CONTRIBUTING.md), [PurpleBooth's
advice](https://gist.github.com/PurpleBooth/b24679402957c63ec426) and the [Contributor
Covenant](https://www.contributor-covenant.org/).
