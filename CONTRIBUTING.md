# Contributing to a8n.tools

Thank you for your interest in contributing to a8n.tools!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/a8n-tools.git`
3. Create a feature branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Run tests: `make test`
6. Commit your changes following our commit message format
7. Push to your fork: `git push origin feature/your-feature-name`
8. Open a Pull Request

## Development Setup

See the [README](README.md) for detailed setup instructions.

## Issue Guidelines

### Reporting Bugs

When reporting a bug, please include:

- A clear, descriptive title
- Steps to reproduce the issue
- Expected behavior
- Actual behavior
- Environment details (OS, browser, versions)
- Screenshots if applicable
- Relevant logs or error messages

### Requesting Features

When requesting a feature:

- Check if the feature has already been requested
- Describe the problem you're trying to solve
- Explain your proposed solution
- Consider alternatives you've thought about

## Pull Request Process

1. **Create an issue first** for significant changes
2. **Keep PRs focused** - one feature or fix per PR
3. **Update documentation** if needed
4. **Add tests** for new functionality
5. **Ensure tests pass** before requesting review
6. **Request review** from a maintainer

### PR Checklist

- [ ] Code follows project style guidelines
- [ ] Tests added/updated and passing
- [ ] Documentation updated
- [ ] Commit messages follow format
- [ ] No merge conflicts
- [ ] CI checks pass

## Commit Message Format

We follow the Conventional Commits specification:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Types

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, semicolons, etc.)
- `refactor`: Code changes that neither fix bugs nor add features
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `build`: Changes to build system or dependencies
- `ci`: Changes to CI configuration
- `chore`: Other changes that don't modify src or test files

### Examples

```
feat(auth): add magic link authentication

Implement passwordless authentication via magic links.
Users can now sign in by clicking a link sent to their email.

Closes #123
```

```
fix(api): handle null values in user response

Previously, null values in optional fields caused JSON
serialization errors. This fix properly handles null values.

Fixes #456
```

## Code Style

### Rust

- Follow the Rust API Guidelines
- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`
- Write documentation comments for public items
- Avoid `unwrap()` in production code - use proper error handling

### TypeScript/React

- Use TypeScript strict mode
- Follow ESLint configuration
- Use Prettier for formatting
- Use functional components with hooks
- Keep components small and focused

### General

- Write clear, descriptive variable and function names
- Keep functions focused on a single task
- Add comments for complex logic
- Update types when modifying data structures

## Testing

### API Tests

```bash
# Run all API tests
make test-api

# Run specific test
cd api && cargo test test_name

# Run with logging
cd api && RUST_LOG=debug cargo test
```

### Frontend Tests

```bash
# Run all frontend tests
make test-frontend

# Run with watch mode
cd frontend && npm test -- --watch
```

## Code Review Process

1. All PRs require at least one approval
2. Reviewers will check:
   - Code quality and style
   - Test coverage
   - Documentation
   - Security considerations
   - Performance implications
3. Address all review comments
4. Re-request review after making changes

## Questions?

If you have questions, feel free to:

- Open an issue with the "question" label
- Join our Discord server (link TBD)
- Email the maintainers

Thank you for contributing!
