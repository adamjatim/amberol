# Contribution guidelines

Thank you for considering contributing to the Amberol project!

Following these guidelines helps to communicate that you respect the time of
the developers managing and developing this free software project. In return,
they should reciprocate that respect in addressing your issue, assessing
changes, and helping you finalize your pull requests.

There are many ways to contribute, from improving the documentation,
submitting bug reports and feature requests, localizing the user interface, or
writing code which can be incorporated into Amberol itself.

The issue tracker is meant to be used for actionable issues only. Please,
don't use the issue tracker for support questions. Feel free to use the
[GNOME Discourse forum](https://discourse.gnome.org) to ask your questions.

## How to report bugs

Issues should only be reported [on the project page](https://gitlab.gnome.org/Amberol/issues/).

### Bug reports

If you're reporting a bug make sure to list:

 0. which version of Amberol are you using?
 0. which operating system are you using?
 0. how did you install Amberol?
 0. the necessary steps to reproduce the issue
 0. the expected outcome
 0. a description of the behavior; screenshots are also welcome

If the issue includes a crash, you should also include:

 0. the eventual warnings printed on the terminal
 0. a backtrace, obtained with tools such as GDB or LLDB

It is fine to include screenshots of screen recordings to demonstrate
an issue that is best to understand visually, but please don't just
attach screen recordings without further details into issues. It is
essential that the problem is described in enough detail to reproduce
it without watching a video.

For small issues, such as:

 - spelling/grammar fixes in the documentation
 - typo correction
 - comment clean ups
 - changes to metadata files (CI, `.gitignore`)
 - build system changes
 - source tree clean ups and reorganizations

You should directly open a merge request instead of filing a new issue.

### Security issues

If you have a security issue, please mark it as confidential in the issue
tracker, to ensure that only the maintainers can see it.

### Features and enhancements

Feature discussion can be open ended and require high bandwidth channels; if
you are proposing a new feature on the issue tracker, make sure to make
an actionable proposal, and list:

 0. what you're trying to achieve
 0. prior art, in other applications
 0. design and theming changes

When in doubt, you should open an issue to discuss your changes and ask
questions before opening your code editor and hacking away; this way you'll get
feedback from the project maintainers, if they have any, and you will avoid
spending unnecessary effort.

## Your first contribution

### Prerequisites

If you want to contribute to the Amberol project, you will need to have the
development tools appropriate for your operating system, including:

 - Python 3.x
 - Meson
 - Ninja
 - the Rust compiler
 - Cargo

### Dependencies

You will also need the various dependencies needed to build Amberol from
source. You will find the compile time dependencies in the
[`Cargo.toml`](./Cargo.toml) file, while the run time dependencies are listed
in the [`meson.build`](./meson.build) file.

You are strongly encouraged to use GNOME Builder to build and run Amberol,
as it knows how to download and build all the dependencies necessary.

### Getting started

You should start by forking the Amberol repository from the GitLab web UI;
then you can select *Clone Repository* from GNOME Builder and use your
fork's URL as the repository URL.

GNOME Builder will find all the dependencies and download them for you.

----

If you want to use another development environment, you will need to clone
the repository manually:

```sh
$ git clone git@gitlab.gnome.org:yourusername/amberol.git
$ cd amberol
```

To compile the Git version of GTK on your system, you will need to
configure your build using Meson:

```sh
$ meson setup _builddir .
$ meson compile -C _builddir
```

Meson will search for all the required dependencies during the setup
step, and will run Cargo in the compile step.

You can run Amberol uninstalled by using the Meson devenv command:

```sh
$ meson devenv -C _builddir
$ ./src/amberol
$ exit
```

----

You can now switch to a new branch to work on Amberol:

```sh
$ git switch -C your-branch
```

Once you've finished working on the bug fix or feature, push the branch
to your Git repository and open a new merge request, to let the Amberol
maintainers review your contribution.

Remember that the Amberol is maintained by volunteers, so it might take a
little while to get reviews or feedback. Don't be discouraged, and feel
free to join the `#amberol:gnome.org` channel on Matrix for any issue you
may find.

### Coding style

Amberol uses the standard Rust coding style. You can use:

    cargo +nightly fmt --all

To ensure that your contribution is following the expected format.

Amberol has an additional set of checks available in the
[`checks.sh`](./build-aux/checks.sh) tool.

### Commit messages

The expected format for git commit messages is as follows:

```plain
Short explanation of the commit

Longer explanation explaining exactly what's changed, whether any
external or private interfaces changed, what bugs were fixed (with bug
tracker reference if applicable) and so forth. Be concise but not too
brief.

Closes #1234
```

 - Always add a brief description of the commit to the _first_ line of
 the commit and terminate by two newlines (it will work without the
 second newline, but that is not nice for the interfaces).

 - First line (the brief description) must only be one sentence and
 should start with a capital letter unless it starts with a lowercase
 symbol or identifier. Don't use a trailing period either. Don't exceed
 72 characters.

 - The main description (the body) is normal prose and should use normal
 punctuation and capital letters where appropriate. Consider the commit
 message as an email sent to the developers (or yourself, six months
 down the line) detailing **why** you changed something. There's no need
 to specify the **how**: the changes can be inlined.

 - When committing code on behalf of others use the `--author` option, e.g.
 `git commit -a --author "Joe Coder <joe@coder.org>"` and `--signoff`.

 - If your commit is addressing an issue, use the
 [GitLab syntax](https://docs.gitlab.com/ce/user/project/issues/automatic_issue_closing.html)
 to automatically close the issue when merging the commit with the upstream
 repository:

```plain
Closes #1234
Fixes #1234
Closes: https://gitlab.gnome.org/GNOME/gtk/issues/1234
```

 - If you have a merge request with multiple commits and none of them
 completely fixes an issue, you should add a reference to the issue in
 the commit message, e.g. `Bug: #1234`, and use the automatic issue
 closing syntax in the description of the merge request.
