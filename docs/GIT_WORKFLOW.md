# Git Workflow Guide

This document describes the Git workflow for contributing to TALON. Following these guidelines ensures code quality, maintainability, and smooth collaboration.

## Table of Contents

- [Branch Strategy](#branch-strategy)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Workflow](#pull-request-workflow)
- [Code Review Process](#code-review-process)
- [Release Process](#release-process)
- [Common Scenarios](#common-scenarios)
- [Troubleshooting](#troubleshooting)

---

## Branch Strategy

### Main Branches

**`main`**
- Production-ready code
- Always stable and deployable
- Protected branch (requires PR approval)
- Tagged with version numbers (e.g., `v0.1.0`)

**`develop`**
- Integration branch for features
- Pre-release testing happens here
- Merged to `main` during releases
- Protected branch (requires PR approval)

### Feature Branches

**Naming Convention**: `feature/<descriptive-name>`

```bash
# Examples
feature/rop-auto-solver
feature/kernel-exploit-templates
feature/libc-database-integration
```

**Best Practices**:
- Create from `develop` branch
- Keep focused on a single feature
- Regularly sync with `develop` to avoid conflicts
- Delete after merging

```bash
# Create feature branch
git checkout develop
git pull origin develop
git checkout -b feature/my-feature

# Keep feature branch updated
git fetch origin
git rebase origin/develop
```

### Bug Fix Branches

**Naming Convention**: `fix/<issue-number>-<description>`

```bash
# Examples
fix/123-parser-panic
fix/456-memory-leak
fix/789-elf-parsing-error
```

**Best Practices**:
- Reference issue number in branch name
- Create from `develop` (or `main` for hotfixes)
- Include regression test

```bash
# Create bug fix branch
git checkout develop
git pull origin develop
git checkout -b fix/123-parser-panic
```

### Hotfix Branches

**Naming Convention**: `hotfix/<version>-<description>`

```bash
# Examples
hotfix/0.1.1-security-patch
hotfix/0.2.3-critical-bug
```

**Best Practices**:
- Create from `main` for critical production issues
- Merge to both `main` and `develop`
- Bump patch version

```bash
# Create hotfix branch
git checkout main
git pull origin main
git checkout -b hotfix/0.1.1-security-patch

# After fixing, merge to both branches
git checkout main
git merge --no-ff hotfix/0.1.1-security-patch
git tag -a v0.1.1 -m "Security patch"
git push origin main --tags

git checkout develop
git merge --no-ff hotfix/0.1.1-security-patch
git push origin develop
```

### Other Branch Types

**`docs/<topic>`** - Documentation updates
```bash
git checkout -b docs/api-reference
```

**`test/<feature>`** - Test-only changes
```bash
git checkout -b test/heap-exploitation
```

**`perf/<optimization>`** - Performance improvements
```bash
git checkout -b perf/rop-gadget-search
```

**`refactor/<component>`** - Code restructuring
```bash
git checkout -b refactor/interpreter-core
```

---

## Commit Guidelines

### Commit Message Format

We follow [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Commit Types

| Type | Description | Example |
|------|-------------|---------|
| `feat` | New feature | `feat(rop): add automatic ROP chain solver` |
| `fix` | Bug fix | `fix(parser): handle unterminated strings` |
| `docs` | Documentation | `docs(readme): add installation instructions` |
| `test` | Adding/updating tests | `test(heap): add tcache exploitation tests` |
| `perf` | Performance improvement | `perf(elf): optimize symbol table parsing` |
| `refactor` | Code restructuring | `refactor(interpreter): simplify variable lookup` |
| `style` | Code formatting | `style: run cargo fmt` |
| `chore` | Maintenance tasks | `chore: update dependencies` |
| `ci` | CI/CD changes | `ci: add fuzzing workflow` |
| `build` | Build system changes | `build: update Cargo.toml dependencies` |
| `revert` | Revert previous commit | `revert: feat(rop): add automatic solver` |

### Commit Message Examples

**Good Commits**:

```
feat(rop): add gadget quality scoring

Implement a scoring system for ROP gadgets based on:
- Gadget length (shorter is better)
- Number of side effects
- Presence of bad bytes

Closes #123
```

```
fix(parser): prevent panic on malformed hex literals

Previously, parsing "0xZZZZ" would panic. Now returns
a proper ParseError with line/column information.

Fixes #456
```

```
test(heap): add comprehensive tcache tests

- Test tcache poisoning scenarios
- Test double-free detection
- Test size class edge cases

Coverage: +15%
```

**Bad Commits** (avoid these):

```
# Too vague
fix: bug fix

# Not descriptive
update code

# Multiple unrelated changes
feat: add ROP solver, fix parser, update docs
```

### Commit Best Practices

**Atomic Commits**:
- One logical change per commit
- Each commit should build successfully
- Easy to review and revert if needed

```bash
# Good - separate commits for logical changes
git commit -m "feat(rop): add gadget finder"
git commit -m "test(rop): add gadget finder tests"
git commit -m "docs(rop): document gadget finder API"

# Bad - too many unrelated changes
git commit -m "add rop gadget finder, fix parser, update readme"
```

**Commit Frequency**:
- Commit early and often during development
- Squash/rebase before creating PR
- Keep meaningful history

```bash
# During development - commit frequently
git commit -m "wip: initial gadget finder structure"
git commit -m "wip: add capstone integration"
git commit -m "wip: implement quality scoring"

# Before PR - squash work-in-progress commits
git rebase -i develop
# Mark commits as 'squash' to combine them
```

**Commit Body Guidelines**:
- Explain *why*, not *what* (code shows what)
- Reference issues/PRs
- Include breaking changes

```
feat(interpreter): add Shadow Registry for state persistence

The Shadow Registry allows exploit scripts to maintain state
across multiple stages, critical for multi-stage exploitation
where ASLR bases must be preserved between leaks.

BREAKING CHANGE: shadow.store() now requires a type parameter

Closes #789
Related to #456, #567
```

---

## Pull Request Workflow

### Creating a Pull Request

**1. Ensure Code Quality**:

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Run tests
cargo test --all-features

# Run pre-commit checks
./scripts/pre-commit.sh  # Linux/macOS
.\scripts\pre-commit.ps1  # Windows
```

**2. Update Your Branch**:

```bash
# Rebase on latest develop
git fetch origin
git rebase origin/develop

# Resolve conflicts if any
git add .
git rebase --continue
```

**3. Push to Your Fork**:

```bash
# Force push after rebase
git push origin feature/my-feature --force-with-lease
```

**4. Create PR on GitHub**:
- Go to https://github.com/ridpath/talon
- Click "New Pull Request"
- Select your branch
- Fill out the PR template
- Link related issues

### PR Template Checklist

When you create a PR, ensure you've completed:

- [ ] Code follows style guidelines (`cargo fmt`, `cargo clippy`)
- [ ] Added tests for new functionality
- [ ] All tests pass (`cargo test --all-features`)
- [ ] Updated documentation (README, docs/, code comments)
- [ ] Added/updated examples if applicable
- [ ] Followed conventional commit messages
- [ ] PR title follows conventional commits format
- [ ] Linked related issues (Closes #123, Fixes #456)
- [ ] Added entry to CHANGELOG.md (if user-facing change)
- [ ] Tested on Windows and Linux (if applicable)

### PR Best Practices

**Good PR Characteristics**:
- Focused on a single feature/fix
- Includes tests and documentation
- Has clear description of changes
- References related issues
- Small enough to review (<500 lines when possible)

**Large PRs**:
- Break into smaller PRs when possible
- Use draft PRs for work-in-progress
- Provide detailed description and testing instructions

```bash
# Create draft PR for early feedback
gh pr create --draft --title "feat(rop): ROP chain auto-solver" \
  --body "Work in progress. Feedback welcome on architecture."
```

### PR Labels

| Label | Purpose |
|-------|---------|
| `enhancement` | New features |
| `bug` | Bug fixes |
| `documentation` | Documentation updates |
| `performance` | Performance improvements |
| `security` | Security-related changes |
| `breaking` | Breaking changes |
| `needs-review` | Ready for review |
| `work-in-progress` | Not ready for review |
| `blocked` | Blocked by another issue/PR |

---

## Code Review Process

### For Authors

**Responding to Feedback**:
- Address all comments
- Ask questions if unclear
- Push additional commits (don't force-push during review)
- Mark conversations as resolved after addressing

```bash
# Make requested changes
git add .
git commit -m "fix: address review feedback"
git push origin feature/my-feature
```

**After Approval**:
- Squash commits if needed
- Update PR description
- Wait for maintainer to merge (don't merge your own PR)

### For Reviewers

**Review Checklist**:
- [ ] Code quality and readability
- [ ] Test coverage adequate
- [ ] Documentation complete
- [ ] No security issues
- [ ] Performance implications considered
- [ ] Breaking changes documented
- [ ] CI passes

**Review Guidelines**:
- Be constructive and respectful
- Suggest specific improvements
- Approve if changes are minor
- Request changes if major issues exist

**Review Comments**:
```
# Good - specific and constructive
Consider using `Result` instead of `Option` here to preserve
error context. This will make debugging easier when gadget
search fails.

# Good - positive feedback
Nice use of property-based testing! This will catch edge cases
we wouldn't have thought of.

# Avoid - vague or negative
This doesn't look right.
```

### Merge Strategy

**Squash and Merge** (preferred for most PRs):
- Creates single commit on `develop`
- Keeps history clean
- Combines all PR commits

**Rebase and Merge** (for multiple meaningful commits):
- Preserves individual commits
- Use when commits tell a story
- Requires clean commit history

**Merge Commit** (rarely used):
- Creates explicit merge commit
- Used for major feature branches
- Preserves full branch history

---

## Release Process

### Version Numbers

Follow [Semantic Versioning](https://semver.org/):

```
MAJOR.MINOR.PATCH

Examples:
0.1.0 - Initial alpha release
0.2.0 - Add new feature (backward compatible)
0.2.1 - Bug fix (backward compatible)
1.0.0 - First stable release
2.0.0 - Breaking changes
```

### Release Steps

**1. Prepare Release Branch**:

```bash
git checkout develop
git pull origin develop
git checkout -b release/0.2.0
```

**2. Update Version Information**:

```bash
# Update Cargo.toml
vim Cargo.toml  # Change version = "0.2.0"

# Update CHANGELOG.md
vim CHANGELOG.md  # Add release notes

# Commit changes
git commit -am "chore: bump version to 0.2.0"
```

**3. Final Testing**:

```bash
# Full test suite
cargo test --all-features

# Fuzzing (extended run)
./scripts/run_fuzz.sh 3600  # 1 hour per target

# Benchmarks
cargo bench

# Security audit
cargo audit
cargo deny check

# Manual QA
# Follow docs/QA_CHECKLIST.md
```

**4. Merge to Main**:

```bash
# Merge to main
git checkout main
git merge --no-ff release/0.2.0
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin main --tags

# Merge back to develop
git checkout develop
git merge --no-ff release/0.2.0
git push origin develop
```

**5. Create GitHub Release**:
- Go to https://github.com/ridpath/talon/releases
- Click "Draft a new release"
- Select tag `v0.2.0`
- Copy CHANGELOG.md content
- Attach release artifacts (binaries)
- Publish release

**6. Post-Release**:
- Announce on social media/forums
- Update documentation site
- Monitor issues for release-related bugs

---

## Common Scenarios

### Syncing Fork with Upstream

```bash
# Add upstream remote (first time only)
git remote add upstream https://github.com/ridpath/talon.git

# Fetch and merge upstream changes
git fetch upstream
git checkout develop
git merge upstream/develop
git push origin develop
```

### Resolving Merge Conflicts

```bash
# Update branch
git fetch origin
git rebase origin/develop

# Conflicts occur - resolve in editor
vim conflicted_file.rs

# Mark as resolved
git add conflicted_file.rs
git rebase --continue

# Abort if needed
git rebase --abort
```

### Fixing Mistakes

**Amend Last Commit**:
```bash
# Fix typo in last commit message
git commit --amend -m "fix(parser): correct typo in error message"

# Add forgotten file to last commit
git add forgotten_file.rs
git commit --amend --no-edit
```

**Undo Last Commit (keep changes)**:
```bash
git reset --soft HEAD~1
```

**Undo Last Commit (discard changes)**:
```bash
git reset --hard HEAD~1
```

**Revert Commit (create new commit)**:
```bash
git revert <commit-hash>
```

### Cleaning Up Branch History

```bash
# Interactive rebase (last 5 commits)
git rebase -i HEAD~5

# Options:
# pick   - keep commit as-is
# reword - change commit message
# squash - combine with previous commit
# drop   - remove commit
```

### Working with Stash

```bash
# Save current changes
git stash

# List stashes
git stash list

# Apply most recent stash
git stash apply

# Apply and remove stash
git stash pop

# Apply specific stash
git stash apply stash@{1}

# Clear all stashes
git stash clear
```

---

## Troubleshooting

### Common Issues

**Issue**: Force push rejected
```bash
# Solution: Use --force-with-lease (safer than --force)
git push origin feature/my-branch --force-with-lease
```

**Issue**: Merge conflict during rebase
```bash
# Solution: Resolve conflicts manually
git status  # See conflicted files
vim conflicted_file.rs  # Fix conflicts
git add conflicted_file.rs
git rebase --continue
```

**Issue**: Accidentally committed to wrong branch
```bash
# Solution: Cherry-pick to correct branch
git log  # Find commit hash
git checkout correct-branch
git cherry-pick <commit-hash>
git checkout wrong-branch
git reset --hard HEAD~1
```

**Issue**: Need to undo last push
```bash
# Solution: Revert the commit
git revert HEAD
git push origin branch-name
```

**Issue**: Branch diverged from remote
```bash
# Solution: Rebase onto remote
git fetch origin
git rebase origin/branch-name
git push --force-with-lease
```

### Getting Help

**Git Commands**:
```bash
git help <command>
git <command> --help
man git-<command>
```

**Project-Specific Help**:
- Read `CONTRIBUTING.md`
- Check `docs/` directory
- Ask in GitHub Issues/Discussions
- Review existing PRs for examples

---

## Advanced Topics

### Git Hooks

**Pre-commit Hook** (automatic quality checks):
```bash
# Install project hooks
./scripts/install_hooks.sh  # Linux/macOS
.\scripts\install_hooks.ps1  # Windows

# Runs automatically before each commit:
# - cargo fmt
# - cargo clippy
# - cargo test (changed files)
```

See `docs/PRE_COMMIT_HOOKS.md` for details.

### Git Bisect (finding bugs)

```bash
# Start bisect
git bisect start

# Mark current commit as bad
git bisect bad

# Mark known good commit
git bisect good v0.1.0

# Git will checkout commits
# Test each one:
cargo test
git bisect good  # or 'git bisect bad'

# When done
git bisect reset
```

### Git Worktrees (multiple branches simultaneously)

```bash
# Create worktree for feature branch
git worktree add ../talon-feature feature/my-feature

# Work in both directories simultaneously
cd ../talon-feature
cargo test

# Remove worktree when done
git worktree remove ../talon-feature
```

### Submodules (not used in TALON currently)

```bash
# Add submodule
git submodule add https://github.com/user/repo.git path/

# Update submodules
git submodule update --init --recursive
```

---

## Best Practices Summary

✅ **DO**:
- Use descriptive branch names
- Write clear commit messages (conventional commits)
- Keep commits atomic and focused
- Test before pushing
- Rebase frequently to stay updated
- Respond to PR feedback promptly
- Use `--force-with-lease` instead of `--force`

❌ **DON'T**:
- Commit secrets/keys
- Force push to shared branches (`main`, `develop`)
- Merge your own PRs (wait for approval)
- Commit generated files (unless in `.gitignore`)
- Mix unrelated changes in one commit
- Push failing tests
- Ignore CI failures

---

## Additional Resources

- [Pro Git Book](https://git-scm.com/book/en/v2)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Semantic Versioning](https://semver.org/)
- [GitHub Flow](https://guides.github.com/introduction/flow/)
- [Git Flight Rules](https://github.com/k88hudson/git-flight-rules)

---

## Questions?

- **Documentation**: Read `CONTRIBUTING.md` and `docs/`
- **Issues**: Check existing [GitHub Issues](https://github.com/ridpath/talon/issues)
- **Discussions**: Use [GitHub Discussions](https://github.com/ridpath/talon/discussions)
- **Security**: See `SECURITY.md` for security-related questions

Happy contributing! 🚀
