use std::{
    future::{self, Future},
    pin::Pin,
    time::Duration,
};

use super::{AttemptFailure, ConnectError, connect_with_fallback};
use crate::domain::AppError;

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("test runtime")
}

fn block<T>(future: impl Future<Output = T>) -> T {
    runtime().block_on(future)
}

type ConnectFuture = Pin<Box<dyn Future<Output = Result<&'static str, AttemptFailure>>>>;

#[test]
fn uses_the_default_port_when_it_connects() {
    let mut ports = Vec::new();
    let result = block(connect_with_fallback(
        &[None, Some(443)],
        Duration::from_millis(50),
        |port| {
            ports.push(port);
            Box::pin(async { Ok("ready") }) as ConnectFuture
        },
    ));

    assert_eq!(result.ok(), Some("ready"));
    assert_eq!(ports, [None]);
}

#[test]
fn falls_back_to_443_when_the_default_port_times_out() {
    let mut ports = Vec::new();
    let result = block(connect_with_fallback(
        &[None, Some(443)],
        Duration::from_millis(30),
        |port| {
            ports.push(port);
            Box::pin(async move {
                if port.is_none() {
                    future::pending().await
                } else {
                    Ok("fallback")
                }
            }) as ConnectFuture
        },
    ));

    assert_eq!(result.ok(), Some("fallback"));
    assert_eq!(ports, [None, Some(443)]);
}

#[test]
fn falls_back_immediately_when_the_default_port_fails() {
    let mut ports = Vec::new();
    let result = block(connect_with_fallback(
        &[None, Some(443)],
        Duration::from_secs(30),
        |port| {
            ports.push(port);
            Box::pin(async move {
                if port.is_none() {
                    Err(AttemptFailure::Retryable)
                } else {
                    Ok("fallback")
                }
            }) as ConnectFuture
        },
    ));

    assert_eq!(result.ok(), Some("fallback"));
    assert_eq!(ports, [None, Some(443)]);
}

#[test]
fn reports_exhausted_when_every_attempt_times_out() {
    let mut ports = Vec::new();
    let result: Result<(), ConnectError> = block(connect_with_fallback(
        &[None, Some(443)],
        Duration::from_millis(20),
        |port| {
            ports.push(port);
            Box::pin(future::pending())
        },
    ));

    assert!(matches!(result, Err(ConnectError::Exhausted)));
    assert_eq!(ports, [None, Some(443)]);
}

#[test]
fn stops_on_a_terminal_failure_without_trying_the_fallback_port() {
    let mut ports = Vec::new();
    let result: Result<(), ConnectError> = block(connect_with_fallback(
        &[None, Some(443)],
        Duration::from_millis(50),
        |port| {
            ports.push(port);
            Box::pin(async { Err(AttemptFailure::Terminal(AppError::ReauthenticationRequired)) })
        },
    ));

    assert!(matches!(
        result,
        Err(ConnectError::Terminal(AppError::ReauthenticationRequired))
    ));
    assert_eq!(ports, [None]);
}

#[test]
fn fixed_port_does_not_fall_back_after_a_timeout() {
    let mut ports = Vec::new();
    let result: Result<(), ConnectError> = block(connect_with_fallback(
        &[Some(4070)],
        Duration::from_millis(20),
        |port| {
            ports.push(port);
            Box::pin(future::pending())
        },
    ));

    assert!(matches!(result, Err(ConnectError::Exhausted)));
    assert_eq!(ports, [Some(4070)]);
}

#[test]
fn playback_connect_timeout_stays_bounded() {
    let timeout = super::PLAYBACK_CONNECT_TIMEOUT;
    assert!(timeout >= Duration::from_secs(5));
    assert!(timeout <= Duration::from_secs(15));
}
