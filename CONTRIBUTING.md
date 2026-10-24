# Contributing to Kable

Contributions to Kable are welcome, including bug fixes, improvements, and documentation. Please read this guide before submitting changes.

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) and npm
- [Rust](https://www.rust-lang.org/tools/install) and Cargo
- [Git](https://git-scm.com/)
- Platform-specific [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

### Development Setup

1. Fork the repository on GitHub.
2. Clone your fork and enter the project directory:

   ```bash
   git clone https://github.com/luckyluuk12/kable.git
   cd kable
   ```

3. Install dependencies:

   ```bash
   npm install
   ```

4. Start the development application:

   ```bash
   npm run dev
   ```

To build the application, run:

```bash
npm run build
npm run tauri build
```

## Contributing Changes

### Before Starting

- Check [existing issues](../../issues) and [discussions](../../discussions) for related work.
- For substantial changes or new features, open a discussion or issue before investing time in implementation.
- Keep changes focused and consistent with the existing architecture.

### Pull Requests

1. Create a branch for your changes:

   ```bash
   git checkout -b <yourname>/<feat|fix|chore>/short-descriptive-name
   ```

2. Implement and test your changes.
3. Update relevant documentation if necessary.
4. Commit your changes using a descriptive message.
5. Push your branch and open a pull request against the appropriate branch.

Include the following in your pull request:

- A clear description of the change and its motivation.
- Relevant issue references, if applicable.
- Tests performed and any known limitations.
- Screenshots or recordings for user-facing changes, where useful.

Keep pull requests focused. Large or unrelated changes are harder to review.

### Bug Reports and Feature Requests

Use [GitHub Issues](../../issues) for bug reports and [GitHub Discussions](../../discussions) for questions and feature proposals.

For bug reports, include reproduction steps, expected and actual behavior, your operating system, and the Kable version. Attach relevant logs or screenshots when available, but remove authentication tokens and other sensitive information first.

## Code Guidelines

- Follow the conventions and architectural patterns already used in the relevant part of the codebase.
- Prefer clear, maintainable implementations over unnecessary abstraction.
- Use TypeScript types and Rust's type system appropriately.
- Handle errors explicitly, particularly around filesystem operations, external services, and application initialization.
- Avoid unrelated refactoring and unnecessary dependencies.
- Update or add tests when appropriate.
- Review your changes before submitting them, including any code generated with AI assistance.

AI-assisted development is permitted. Contributors remain responsible for the correctness, security, maintainability, and licensing of their submissions. Generated code should be reviewed and tested like any other code.

## Testing

Test the affected functionality before opening a pull request.

Depending on the changes, this may include:

- Running the relevant frontend checks and tests.
- Running Rust tests with `cargo test` from the Rust workspace or crate directory.
- Building the application.
- Manually testing affected launcher functionality.

Use the test scripts defined in `package.json` rather than assuming every type of test is available. Include the checks you ran in your pull request description.

## Releases

Kable uses GitHub Actions to automate nightly and release builds.

- **`main`** is used for active development and nightly builds.
- **`release`** is used for stable release builds.

Check the relevant [GitHub Actions workflows](../../actions) before changing release-related files or workflows.

Commit messages may use [Conventional Commits](https://www.conventionalcommits.org/), for example:

- `feat: add profile export`
- `fix: handle missing Minecraft directory`
- `docs: update setup instructions`

If a workflow supports specific commit-message skip markers, follow the conditions defined in that workflow.

## License and Project Policies

Kable is distributed under a proprietary license. Before contributing, review [LICENSE.md](LICENSE.md) and ensure that you have the right to submit your contribution.

Contributions may be subject to additional terms concerning ownership, licensing, or permission to use submitted code. Clarify these terms with the maintainer before contributing if they are not clear from the project's license or contribution policy.

Please also follow the project's [Code of Conduct](CODE_OF_CONDUCT.md).

## Questions

For questions about contributing, preferably use [The Discord Server](https://kablan.nl/discord) or [GitHub Discussions](../../discussions). For bugs and actionable feature requests, use [GitHub Issues](../../issues).

---

Thank you for helping improve Kable.
