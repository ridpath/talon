# Pre-commit Hooks & Development Tools

## Overview

TALON includes a comprehensive pre-commit hook system that automatically validates code quality, security, and correctness before each commit. This ensures that all code entering the repository meets our high standards.

## Features

### Automated Checks

The pre-commit hooks perform the following checks:

1. **Code Formatting** (`cargo fmt`)
   - Ensures consistent Rust code formatting
   - Follows the official Rust style guide
   - Auto-fixable: `cargo fmt`

2. **Linting** (`cargo clippy`)
   - Detects common mistakes and anti-patterns
   - Enforces best practices
   - Treats warnings as errors (`-D warnings`)

3. **Compilation** (`cargo check`)
   - Verifies code compiles successfully
   - Checks all features and targets
   - Fast alternative to full build

4. **Fast Unit Tests** (`cargo test`)
   - Runs library and binary tests only
   - Skips slow integration tests
   - Uses parallel test execution

5. **Security Audit** (`cargo deny`)
   - Checks for security vulnerabilities
   - Validates dependency licenses
   - Detects banned/yanked crates

6. **File Pattern Checks**
   - Blocks large files (>1MB)
   - Prevents committing private keys
   - Detects exploit artifacts
   - Scans for potential secrets

7. **Debug Statement Detection**
   - Warns about `println!`, `dbg!`, `eprintln!` in non-test code
   - Helps keep production code clean

## Installation

### Quick Install

**Linux/macOS:**
```bash
./scripts/install_hooks.sh
```

**Windows (PowerShell):**
```powershell
.\scripts\install_hooks.ps1
```

### Manual Installation

1. **Copy hook to `.git/hooks/`:**
   ```bash
   cp scripts/pre-commit.sh .git/hooks/pre-commit
   chmod +x .git/hooks/pre-commit
   ```

2. **Or create symlink:**
   ```bash
   ln -sf ../../scripts/pre-commit.sh .git/hooks/pre-commit
   ```

### Pre-commit Framework (Optional)

For advanced features like language-specific hooks and better hook management:

1. **Install pre-commit framework:**
   ```bash
   pip install pre-commit
   ```

2. **Install hooks:**
   ```bash
   pre-commit install
   ```

3. **Run manually:**
   ```bash
   pre-commit run --all-files
   ```

## Usage

### Normal Workflow

Once installed, hooks run automatically on `git commit`:

```bash
git add src/my_feature.rs
git commit -m "feat: add new feature"

# Hooks run automatically:
# ✓ Checking code formatting...
# ✓ Running Clippy lints...
# ✓ Checking compilation...
# ✓ Running fast unit tests...
# ✓ All pre-commit checks passed!
```

### Skipping Hooks (Not Recommended)

In rare cases where you need to bypass hooks:

```bash
git commit --no-verify -m "WIP: work in progress"
```

**Warning:** Only use `--no-verify` for:
- Work-in-progress commits on feature branches
- Commits that will be squashed before merging
- Emergency hotfixes (followed by immediate fix)

### Running Hooks Manually

Test hooks without committing:

**Linux/macOS:**
```bash
./scripts/pre-commit.sh
```

**Windows:**
```powershell
.\scripts\pre-commit.ps1
```

**With pre-commit framework:**
```bash
pre-commit run --all-files
```

## Configuration

### `.pre-commit-config.yaml`

The main configuration file for the pre-commit framework. Defines all hooks and their settings.

**Key sections:**
```yaml
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        entry: cargo fmt --all --
        pass_filenames: false
```

### `.markdownlint.json`

Configuration for Markdown linting:
```json
{
  "MD013": false,  // Disable line length rule
  "MD033": false   // Allow inline HTML
}
```

### `.secrets.baseline`

Baseline file for `detect-secrets` plugin. Tracks known false positives:
```bash
# Update baseline:
detect-secrets scan --baseline .secrets.baseline
```

## Customization

### Modifying Checks

Edit `scripts/pre-commit.sh` or `scripts/pre-commit.ps1`:

```bash
# Example: Add custom check
echo "[8/8] Running custom check..."
if ! my_custom_command; then
    FAILED=1
fi
```

### Adjusting Strictness

**Disable specific Clippy lints:**
```bash
# In pre-commit.sh, change:
cargo clippy --all-features --all-targets -- -D warnings -A clippy::my_lint
```

**Skip slow tests:**
```bash
# Already configured - runs only lib/bin tests
cargo test --lib --bins --all-features
```

### Adding File Pattern Checks

Edit the forbidden patterns section:

```bash
# Block additional file types
if echo "$STAGED_FILES" | grep -qE '\.(db|sqlite)$'; then
    echo "ERROR: Database files detected"
    FAILED=1
fi
```

## Troubleshooting

### Hook Not Running

**Symptoms:** Commits succeed without running checks

**Solutions:**
1. Verify installation:
   ```bash
   ls -la .git/hooks/pre-commit
   ```

2. Check permissions:
   ```bash
   chmod +x .git/hooks/pre-commit
   ```

3. Reinstall:
   ```bash
   ./scripts/install_hooks.sh
   ```

### Cargo Not Found

**Symptoms:** `ERROR: cargo not found`

**Solution:** Install Rust toolchain:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Clippy Warnings Fail

**Symptoms:** Hook fails on Clippy warnings

**Solutions:**
1. **Fix warnings (recommended):**
   ```bash
   cargo clippy --fix --all-features --allow-dirty
   ```

2. **Temporary bypass:**
   ```bash
   git commit --no-verify
   ```

3. **Adjust strictness:**
   Edit hook to use `-W warnings` instead of `-D warnings`

### Slow Hook Execution

**Symptoms:** Hooks take >30 seconds

**Solutions:**
1. **Use incremental compilation:**
   ```bash
   export CARGO_INCREMENTAL=1
   ```

2. **Skip integration tests (already configured)**

3. **Use cargo check instead of full tests:**
   Edit hook to remove `cargo test` for faster commits

### Windows PowerShell Execution Policy

**Symptoms:** `execution of scripts is disabled`

**Solution:**
```powershell
# Run as Administrator
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Pre-commit Framework Issues

**Symptoms:** `pre-commit: command not found`

**Solutions:**
1. Install Python pre-commit:
   ```bash
   pip install pre-commit
   ```

2. Use standalone scripts instead:
   ```bash
   ./scripts/pre-commit.sh
   ```

## Best Practices

### 1. Run Hooks Before Committing

Test locally before committing:
```bash
cargo fmt
cargo clippy --fix
cargo test
./scripts/pre-commit.sh
git commit
```

### 2. Keep Commits Atomic

Make small, focused commits that pass all checks:
```bash
# Good
git commit -m "fix: resolve clippy warning in parser.rs"

# Avoid
git commit -m "WIP: multiple features" --no-verify
```

### 3. Fix Issues, Don't Skip

Always fix issues rather than bypassing hooks:
```bash
# Bad
git commit --no-verify

# Good
cargo fmt
cargo clippy --fix
git commit
```

### 4. Update Hooks Regularly

Pull latest hook updates:
```bash
git pull
./scripts/install_hooks.sh  # Reinstall if hooks changed
```

### 5. Use Feature Branches

Test extensively on feature branches:
```bash
git checkout -b feature/my-feature
# Make changes, commit with hooks
git push origin feature/my-feature
# Create PR - CI runs full checks
```

## Integration with CI/CD

Pre-commit hooks complement CI/CD pipelines:

**Local (Pre-commit Hooks):**
- Fast feedback (<30s)
- Catches obvious errors
- Runs before every commit

**CI/CD (GitHub Actions):**
- Comprehensive testing (minutes)
- Multiple platforms
- Integration and E2E tests

**Workflow:**
```
Developer → Pre-commit → Commit → Push → CI/CD → Merge
            (fast)                       (thorough)
```

## Advanced Features

### Conditional Checks

Skip certain checks based on branch:
```bash
BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [[ "$BRANCH" != "main" ]]; then
    echo "Skipping slow checks on feature branch"
    SKIP_TESTS=1
fi
```

### Auto-fixing

Some checks can auto-fix issues:
```bash
# Auto-fix formatting
cargo fmt

# Auto-fix some Clippy warnings
cargo clippy --fix --allow-dirty

# Auto-fix with pre-commit
pre-commit run --all-files
```

### Hook Chaining

Run multiple hook types:
```bash
.git/hooks/pre-commit  # Quality checks
.git/hooks/pre-push    # Full test suite
```

### Custom Hook Scripts

Create additional hooks:

**`.git/hooks/commit-msg`** (validate commit messages):
```bash
#!/bin/bash
MSG=$(cat "$1")
if ! echo "$MSG" | grep -qE '^(feat|fix|docs|test|refactor):'; then
    echo "ERROR: Commit message must start with type (feat|fix|docs|test|refactor)"
    exit 1
fi
```

**`.git/hooks/pre-push`** (run full tests):
```bash
#!/bin/bash
echo "Running full test suite before push..."
cargo test --all-features
```

## Maintenance

### Updating Hooks

When hooks are updated in the repository:

```bash
git pull
./scripts/install_hooks.sh
```

### Removing Hooks

```bash
rm .git/hooks/pre-commit
pre-commit uninstall  # If using framework
```

### Debugging Hooks

Enable verbose output:
```bash
# In pre-commit.sh, add:
set -x  # Print each command

# Run hook
./scripts/pre-commit.sh
```

## Related Documentation

- [TESTING.md](TESTING.md) - Comprehensive testing guide
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines
- [SECURITY.md](../SECURITY.md) - Security policies
- [CI.yml](../.github/workflows/ci.yml) - CI/CD configuration

## Support

**Issues with hooks?**
1. Check this documentation
2. Run `./scripts/install_hooks.sh` again
3. Review [GitHub Issues](../../issues)
4. Contact maintainers

**Contributing:**
- Suggest improvements via PR
- Report bugs in hook scripts
- Share custom hooks with the community
