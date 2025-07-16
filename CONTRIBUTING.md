# Contributing to Kable

Thank you for your interest in contributing to Kable! We welcome community contributions that help improve the launcher for everyone. Please read this guide carefully before contributing.

## 📋 Table of Contents

- [AI-Assisted Development](#ai-assisted-development)
- [Code of Conduct](#code-of-conduct)
- [How to Contribute](#how-to-contribute)
- [Development Setup](#development-setup)
- [Contribution Types](#contribution-types)
- [Submission Guidelines](#submission-guidelines)
- [Code Style](#code-style)
- [Testing](#testing)
- [License Agreement](#license-agreement)

## � AI-Assisted Development

### Our Stance on AI
We **welcome and encourage** the use of AI tools for development! AI can be incredibly helpful for:
- **Code generation** and boilerplate creation
- **Bug fixing** and problem-solving
- **Documentation** writing and improvement
- **Test creation** and edge case discovery
- **Code review** and optimization suggestions

### Requirements for AI-Generated Code
However, we have **strict requirements** for AI-assisted contributions:

#### ✅ You MUST:
- **Read and understand** every line of AI-generated code before submitting
- **Test thoroughly** - AI code often needs adjustments to work correctly
- **Verify correctness** - Ensure the code actually solves the intended problem
- **Check for security issues** - AI can introduce vulnerabilities
- **Ensure code quality** - Make sure it follows our style guidelines
- **Add proper comments** - Explain complex logic, especially if AI-generated
- **Take responsibility** - You are accountable for all code in your PR, regardless of how it was created

#### ❌ You MUST NOT:
- **Copy-paste blindly** without understanding what the code does
- **Submit untested code** - "it compiles" is not enough
- **Ignore code style** - AI often doesn't follow project conventions
- **Skip code review** - Always review AI suggestions critically
- **Submit broken code** - Test everything before creating a PR

### Best Practices for AI Usage

#### Code Generation
```typescript
// ❌ BAD: Blindly copied from AI
const result = await someComplexFunction(data);

// ✅ GOOD: AI-generated but reviewed and understood
const result = await someComplexFunction(data);
// This function processes the installation data and returns a validated config
// We need to handle the potential errors from invalid mod loader types
```

#### Testing AI Code
- **Run the code** in your local environment
- **Test edge cases** that AI might have missed
- **Verify error handling** works as expected
- **Check performance** for large datasets
- **Validate security** implications

#### Documentation
- **Explain complex AI-generated logic** in comments
- **Document any AI-specific patterns** or optimizations used
- **Note if code behavior** differs from typical patterns

### Why These Requirements?

#### Code Quality
- **AI isn't perfect** - it can generate buggy, inefficient, or insecure code
- **Context matters** - AI might not understand the full project context
- **Consistency** - AI often doesn't follow project-specific patterns
- **Maintainability** - Code you don't understand is hard to maintain

#### Review Efficiency
- **Faster PR reviews** when code is well-understood by the author
- **Better discussions** when contributors can explain their implementation choices
- **Reduced back-and-forth** on obviously broken or inappropriate code
- **Focus on architecture** rather than basic correctness issues

#### Learning and Growth
- **Skill development** - Understanding code makes you a better developer
- **Project knowledge** - Helps you contribute more effectively over time
- **Problem-solving** - Learn from AI suggestions rather than just copying them

### AI Tools We Recommend
- **GitHub Copilot** - Great for in-editor suggestions
- **ChatGPT/Claude** - Excellent for explaining complex problems
- **Cursor/Windsurf** - AI-powered IDEs with good context awareness
- **Codeium** - Free alternative to Copilot
- **Tabnine** - Privacy-focused AI completion

### Declaring AI Usage
While not required, we appreciate when contributors mention AI usage in PRs:
```markdown
## AI Usage
This PR used AI assistance for:
- Initial implementation of the mod loader detection algorithm
- Test case generation for edge cases
- Documentation improvements

All code has been reviewed, tested, and understood before submission.
```

### The Bottom Line
**AI is a powerful tool, but you are the developer.** Use AI to enhance your productivity, but always take responsibility for the final result. We'd rather receive well-understood, AI-assisted code than poorly understood "pure" human code.

Remember: **The goal is shipping quality software, not proving human superiority over AI.**

## �🤝 Code of Conduct

### Our Standards
- **Be respectful** and constructive in all interactions
- **Be inclusive** and welcoming to contributors of all backgrounds
- **Focus on what is best** for the community and the project
- **Show empathy** towards other community members

### Unacceptable Behavior
- Harassment, discrimination, or inappropriate conduct
- Publishing others' private information without permission
- Trolling, insulting/derogatory comments, and personal attacks
- Spamming or off-topic discussions

## 🚀 How to Contribute

### 1. Before You Start
- Check existing [issues](../../issues) to avoid duplicates
- Review our [project roadmap](README.md#future-goals) to understand priorities
- Read this contributing guide thoroughly
- Ensure you understand our [license terms](LICENSE.md)

### 2. Ways to Contribute
- **Bug Reports**: Help us identify and fix issues
- **Feature Requests**: Suggest new functionality
- **Code Contributions**: Submit bug fixes and improvements
- **Documentation**: Improve guides and documentation
- **Testing**: Help test new features and releases

## 🛠️ Development Setup

### Prerequisites
- **Node.js** 18.x or higher
- **npm** or **yarn** package manager
- **Rust** (latest stable version)
- **Git** for version control

### Platform-Specific Requirements

#### Windows
- **Microsoft C++ Build Tools** or Visual Studio with C++ support
- **Windows 10 SDK**

#### macOS
- **Xcode Command Line Tools**
```bash
xcode-select --install
```

#### Linux
- **build-essential** package
- **libwebkit2gtk-4.0-dev** and other WebKit dependencies
```bash
sudo apt update
sudo apt install build-essential libwebkit2gtk-4.0-dev libgtk-3-dev libappindicator3-dev
```

### Setup Steps

1. **Fork the repository**
   ```bash
   # Fork on GitHub, then clone your fork
   git clone https://github.com/YOUR_USERNAME/kable.git
   cd kable
   ```

2. **Add upstream remote**
   ```bash
   git remote add upstream https://github.com/LuckyLuuk12/kable.git
   ```

3. **Install dependencies**
   ```bash
   npm install
   ```

4. **Start development server**
   ```bash
   npm run dev
   ```

5. **Verify setup**
   - The application should open automatically
   - Check that hot reload works by making a small change

## 🎯 Contribution Types

### 🐛 Bug Reports
**Before submitting:**
- Search existing issues for duplicates
- Test with the latest version
- Provide minimal reproduction steps

**Include in your report:**
- **Operating System** and version
- **Application version**
- **Steps to reproduce** the issue
- **Expected behavior** vs **actual behavior**
- **Screenshots** or **error logs** if applicable
- **System specifications** (Java version, available memory, etc.)

### ✨ Feature Requests
**Before submitting:**
- Check if the feature exists in development roadmap
- Discuss complex features in [GitHub Discussions](../../discussions) first

**Include in your request:**
- **Clear description** of the feature
- **Use case** and **motivation**
- **Potential implementation** ideas (if applicable)
- **Mockups** or **wireframes** (if UI-related)

### 💻 Code Contributions

#### Types of Code Contributions
- **Bug fixes** - High priority, especially for crashes or data loss
- **Performance improvements** - Optimizations and efficiency gains
- **UI/UX improvements** - Better user experience and accessibility
- **New features** - Must align with project roadmap and be discussed first
- **Documentation** - Code comments, README updates, guides

#### Priority Levels
1. **Critical**: Security issues, crashes, data loss
2. **High**: Major bugs, performance issues, accessibility
3. **Medium**: Minor bugs, usability improvements
4. **Low**: New features, code refactoring, nice-to-have improvements

## 📝 Submission Guidelines

### Pull Request Process

1. **Create a new branch**
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/issue-description
   ```

2. **Make your changes**
   - Follow our [code style guidelines](#code-style)
   - Add tests for new functionality
   - Update documentation as needed

3. **Commit your changes**
   ```bash
   # Use conventional commit format
   git commit -m "feat: add new mod loader detection"
   git commit -m "fix: resolve authentication token refresh issue"
   git commit -m "docs: update installation instructions"
   ```

4. **Push and create PR**
   ```bash
   git push origin your-branch-name
   ```
   - Create a pull request through GitHub
   - Fill out the PR template completely
   - Link related issues using keywords (e.g., "Fixes #123")

### PR Requirements
- **Descriptive title** following conventional commit format
- **Complete description** of changes and motivation
- **Screenshots** for UI changes
- **Testing instructions** for reviewers
- **Documentation updates** if applicable
- **No merge conflicts** with main branch

### Review Process
1. **Automated checks** must pass (linting, tests, build)
2. **Code review** by maintainers
3. **Testing** on multiple platforms if needed
4. **Approval** and merge by project maintainer

## 🎨 Code Style

### General Principles
- **Clarity over cleverness** - Write readable, maintainable code
- **Consistency** - Follow existing patterns in the codebase
- **Documentation** - Comment complex logic and public APIs
- **Performance** - Consider performance implications of changes

### TypeScript/JavaScript
```typescript
// Use clear, descriptive variable names
const minecraftInstallations = await getInstallations();

// Prefer async/await over promises
async function loadUserProfile(): Promise<UserProfile> {
    try {
        const profile = await fetchProfile();
        return profile;
    } catch (error) {
        Logger.error('Failed to load profile:', error);
        throw error;
    }
}

// Use proper typing
interface InstallationConfig {
    name: string;
    version: string;
    modLoader: ModLoaderType;
    memory?: number;
}
```

### Rust
```rust
// Use descriptive function names
pub async fn load_minecraft_installations() -> Result<Vec<Installation>, AppError> {
    // Implementation
}

// Proper error handling
fn parse_version_manifest(content: &str) -> Result<VersionManifest, AppError> {
    serde_json::from_str(content)
        .map_err(|e| AppError::Json(format!("Failed to parse manifest: {}", e)))
}

// Use appropriate logging levels
Logger::info_global("Loading installations...", None);
Logger::debug_global(&format!("Found {} versions", versions.len()), None);
```

### Svelte Components
```svelte
<script lang="ts">
    // Use TypeScript for component props
    interface Props {
        installation: Installation;
        onLaunch?: (id: string) => void;
    }
    
    let { installation, onLaunch }: Props = $props();
    
    // Use descriptive event handlers
    function handleLaunchClick() {
        onLaunch?.(installation.id);
    }
</script>

<!-- Use semantic HTML and proper accessibility -->
<button 
    class="launch-button"
    onclick={handleLaunchClick}
    aria-label="Launch {installation.name}"
>
    Launch
</button>

<style>
    .launch-button {
        /* Use CSS custom properties for theming */
        background: var(--primary-color);
        color: var(--primary-text);
    }
</style>
```

### Commit Message Format
Use [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(auth): add Microsoft account token refresh
fix(launcher): resolve memory allocation for large modpacks
docs(readme): update installation instructions
refactor(ui): simplify installation card component
```

## 🧪 Testing

### Running Tests
```bash
# Frontend tests
npm run test

# Rust tests
cargo test

# Integration tests
npm run test:integration
```

### Testing Guidelines
- **Write tests** for new functionality
- **Update tests** when modifying existing code
- **Test edge cases** and error conditions
- **Verify cross-platform** compatibility when possible

### Manual Testing
- Test on your target platform
- Verify UI changes in different screen sizes
- Test with different Minecraft versions and mod loaders
- Check performance with large installations

## 📜 License Agreement

### Contributor Rights
By contributing to this project, you agree that:

1. **Your contributions are original** work created by you
2. **You have the right** to submit the work under this project's license
3. **You grant Luuk Kablan** perpetual, irrevocable rights to use, modify, and distribute your contribution
4. **Your contribution** will be licensed under the same proprietary terms as the project

### Attribution
- Contributors will be credited in project acknowledgments
- Significant contributors may be listed in the README
- All contributions remain subject to the project's proprietary license

## ❓ Questions and Support

### Getting Help
- **Documentation**: Check existing docs and README
- **Search Issues**: Look for similar questions or problems
- **GitHub Discussions**: For general questions and feature discussions
- **GitHub Issues**: For specific bugs or technical problems

### Response Times
- **Bug reports**: 1-3 days for initial response
- **Feature requests**: 1-7 days for initial review
- **Pull requests**: 3-7 days for review (depending on complexity)

## 🏆 Recognition

### Contributors
We appreciate all contributions and will recognize contributors through:
- **GitHub Contributors** section
- **Release notes** for significant contributions
- **Special mentions** for outstanding contributions

### Becoming a Maintainer
Regular, high-quality contributors may be invited to become project maintainers with additional privileges and responsibilities.

---

**Thank you for contributing to Kable! Together, we can build the best Minecraft launcher experience possible.**
