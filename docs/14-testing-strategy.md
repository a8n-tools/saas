# 14 - Testing Strategy

## Overview

This document contains prompts for implementing comprehensive testing including unit tests, integration tests, and end-to-end tests.

## Prerequisites
- All application features complete
- Development environment running

---

## Prompt 14.1: Rust Unit Testing Setup

```text
Set up unit testing infrastructure for the Rust API.

Organize test modules in each source file:
```rust
// src/services/password.rs

pub struct PasswordService {
    // ...
}

impl PasswordService {
    pub fn hash(&self, password: &str) -> Result<String, AppError> {
        // Implementation
    }

    pub fn verify(&self, password: &str, hash: &str) -> Result<bool, AppError> {
        // Implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let service = PasswordService::new();
        let password = "SecurePassword123!";

        let hash = service.hash(password).unwrap();
        assert!(service.verify(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_wrong_password() {
        let service = PasswordService::new();
        let hash = service.hash("CorrectPassword123!").unwrap();

        assert!(!service.verify("WrongPassword123!", &hash).unwrap());
    }

    #[test]
    fn test_hash_is_unique() {
        let service = PasswordService::new();
        let password = "SamePassword123!";

        let hash1 = service.hash(password).unwrap();
        let hash2 = service.hash(password).unwrap();

        assert_ne!(hash1, hash2);  // Different salts
    }
}
```

Create src/testing/mod.rs for test utilities:
```rust
pub mod fixtures;
pub mod mocks;
pub mod helpers;

#[cfg(test)]
pub use fixtures::*;
#[cfg(test)]
pub use mocks::*;
#[cfg(test)]
pub use helpers::*;
```

Create test fixtures:
```rust
// src/testing/fixtures.rs

pub fn test_user() -> User {
    User {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        email_verified: true,
        password_hash: Some("$argon2id$...".to_string()),
        role: "subscriber".to_string(),
        subscription_status: "active".to_string(),
        // ...
    }
}

pub fn test_admin() -> User {
    User {
        role: "admin".to_string(),
        ..test_user()
    }
}
```

Run tests:
```bash
cargo test
cargo test -- --nocapture  # Show println output
cargo test --test '*' -- --test-threads=1  # Run sequentially
```
```

---

## Prompt 14.2: Integration Testing for API

```text
Set up integration testing for API endpoints.

Create tests/api/mod.rs:
```rust
mod auth;
mod users;
mod subscriptions;
mod applications;
mod admin;

use a8n_api::{create_app, config::Config};
use actix_web::{test, web, App};
use sqlx::PgPool;

pub struct TestApp {
    pub pool: PgPool,
    pub config: Config,
}

impl TestApp {
    pub async fn new() -> Self {
        dotenv::from_filename(".env.test").ok();

        let config = Config::from_env().expect("Failed to load config");
        let pool = PgPool::connect(&config.database_url)
            .await
            .expect("Failed to connect to database");

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        Self { pool, config }
    }

    pub async fn cleanup(&self) {
        // Truncate all tables
        sqlx::query("TRUNCATE users, subscriptions, audit_logs CASCADE")
            .execute(&self.pool)
            .await
            .unwrap();
    }
}
```

Create tests/api/auth.rs:
```rust
use super::*;
use actix_web::{http::StatusCode, test};

#[actix_web::test]
async fn test_register_success() {
    let app = TestApp::new().await;
    let service = test::init_service(create_app(app.pool.clone())).await;

    let req = test::TestRequest::post()
        .uri("/v1/auth/register")
        .set_json(json!({
            "email": "newuser@example.com",
            "password": "SecurePassword123!"
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["email"], "newuser@example.com");

    app.cleanup().await;
}

#[actix_web::test]
async fn test_register_duplicate_email() {
    let app = TestApp::new().await;

    // Create existing user
    sqlx::query!(
        "INSERT INTO users (email, password_hash) VALUES ($1, $2)",
        "existing@example.com",
        "hash"
    )
    .execute(&app.pool)
    .await
    .unwrap();

    let service = test::init_service(create_app(app.pool.clone())).await;

    let req = test::TestRequest::post()
        .uri("/v1/auth/register")
        .set_json(json!({
            "email": "existing@example.com",
            "password": "SecurePassword123!"
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);

    app.cleanup().await;
}

#[actix_web::test]
async fn test_login_success() {
    let app = TestApp::new().await;
    let password_service = PasswordService::new();
    let hash = password_service.hash("SecurePassword123!").unwrap();

    sqlx::query!(
        "INSERT INTO users (email, password_hash, email_verified) VALUES ($1, $2, true)",
        "user@example.com",
        hash
    )
    .execute(&app.pool)
    .await
    .unwrap();

    let service = test::init_service(create_app(app.pool.clone())).await;

    let req = test::TestRequest::post()
        .uri("/v1/auth/login")
        .set_json(json!({
            "email": "user@example.com",
            "password": "SecurePassword123!"
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Check cookies are set
    let cookies: Vec<_> = resp.response().cookies().collect();
    assert!(cookies.iter().any(|c| c.name() == "access_token"));
    assert!(cookies.iter().any(|c| c.name() == "refresh_token"));

    app.cleanup().await;
}

#[actix_web::test]
async fn test_login_invalid_credentials() {
    // Test with wrong password
}

#[actix_web::test]
async fn test_protected_endpoint_without_auth() {
    let app = TestApp::new().await;
    let service = test::init_service(create_app(app.pool.clone())).await;

    let req = test::TestRequest::get()
        .uri("/v1/users/me")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    app.cleanup().await;
}
```

Run integration tests:
```bash
cargo test --test api
```
```

---

## Prompt 14.3: Frontend Unit Testing

```text
Set up frontend unit testing with Vitest.

Install dependencies:
```bash
bun add -D vitest @testing-library/react @testing-library/jest-dom @testing-library/user-event jsdom
```

Configure vitest in vite.config.ts:
```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
    coverage: {
      reporter: ['text', 'json', 'html'],
      exclude: ['node_modules/', 'src/test/'],
    },
  },
});
```

Create src/test/setup.ts:
```typescript
import '@testing-library/jest-dom';
import { afterEach } from 'vitest';
import { cleanup } from '@testing-library/react';

afterEach(() => {
  cleanup();
});
```

Create test utilities:
```typescript
// src/test/utils.tsx
import { ReactNode } from 'react';
import { render, RenderOptions } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { BrowserRouter } from 'react-router-dom';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
    },
  },
});

function AllProviders({ children }: { children: ReactNode }) {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        {children}
      </BrowserRouter>
    </QueryClientProvider>
  );
}

export function renderWithProviders(
  ui: React.ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>
) {
  return render(ui, { wrapper: AllProviders, ...options });
}

export * from '@testing-library/react';
export { renderWithProviders as render };
```

Write component tests:
```typescript
// src/components/PasswordStrengthIndicator.test.tsx
import { describe, it, expect } from 'vitest';
import { render, screen } from '@/test/utils';
import { PasswordStrengthIndicator } from './PasswordStrengthIndicator';

describe('PasswordStrengthIndicator', () => {
  it('shows weak for short passwords', () => {
    render(<PasswordStrengthIndicator password="short" />);
    expect(screen.getByText(/weak/i)).toBeInTheDocument();
  });

  it('shows strong for passwords meeting all criteria', () => {
    render(<PasswordStrengthIndicator password="SecurePass123!" />);
    expect(screen.getByText(/strong/i)).toBeInTheDocument();
  });

  it('shows checkmarks for met requirements', () => {
    render(<PasswordStrengthIndicator password="Password123" />);
    expect(screen.getByText(/uppercase/i)).toHaveClass('text-green-500');
    expect(screen.getByText(/lowercase/i)).toHaveClass('text-green-500');
    expect(screen.getByText(/number/i)).toHaveClass('text-green-500');
    expect(screen.getByText(/special/i)).not.toHaveClass('text-green-500');
  });
});
```

Run tests:
```bash
bun test
bun test -- --coverage
```
```

---

## Prompt 14.4: Frontend Integration Testing

```text
Create frontend integration tests with mocked API.

Install MSW for API mocking:
```bash
bun add -D msw
```

Create src/test/mocks/handlers.ts:
```typescript
import { rest } from 'msw';

export const handlers = [
  rest.post('/v1/auth/login', (req, res, ctx) => {
    const { email, password } = req.body as any;

    if (email === 'test@example.com' && password === 'Password123!') {
      return res(
        ctx.status(200),
        ctx.json({
          success: true,
          data: {
            user: {
              id: '123',
              email: 'test@example.com',
              role: 'subscriber',
              subscription_status: 'active',
            },
          },
        })
      );
    }

    return res(
      ctx.status(401),
      ctx.json({
        success: false,
        error: {
          code: 'INVALID_CREDENTIALS',
          message: 'Invalid email or password',
        },
      })
    );
  }),

  rest.get('/v1/users/me', (req, res, ctx) => {
    return res(
      ctx.status(200),
      ctx.json({
        success: true,
        data: {
          id: '123',
          email: 'test@example.com',
          role: 'subscriber',
          subscription_status: 'active',
        },
      })
    );
  }),

  rest.get('/v1/applications', (req, res, ctx) => {
    return res(
      ctx.status(200),
      ctx.json({
        success: true,
        data: {
          applications: [
            {
              id: '1',
              slug: 'rus',
              display_name: 'RUS - URL Shortener',
              description: 'URL shortening service',
            },
          ],
        },
      })
    );
  }),
];
```

Create integration tests:
```typescript
// src/pages/auth/LoginPage.test.tsx
import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { setupServer } from 'msw/node';
import { render, screen, waitFor } from '@/test/utils';
import userEvent from '@testing-library/user-event';
import { LoginPage } from './LoginPage';
import { handlers } from '@/test/mocks/handlers';

const server = setupServer(...handlers);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('LoginPage', () => {
  it('logs in successfully with valid credentials', async () => {
    const user = userEvent.setup();
    render(<LoginPage />);

    await user.type(screen.getByLabelText(/email/i), 'test@example.com');
    await user.type(screen.getByLabelText(/password/i), 'Password123!');
    await user.click(screen.getByRole('button', { name: /sign in/i }));

    await waitFor(() => {
      expect(window.location.pathname).toBe('/dashboard');
    });
  });

  it('shows error for invalid credentials', async () => {
    const user = userEvent.setup();
    render(<LoginPage />);

    await user.type(screen.getByLabelText(/email/i), 'wrong@example.com');
    await user.type(screen.getByLabelText(/password/i), 'WrongPassword');
    await user.click(screen.getByRole('button', { name: /sign in/i }));

    await waitFor(() => {
      expect(screen.getByText(/invalid email or password/i)).toBeInTheDocument();
    });
  });

  it('validates email format', async () => {
    const user = userEvent.setup();
    render(<LoginPage />);

    await user.type(screen.getByLabelText(/email/i), 'invalid-email');
    await user.click(screen.getByRole('button', { name: /sign in/i }));

    expect(screen.getByText(/invalid email/i)).toBeInTheDocument();
  });
});
```
```

---

## Prompt 14.5: End-to-End Testing with Playwright

```text
Set up E2E testing with Playwright.

Install Playwright:
```bash
bun create playwright
```

Configure playwright.config.ts:
```typescript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:5173',
    trace: 'on-first-retry',
    video: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
  ],
  webServer: {
    command: 'bun run dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
  },
});
```

Create E2E tests:
```typescript
// e2e/auth.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
  test('user can register and login', async ({ page }) => {
    // Generate unique email for this test
    const email = `test-${Date.now()}@example.com`;

    // Register
    await page.goto('/register');
    await page.fill('input[name="email"]', email);
    await page.fill('input[name="password"]', 'SecurePassword123!');
    await page.fill('input[name="confirmPassword"]', 'SecurePassword123!');
    await page.click('button[type="submit"]');

    // Should redirect to login
    await expect(page).toHaveURL('/login');
    await expect(page.locator('text=Account created')).toBeVisible();

    // Login
    await page.fill('input[name="email"]', email);
    await page.fill('input[name="password"]', 'SecurePassword123!');
    await page.click('button[type="submit"]');

    // Should be on dashboard
    await expect(page).toHaveURL('/dashboard');
    await expect(page.locator('text=Welcome back')).toBeVisible();
  });

  test('user can use magic link', async ({ page, context }) => {
    await page.goto('/magic-link');
    await page.fill('input[name="email"]', 'test@example.com');
    await page.click('button[type="submit"]');

    await expect(page.locator('text=Check your email')).toBeVisible();

    // In real tests, you would intercept the email or use a test token
  });

  test('protected routes redirect to login', async ({ page }) => {
    await page.goto('/dashboard');
    await expect(page).toHaveURL(/\/login/);
  });
});

// e2e/subscription.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Subscription', () => {
  test.beforeEach(async ({ page }) => {
    // Login as test user
    await page.goto('/login');
    await page.fill('input[name="email"]', 'subscriber@example.com');
    await page.fill('input[name="password"]', 'Password123!');
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/dashboard');
  });

  test('can view subscription status', async ({ page }) => {
    await page.goto('/dashboard/subscription');
    await expect(page.locator('text=Active')).toBeVisible();
    await expect(page.locator('text=$3/month')).toBeVisible();
  });

  test('can access applications', async ({ page }) => {
    await page.goto('/dashboard/apps');
    await expect(page.locator('text=RUS')).toBeVisible();
    await expect(page.locator('text=Open Application')).toBeVisible();
  });
});
```

Run E2E tests:
```bash
bunx playwright test
bunx playwright test --ui  # Interactive mode
bunx playwright show-report
```
```

---

## Prompt 14.6: Test Coverage and CI Integration

```text
Configure test coverage and CI pipeline.

Create .github/workflows/test.yml:
```yaml
name: Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
          POSTGRES_DB: test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Run tests
        working-directory: ./api
        env:
          DATABASE_URL: postgres://test:test@localhost:5432/test
        run: |
          cargo test --all-features

      - name: Check formatting
        working-directory: ./api
        run: cargo fmt -- --check

      - name: Run clippy
        working-directory: ./api
        run: cargo clippy -- -D warnings

  frontend-tests:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Bun
        uses: oven-sh/setup-bun@v2

      - name: Install dependencies
        working-directory: ./frontend
        run: bun install --frozen-lockfile

      - name: Run tests
        working-directory: ./frontend
        run: bun test -- --coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./frontend/coverage/lcov.info

  e2e-tests:
    runs-on: ubuntu-latest
    needs: [rust-tests, frontend-tests]

    steps:
      - uses: actions/checkout@v4

      - name: Setup Bun
        uses: oven-sh/setup-bun@v2

      - name: Install dependencies
        working-directory: ./frontend
        run: bun install --frozen-lockfile

      - name: Install Playwright
        working-directory: ./frontend
        run: bunx playwright install --with-deps

      - name: Start services
        run: docker-compose -f docker-compose.test.yml up -d

      - name: Run E2E tests
        working-directory: ./frontend
        run: bunx playwright test

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: playwright-report
          path: frontend/playwright-report/
```

Coverage requirements:
- Overall: 80%+
- Critical paths (auth, payments): 95%+
```

---

## Validation Checklist

After completing all prompts in this section, verify:

- [ ] Unit tests pass: `cargo test`
- [ ] Integration tests pass
- [ ] Frontend tests pass: `bun test`
- [ ] E2E tests pass: `bunx playwright test`
- [ ] Coverage meets targets
- [ ] CI pipeline passes
- [ ] Test database isolated from development
- [ ] Tests run in parallel where possible

---

## Test Commands Summary

```bash
# Rust
cargo test                    # All tests
cargo test --test api         # Integration tests only
cargo test -- --nocapture     # Show output

# Frontend
bun test                      # Unit tests
bun test -- --coverage        # With coverage
bun test -- --watch          # Watch mode

# E2E
bunx playwright test           # All E2E tests
bunx playwright test --ui      # Interactive UI
bunx playwright show-report    # View report

# CI locally
act -j rust-tests            # Run GitHub Actions locally
```

---

## Conclusion

This completes the a8n.tools implementation guide. Follow the documents in order:

1. **00-project-overview.md** - Understand the project
2. **01-project-setup.md** - Set up development environment
3. **02-database-schema.md** - Create database structure
4. **03-authentication.md** - Implement auth system
5. **04-api-core.md** - Build core API
6. **05-stripe-integration.md** - Add payments
7. **06-frontend-foundation.md** - Set up React frontend
8. **07-frontend-auth.md** - Build auth UI
9. **08-frontend-dashboard.md** - Create dashboard
10. **09-admin-panel.md** - Build admin interface
11. **10-email-system.md** - Implement emails
12. **11-infrastructure.md** - Configure deployment
13. **12-monitoring.md** - Add observability
14. **13-security.md** - Harden security
15. **14-testing-strategy.md** - Implement tests

Each document contains self-contained prompts that build on previous work.
