# Security Auditing Guide

## Overview

TALON implements a comprehensive security auditing infrastructure to ensure the codebase remains free of known vulnerabilities and adheres to security best practices. This guide explains the security tools, processes, and workflows in place.

## Security Tools

### 1. cargo-audit

**Purpose**: Detects known security vulnerabilities in Rust dependencies.

**What it checks**:
- CVEs and RustSec advisories in the dependency tree
- Known vulnerabilities in direct and transitive dependencies
- Unmaintained crates with security implications

**Configuration**: Uses the RustSec Advisory Database (automatically updated)

**Usage**:
```bash
# Basic audit
cargo audit

# Audit with warnings as errors
cargo audit --deny warnings

# Update advisory database
cargo audit fetch
```

**CI Integration**: Runs automatically on push, PR, and weekly schedule

### 2. cargo-deny

**Purpose**: Comprehensive dependency management and policy enforcement.

**What it checks**:
- **Advisories**: Security vulnerabilities (similar to cargo-audit)
- **Licenses**: Ensures dependencies use approved open-source licenses
- **Bans**: Prevents use of explicitly banned crates or versions
- **Sources**: Validates dependencies come from trusted registries

**Configuration**: `deny.toml` in project root

**Usage**:
```bash
# Check all policies
cargo deny check

# Check specific policy
cargo deny check advisories
cargo deny check licenses
cargo deny check bans
cargo deny check sources

# List all dependencies
cargo deny list
```

**CI Integration**: Runs on all pull requests and pushes

### 3. Dependabot

**Purpose**: Automated dependency updates and security patches.

**What it does**:
- Monitors dependencies for security updates
- Creates automated PRs for dependency upgrades
- Groups related dependencies for easier review
- Supports Rust (Cargo), JavaScript (npm), and GitHub Actions

**Configuration**: `.github/dependabot.yml`

**Features**:
- Weekly update schedule (Mondays at 9 AM)
- Grouped dependency updates (crypto, web3, testing, etc.)
- Automatic labeling and commit message formatting
- Ignores patch-level updates for stable dependencies

### 4. GitHub Dependency Review

**Purpose**: Prevents introduction of vulnerable dependencies in pull requests.

**What it checks**:
- New dependencies added in PRs
- Version changes that introduce known vulnerabilities
- License compliance for new dependencies

**Configuration**: `.github/workflows/security.yml`

**Threshold**: Fails on "moderate" or higher severity vulnerabilities

## Security Audit Workflow

### Automated Checks (CI/CD)

1. **On Every Push/PR**:
   ```
   ├── cargo-audit: Vulnerability scanning
   ├── cargo-deny: License & policy checks
   └── dependency-review: PR-specific analysis
   ```

2. **Weekly Schedule**:
   ```
   ├── cargo-audit: Fresh scan with updated advisory DB
   ├── dependabot: Automated dependency updates
   └── Security alerts: Email notifications
   ```

### Manual Security Audits

Run comprehensive security audit locally:

**Linux/macOS**:
```bash
./scripts/security_audit.sh
```

**Windows**:
```powershell
.\scripts\security_audit.ps1
```

**What the script does**:
1. Installs cargo-audit and cargo-deny (if needed)
2. Scans for known vulnerabilities
3. Validates licenses and sources
4. Checks for banned dependencies
5. Analyzes dependency tree
6. Reports outdated dependencies
7. Provides color-coded summary

## Configuration Files

### deny.toml

Main cargo-deny configuration file.

**Key sections**:

```toml
[advisories]
vulnerability = "deny"          # Deny any known vulnerabilities
severity-threshold = "medium"   # Minimum severity to report

[licenses]
allow = ["MIT", "Apache-2.0", "BSD-3-Clause", ...]
deny = ["GPL-3.0", "AGPL-3.0"] # Copyleft licenses

[bans]
multiple-versions = "warn"      # Warn on duplicate dependencies

[sources]
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

**Customization**:
- Add allowed licenses: Update `licenses.allow` array
- Ignore specific advisories: Add to `advisories.ignore` (with justification)
- Ban specific crates: Add to `bans.deny`
- Allow git dependencies: Add to `sources.allow-git`

### .github/dependabot.yml

Dependabot configuration for automated updates.

**Key features**:
- Separate update schedules for Cargo, npm, and GitHub Actions
- Dependency grouping (crypto, web3, testing, etc.)
- Custom commit message prefixes
- Label automation
- PR limits to avoid spam

**Customization**:
- Adjust update frequency: Change `schedule.interval`
- Modify PR limits: Change `open-pull-requests-limit`
- Add/remove groups: Update `groups` section
- Change assignees/reviewers: Update respective fields

### .github/workflows/security.yml

GitHub Actions workflow for automated security checks.

**Jobs**:
1. **security-audit**: Runs cargo-audit with deny-warnings
2. **cargo-deny**: Runs all cargo-deny checks
3. **dependency-review**: Analyzes PRs for vulnerable dependencies

**Triggers**:
- Push to main/develop branches
- Pull requests to main/develop
- Weekly schedule (Sunday at midnight)

## Security Policies

### Vulnerability Severity Levels

| Severity | Action | Timeline |
|----------|--------|----------|
| Critical | Immediate fix required | < 24 hours |
| High | Priority fix | < 7 days |
| Medium | Planned fix | < 30 days |
| Low | Best effort | Next release |

### License Policies

**Allowed Licenses** (Permissive):
- MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause
- ISC, 0BSD, Zlib, CC0-1.0
- MPL-2.0, Unicode-DFS-2016

**Denied Licenses** (Copyleft):
- GPL-3.0, AGPL-3.0

**Policy Rationale**: TALON is designed for security research and must remain freely usable in commercial and closed-source contexts.

### Dependency Sources

**Allowed**:
- Official crates.io registry
- Approved git repositories (case-by-case)

**Denied**:
- Unknown registries
- Arbitrary git repositories
- Local path dependencies (in production)

## Common Security Issues

### 1. Known Vulnerabilities

**Symptom**: `cargo audit` fails with vulnerability report

**Resolution**:
1. Review the advisory: `cargo audit --json | jq`
2. Update the vulnerable dependency: `cargo update -p <crate_name>`
3. If no fix available, consider alternatives or mitigations
4. Document temporary ignores in `deny.toml` with justification

**Example**:
```toml
[advisories]
ignore = [
    { id = "RUSTSEC-2024-0001", reason = "False positive - we don't use affected feature" }
]
```

### 2. License Violations

**Symptom**: `cargo deny check licenses` fails

**Resolution**:
1. Identify the violating dependency: `cargo deny list`
2. Options:
   - Add license to allowed list (if acceptable)
   - Find alternative dependency with compatible license
   - Remove the dependency if not critical
3. Update `deny.toml` accordingly

### 3. Duplicate Dependencies

**Symptom**: Warning about multiple versions of same crate

**Resolution**:
1. Analyze dependency tree: `cargo tree -d`
2. Update dependencies to use compatible versions
3. Use `cargo update` to resolve conflicts
4. Consider explicit version pinning in `Cargo.toml`

### 4. Unmaintained Dependencies

**Symptom**: cargo-audit reports unmaintained crates

**Resolution**:
1. Assess criticality of the dependency
2. Search for maintained forks or alternatives
3. Consider vendoring and maintaining in-tree
4. Document the decision in project documentation

## Best Practices

### For Developers

1. **Before Committing**:
   ```bash
   cargo audit
   cargo deny check
   cargo test
   ```

2. **Adding Dependencies**:
   - Research the crate's maintenance status
   - Check GitHub activity and issue tracker
   - Verify license compatibility
   - Review recent security advisories

3. **Updating Dependencies**:
   - Read changelogs for breaking changes
   - Test thoroughly after updates
   - Update one dependency at a time for critical crates
   - Batch non-critical updates

4. **Security Reviews**:
   - Review Dependabot PRs promptly
   - Investigate any new security warnings
   - Participate in security discussions

### For Maintainers

1. **Weekly Tasks**:
   - Review Dependabot PRs
   - Check CI security job results
   - Update advisory database: `cargo audit fetch`

2. **Monthly Tasks**:
   - Run full security audit: `./scripts/security_audit.sh`
   - Review dependency licenses
   - Check for unmaintained dependencies
   - Update security documentation

3. **On Security Advisory**:
   - Assess impact on TALON
   - Create issue or PR to address
   - Notify users if critical
   - Document in security advisories

## Integration with Development Workflow

### Pre-commit Hooks

Add to `.git/hooks/pre-commit` (or use pre-commit framework):
```bash
#!/bin/bash
cargo audit --deny warnings
cargo deny check advisories
```

### VS Code Integration

Recommended extensions:
- rust-analyzer: Real-time linting
- Even Better TOML: Edit deny.toml with validation

### CI/CD Pipeline

Security checks run at multiple stages:
```
Developer Commit
    ↓
Pre-commit Hooks (local)
    ↓
GitHub Push
    ↓
Security Workflow (CI)
    ├── cargo-audit
    ├── cargo-deny
    └── dependency-review (PR only)
    ↓
Required Status Checks
    ↓
Merge to Main
```

## Troubleshooting

### cargo-audit fails to connect

**Issue**: Network timeout fetching advisory database

**Solution**:
```bash
# Use offline mode with cached DB
cargo audit --offline

# Or set proxy if behind firewall
export HTTPS_PROXY=http://proxy.example.com:8080
cargo audit
```

### cargo-deny fails with "unknown license"

**Issue**: Dependency uses non-standard license identifier

**Solution**:
1. Check the crate's actual license on crates.io or GitHub
2. Add to `deny.toml`:
   ```toml
   [[licenses.clarify]]
   name = "crate-name"
   expression = "MIT OR Apache-2.0"
   license-files = [{ path = "LICENSE", hash = 0x... }]
   ```

### Dependabot PRs failing tests

**Issue**: Automated update breaks compatibility

**Solution**:
1. Review the changelog for breaking changes
2. Update code to match new API
3. Or pin to previous version temporarily:
   ```toml
   [dependencies]
   problematic-crate = "=1.2.3"  # Pin exact version
   ```
4. Create issue to track required migration

## Resources

### Documentation

- [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [cargo-deny](https://embarkstudios.github.io/cargo-deny/)
- [RustSec Advisory Database](https://rustsec.org/)
- [Dependabot Documentation](https://docs.github.com/en/code-security/dependabot)

### Security Advisories

- [RustSec Advisories](https://rustsec.org/advisories/)
- [GitHub Advisory Database](https://github.com/advisories)
- [CVE Database](https://cve.mitre.org/)

### Best Practices

- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [OWASP Dependency Check](https://owasp.org/www-project-dependency-check/)
- [Supply Chain Security](https://slsa.dev/)

## Continuous Improvement

TALON's security posture is continuously improved through:

1. **Regular Updates**: Weekly dependency updates via Dependabot
2. **Advisory Monitoring**: Automated scanning for new vulnerabilities
3. **Community Review**: Open-source security review and contributions
4. **Audit Trail**: Full git history of security-related changes
5. **Documentation**: This guide is updated with new threats and mitigations

## Reporting Security Issues

See [SECURITY.md](../SECURITY.md) for reporting vulnerabilities in TALON itself.

---

**Last Updated**: January 15, 2026  
**Maintained By**: TALON Security Team
