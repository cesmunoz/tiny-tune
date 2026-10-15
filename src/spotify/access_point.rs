use std::{future::Future, time::Duration};

use tracing::warn;

use crate::domain::AppError;

/// Bounds one access-point attempt. A blackholed TCP port must not wait forever.
pub(crate) const PLAYBACK_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) enum AttemptFailure {
    Retryable,
    Terminal(AppError),
}

pub(crate) enum ConnectError {
    Exhausted,
    Terminal(AppError),
}

/// Runs each access-point attempt inside `timeout`.
///
/// A retryable failure or a timeout continues while attempts remain.
/// A terminal failure stops immediately.
pub(crate) async fn connect_with_fallback<T, F, Fut>(
    attempts: &[Option<u16>],
    timeout: Duration,
    mut connect: F,
) -> Result<T, ConnectError>
where
    F: FnMut(Option<u16>) -> Fut,
    Fut: Future<Output = Result<T, AttemptFailure>>,
{
    let Some((last, earlier)) = attempts.split_last() else {
        return Err(ConnectError::Exhausted);
    };

    for port in earlier.iter().copied() {
        match tokio::time::timeout(timeout, connect(port)).await {
            Ok(Ok(value)) => return Ok(value),
            Ok(Err(AttemptFailure::Terminal(error))) => {
                return Err(ConnectError::Terminal(error));
            }
            Ok(Err(AttemptFailure::Retryable)) => warn_attempt(port, "failed"),
            Err(_) => warn_attempt(port, "timed out"),
        }
    }

    match tokio::time::timeout(timeout, connect(*last)).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(AttemptFailure::Terminal(error))) => Err(ConnectError::Terminal(error)),
        Ok(Err(AttemptFailure::Retryable)) => {
            warn_attempt(*last, "failed");
            Err(ConnectError::Exhausted)
        }
        Err(_) => {
            warn_attempt(*last, "timed out");
            Err(ConnectError::Exhausted)
        }
    }
}

fn warn_attempt(port: Option<u16>, reason: &'static str) {
    if let Some(port) = port {
        warn!(port, reason, "spotify access point attempt did not connect");
    } else {
        warn!(
            port = "default",
            reason, "spotify access point attempt did not connect"
        );
    }
}

#[cfg(test)]
#[path = "__tests__/access_point.rs"]
mod tests;
