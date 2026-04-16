//! Per-user concurrency (manifest fetches only) + daily pull counter.
//!
//! Mirrors `DownloadLimiter`. In-process concurrency map is single-
//! instance only — Postgres-backed replacement tracked as follow-up.

use chrono::Utc;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::errors::AppError;
use crate::repositories::OciPullDailyCountRepository;

#[derive(Debug, PartialEq)]
pub enum OciLimitDenial {
    Concurrency,
    DailyCap { reset_in_secs: i64 },
}

#[derive(Clone)]
pub struct OciLimiter {
    concurrency_per_user: u32,
    daily_limit: u32,
    inflight: Arc<Mutex<HashMap<Uuid, u32>>>,
}

/// RAII guard that releases a concurrency slot on drop.
#[derive(Debug)]
pub struct OciPullGuard {
    user_id: Uuid,
    inflight: Arc<Mutex<HashMap<Uuid, u32>>>,
}

impl Drop for OciPullGuard {
    fn drop(&mut self) {
        let mut m = self.inflight.lock().unwrap();
        if let Some(n) = m.get_mut(&self.user_id) {
            *n = n.saturating_sub(1);
            if *n == 0 {
                m.remove(&self.user_id);
            }
        }
    }
}

impl OciLimiter {
    pub fn new(concurrency_per_user: u32, daily_limit: u32) -> Self {
        Self {
            concurrency_per_user,
            daily_limit,
            inflight: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Try to acquire a manifest-pull slot. Returns a guard on success.
    pub async fn acquire(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Result<OciPullGuard, OciLimitDenial>, AppError> {
        {
            let mut m = self.inflight.lock().unwrap();
            let entry = m.entry(user_id).or_insert(0);
            if *entry >= self.concurrency_per_user {
                return Ok(Err(OciLimitDenial::Concurrency));
            }
            *entry += 1;
        }

        let today = Utc::now().date_naive();
        let count = match OciPullDailyCountRepository::increment(pool, user_id, today).await {
            Ok(n) => n,
            Err(e) => {
                let mut m = self.inflight.lock().unwrap();
                if let Some(n) = m.get_mut(&user_id) {
                    *n = n.saturating_sub(1);
                    if *n == 0 { m.remove(&user_id); }
                }
                return Err(e);
            }
        };

        if (count as u32) > self.daily_limit {
            OciPullDailyCountRepository::decrement(pool, user_id, today).await.ok();
            let mut m = self.inflight.lock().unwrap();
            if let Some(n) = m.get_mut(&user_id) {
                *n = n.saturating_sub(1);
                if *n == 0 { m.remove(&user_id); }
            }
            let reset_in_secs = seconds_until_utc_midnight();
            return Ok(Err(OciLimitDenial::DailyCap { reset_in_secs }));
        }

        Ok(Ok(OciPullGuard {
            user_id,
            inflight: self.inflight.clone(),
        }))
    }
}

fn seconds_until_utc_midnight() -> i64 {
    let now = Utc::now();
    let tomorrow = (now + chrono::Duration::days(1)).date_naive();
    let midnight = tomorrow.and_hms_opt(0, 0, 0).unwrap().and_utc();
    (midnight - now).num_seconds().max(0)
}

#[cfg(test)]
mod tests {
    //! DB-backed. Skipped when DATABASE_URL is unset.
    use super::*;

    async fn maybe_pool() -> Option<PgPool> {
        let url = std::env::var("DATABASE_URL").ok()?;
        PgPool::connect(&url).await.ok()
    }

    async fn create_test_user(pool: &PgPool) -> Option<Uuid> {
        let user = Uuid::new_v4();
        let email = format!("oci-limiter-test-{}@example.com", user);
        let res = sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'x')")
            .bind(user)
            .bind(&email)
            .execute(pool)
            .await;
        if res.is_err() { return None; }
        Some(user)
    }

    async fn cleanup_user(pool: &PgPool, user: Uuid) {
        // ON DELETE CASCADE should drop pull-count rows.
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user).execute(pool).await.ok();
    }

    #[actix_rt::test]
    async fn guard_releases_slot_on_drop() {
        let Some(pool) = maybe_pool().await else { return; };
        let Some(user) = create_test_user(&pool).await else { return; };

        let limiter = OciLimiter::new(1, 50);
        {
            let guard = limiter.acquire(&pool, user).await.unwrap().unwrap();
            match limiter.acquire(&pool, user).await.unwrap() {
                Err(OciLimitDenial::Concurrency) => {}
                other => panic!("expected Concurrency denial, got {other:?}"),
            }
            drop(guard);
        }
        let _ = limiter.acquire(&pool, user).await.unwrap().unwrap();

        cleanup_user(&pool, user).await;
    }

    #[actix_rt::test]
    async fn daily_cap_denies_over_limit() {
        let Some(pool) = maybe_pool().await else { return; };
        let Some(user) = create_test_user(&pool).await else { return; };

        let limiter = OciLimiter::new(5, 2);
        let g1 = limiter.acquire(&pool, user).await.unwrap().unwrap();
        drop(g1);
        let g2 = limiter.acquire(&pool, user).await.unwrap().unwrap();
        drop(g2);

        match limiter.acquire(&pool, user).await.unwrap() {
            Err(OciLimitDenial::DailyCap { reset_in_secs }) => {
                assert!(reset_in_secs > 0);
                assert!(reset_in_secs <= 86_400);
            }
            other => panic!("expected DailyCap denial, got {other:?}"),
        }

        let today = Utc::now().date_naive();
        let cur = OciPullDailyCountRepository::current(&pool, user, today).await.unwrap();
        assert_eq!(cur, 2);

        cleanup_user(&pool, user).await;
    }
}
