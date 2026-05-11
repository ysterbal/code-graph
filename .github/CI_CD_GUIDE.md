# CI/CD Pipeline Documentation

## Overview

This repository uses GitHub Actions for continuous integration and deployment. The pipeline ensures code quality, security, and reliability through automated testing and validation.

## Workflows

### 1. **ci.yml** - Main CI/CD Pipeline

Runs on every push to `master`/`main` and all pull requests.

#### Jobs:

| Job Name | Purpose | Trigger |
|----------|---------|---------|
| `code-quality` | Formatting & Clippy checks | PR, Push |
| `build-and-test` | Build & test on multiple Rust versions | PR, Push |
| `dependency-check` | Audit dependencies for vulnerabilities | PR, Push |
| `build-binaries` | Build release binaries | Push to master only |
| `msrv-check` | Verify Minimum Supported Rust Version | PR, Push |
| `docs-build` | Build and upload documentation | PR, Push |

#### Rust Versions Tested:
- ✅ **stable** - Latest stable release
- ✅ **beta** - Next stable version
- ✅ **nightly** - Latest development build

### 2. **code-quality.yml** - Scheduled Quality Checks

Runs on schedule (weekly) and on every PR/push.

#### Jobs:

| Job Name | Purpose | Schedule |
|----------|---------|----------|
| `clippy-check` | Comprehensive linting | Every push/PR |
| `security-audit` | Vulnerability scanning | Weekly + every push/PR |
| `format-lint` | Formatting validation | Every push/PR |
| `performance-check` | Binary size & performance | Every push/PR |
| `dependency-tree` | Dependency analysis | Every push/PR |

## Local Testing

You can test the workflows locally using [act](https://github.com/nektos/act):

```bash
# Install act (macOS)
brew install act

# Run all workflows locally
act push

# Run specific workflow
act -j code-quality

# Run with secrets
act push --secret FILE=.env
```

## Configuration Options

### Environment Variables

Set these in your repository settings or workflow:

```yaml
env:
  CARGO_TERM_COLOR: always  # Always show colors in output
```

### Secrets

Required secrets (configure in GitHub → Settings → Secrets):

| Secret | Purpose |
|--------|---------|
| `CRATES_IO_TOKEN` | For publishing to crates.io |
| `GITHUB_TOKEN` | Auto-generated, used for audits |

### Caching Strategy

The workflow caches:
- `~/.cargo/registry` - Crates.io registry
- `~/.cargo/git` - Git repositories
- `target` - Build artifacts

Cache key uses `Cargo.lock` hash for optimal invalidation.

## Customization

### Adding New Jobs

1. Create a new job in the YAML file
2. Follow the naming convention: `lowercase-with-dashes`
3. Add `runs-on` and `steps` as shown in existing jobs

### Changing Rust Versions

Edit the matrix strategy in `ci.yml`:

```yaml
strategy:
  matrix:
    rust: [stable, beta, nightly]  # Add/remove versions here
```

### Skipping Jobs

Add conditions to skip jobs:

```yaml
jobs:
  my-job:
    if: github.event_name == 'push'  # Only run on push
    steps:
      - name: Do something
        run: echo "Only runs on push"
```

### Custom Clippy Lints

Add custom lint configurations in `Cargo.toml`:

```toml
[lints.rust]
unsafe_code = "forbid"
unreachable_pub = "warn"

[lints.clippy]
all = "deny"
pedantic = "warn"
```

## Troubleshooting

### Common Issues

#### Cache Not Working
- Ensure `Cargo.lock` exists in repository
- Check cache key format matches examples above

#### Clippy Fails
- Run `cargo clippy --fix` locally to auto-fix issues
- Check for denied lints: `cargo clippy -- -D warnings`

#### Tests Time Out
- Increase timeout in workflow: `timeout-minutes: 30`
- Optimize slow tests or split into separate jobs

### Debugging Workflows

1. View workflow run logs: GitHub → Actions → Select workflow → Run logs
2. Enable workflow debugging: Settings → Actions → Enable debug logging
3. Use `step-debugging` action for interactive debugging

## Best Practices

✅ **DO:**
- Keep workflows modular and focused
- Use matrix builds for parallel testing
- Cache dependencies to speed up builds
- Write clear commit messages
- Update dependencies regularly

❌ **DON'T:**
- Hardcode secrets in workflows
- Skip important quality checks
- Use outdated action versions
- Ignore failing tests
- Commit `target/` directory

## Security Considerations

1. **Dependency Scanning**: Automated with `cargo-audit`
2. **Secret Management**: Use GitHub Secrets, never commit credentials
3. **Action Pinning**: Always pin actions to specific versions/sha
4. **Code Signing**: Consider signing binaries for release

## Contributing

When contributing to this project:

1. Ensure all tests pass locally: `cargo test --all-targets`
2. Format code: `cargo fmt`
3. Run clippy: `cargo clippy -- -D warnings`
4. Submit PR for review
5. CI will automatically validate your changes

## Support

For issues with the CI/CD pipeline:
- Check workflow logs in GitHub Actions
- Review [GitHub Actions documentation](https://docs.github.com/en/actions)
- See [Rust CI/CD best practices](https://github.com/rust-lang/cargo/issues/8623)

---

**Last Updated:** May 11, 2026  
**Maintained by:** Code Graph Tool Team
