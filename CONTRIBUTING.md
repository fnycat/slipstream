
<!-- omit in toc -->
# Contributing to Slipstream

First off, thanks for taking the time to contribute!

All types of contributions are encouraged and valued. See the [Table of Contents](#table-of-contents) for different ways to help and details about how this project handles them. Please make sure to read the relevant section before making your contribution. It will make it a lot easier for us maintainers and smooth out the experience for all involved. The community looks forward to your contributions.

> And if you like the project, but just don't have time to contribute, that's fine. There are other easy ways to support the project and show your appreciation, which we would also be very happy about:
> - Star the project
> - Tweet about it
> - Refer this project in your project's readme
> - Mention the project at local meetups and tell your friends/colleagues

<!-- omit in toc -->
## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [I Have a Question](#i-have-a-question)
  - [I Want To Contribute](#i-want-to-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Enhancements](#suggesting-enhancements)
  - [Your First Code Contribution](#your-first-code-contribution)
  - [Improving The Documentation](#improving-the-documentation)
- [Styleguides](#styleguides)
  - [Commit Messages](#commit-messages)
- [Join The Project Team](#join-the-project-team)


## Code of Conduct

This project and everyone participating in it is governed by the
[Slipstream Code of Conduct](https://github.com/fnycat/slipstream/blob/master/CODE_OF_CONDUCT.md).
By participating, you are expected to uphold this code. Please report unacceptable behavior
to <funnycat@duck.com>.


## I Have a Question

> If you want to ask a question about development, we assume that you have read the available [Documentation](https://fnycat.github.io/slipstream).

Before you ask a question, it is best to search for existing [Issues](https://github.com/fnycat/slipstream/issues) that might help you. In case you have found a suitable issue and still need clarification, you can write your question in this issue. It is also advisable to search the internet for answers first. For issues that are related to
general modding, you can consult the [Retro Rewind](https://discord.gg/retrorewind) Discord server or the [Mario Kart Modding Community](https://discord.gg/mario-kart-modding-community-219114060325584897) on Discord.

If you then still feel the need to ask a question and need clarification, we recommend the following:

- Open an [Issue](https://github.com/fnycat/slipstream/issues/new).
- Provide as much context as you can about what you're running into.
- Provide project and platform versions (Rust compiler version, operating system, etc), depending on what seems relevant.

We will then take care of the issue as soon as possible.

## I Want To Contribute

> ### Legal Notice <!-- omit in toc -->
> When contributing to this project, you must agree that you have authored 100% of the content, that you have the necessary rights to the content and that the content you contribute may be provided under the project licence.
>
> Your code will be licensed under the project's Apache 2.0 license.

### Reporting Bugs

<!-- omit in toc -->
#### Before Submitting a Bug Report

A good bug report shouldn't leave others needing to chase you up for more information. Therefore, we ask you to investigate carefully, collect information and describe the issue in detail in your report. Please complete the following steps in advance to help us fix any potential bug as fast as possible.

- Make sure that you are using the latest version.
- Determine if your bug is really a bug and not an error on your side e.g. using incompatible environment components/versions (Make sure that you have read the [documentation](https://fnycat.github.io/slipstream). If you are looking for support, you might want to check [this section](#i-have-a-question)).
- To see if other users have experienced (and potentially already solved) the same issue you are having, check if there is not already a bug report existing for your bug or error in the [bug tracker](https://github.com/fnycat/slipstream/issues?q=label%3Abug).
- Also make sure to search the internet to see if users outside of the GitHub community have discussed the issue.
- Collect information about the bug:
  - Stack trace (Traceback)
  - OS, Platform and Version (Windows, Linux, macOS, x86, WebAssembly)
  - Version of the interpreter, compiler, SDK, runtime environment, package manager, depending on what seems relevant.
  - Possibly your input and the output
  - Can you reliably reproduce the issue? And can you also reproduce it with older versions?

<!-- omit in toc -->
#### How Do I Submit a Good Bug Report?

> You must never report security related issues, vulnerabilities or bugs including sensitive information to the issue tracker, or elsewhere in public. Instead sensitive bugs must be sent by email to <funnycat@duck.com>.
<!-- You may add a PGP key to allow the messages to be sent encrypted as well. -->

We use GitHub issues to track bugs and errors. If you run into an issue with the project:

- Open an [Issue](https://github.com/fnycat/slipstream/issues/new). (Since we can't be sure at this point whether it is a bug or not, we ask you not to talk about a bug yet and not to label the issue.)
- Explain the behavior you would expect and the actual behavior.
- Please provide as much context as possible and describe the *reproduction steps* that someone else can follow to recreate the issue on their own. This usually includes your code. For good bug reports you should isolate the problem and create a reduced test case.
- Provide the information you collected in the previous section.

Once it's filed:

- The project team will label the issue accordingly.
- A team member will try to reproduce the issue with your provided steps. If there are no reproduction steps or no obvious way to reproduce the issue, the team will ask you for those steps and mark the issue as `needs-repro`. Bugs with the `needs-repro` tag will not be addressed until they are reproduced.
- If the team is able to reproduce the issue, it will be marked `needs-fix`, as well as possibly other tags (such as `Area: ui`), and the issue will be left to be [implemented by someone](#your-first-code-contribution).

<!-- You might want to create an issue template for bugs and errors that can be used as a guide and that defines the structure of the information to be included. If you do so, reference it here in the description. -->


### Suggesting Enhancements

This section guides you through submitting an enhancement suggestion for Slipstream, **including completely new features and minor improvements to existing functionality**. Following these guidelines will help maintainers and the community to understand your suggestion and find related suggestions.

<!-- omit in toc -->
#### Before Submitting an Enhancement

- Make sure that you are using the latest version.
- Read the [documentation](https://fnycat.github.io/slipstream) carefully and find out if the functionality is already covered, maybe by an individual configuration.
- Perform a [search](https://github.com/fnycat/slipstream/issues) to see if the enhancement has already been suggested. If it has, add a comment to the existing issue instead of opening a new one.
- Find out whether your idea fits with the scope and aims of the project. It's up to you to make a strong case to convince the project's developers of the merits of this feature. Keep in mind that we want features that will be useful to the majority of our users and not just a small subset.

<!-- omit in toc -->
#### How Do I Submit a Good Enhancement Suggestion?

Enhancement suggestions are tracked as [GitHub issues](https://github.com/fnycat/slipstream/issues).

- Use a **clear and descriptive title** for the issue to identify the suggestion.
- Provide a **step-by-step description of the suggested enhancement** in as many details as possible.
- **Describe the current behavior** and **explain which behavior you expected to see instead** and why. At this point you can also tell which alternatives do not work for you.
- You may want to **include screenshots or screen recordings** which help you demonstrate the steps or point out the part which the suggestion is related to. You can use Xbox Gamebar on Windows, [LICEcap](https://www.cockos.com/licecap/) to record GIFs on macOS and Windows, and the built-in [screen recorder in GNOME](https://help.gnome.org/users/gnome-help/stable/screen-shot-record.html.en) or [SimpleScreenRecorder](https://github.com/MaartenBaert/ssr) on Linux. <!-- this should only be included if the project has a GUI -->
- **Explain why this enhancement would be useful** to most Slipstream users. You may also want to point out the other projects that solved it better and which could serve as inspiration.

<!-- You might want to create an issue template for enhancement suggestions that can be used as a guide and that defines the structure of the information to be included. If you do so, reference it here in the description. -->

### Your First Code Contribution
To get up and running with the project you need to have a recent version the Rust compiler installed. This project currently does not have a specific MSRV (minimum supported Rust version).
> To compile the editor for web, you will need additional tools. See [compiling for web](#compiling-for-web) for instructions.

Then simply execute `cargo run` in the root directory of the package to compile the project. No additional tools are needed.

#### Compiling for web

Install `wasm-pack` using `cargo install wasm-pack`. This tool will take care of the build process and will also automatically install the required targets for your Rust distribution. 
The project can be compiled by running `wasm-pack build --target web slipstream`. By default, `wasm-pack` will build the release profile, pass `--dev` to build the debug profile.

After compiling, the WASM module will be in the `slipstream/pkg` directory. To open the compile module in your browser, run a local HTTP server (for example using `python -m http.server`) and open `slipstream/index.html`.

### Improving The Documentation
The documentation is automatically built by a GitHub action and deployed to the [GitHub Pages site](https://fnycat.github.io/slipstream).

## Styleguides

### Code style

A GitHub actions runner automatically runs `cargo fmt` on the project using the `rustfmt.toml` file in the root of the repository. If your code has not been formatted, it will be rejected.

### Commit Messages

Ensure your commit messages describe the changes you've made.

## Join The Project Team
If you want to become part of the project team, please email <funnycat@duck.com> or send a message on Discord to `fnycat` on Discord.

<!-- omit in toc -->
## Attribution
This guide is based on the [contributing.md](https://contributing.md/generator)!
