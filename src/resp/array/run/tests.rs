use std::sync::Arc;

use crate::resp::array::Array;
use crate::resp::simple_string::SimpleString;
use crate::resp::tests::{assert_run, assert_run_with_storage};
use crate::resp::{BulkString, Integer, Resp};

use super::*;

#[tokio::test]
async fn test_run_ping() -> Result<()> {
    assert_run(
        Resp::Array(Array(vec![Resp::SimpleString(SimpleString(
            "PING".to_string(),
        ))])),
        Resp::SimpleString(SimpleString("PONG".to_string())),
    )
    .await
}

#[tokio::test]
async fn test_run_ping_with_message() -> Result<()> {
    assert_run(
        Resp::Array(Array(vec![
            Resp::SimpleString(SimpleString("PING".to_string())),
            Resp::SimpleString(SimpleString("hello".to_string())),
        ])),
        Resp::SimpleString(SimpleString("hello".to_string())),
    )
    .await
}

#[tokio::test]
async fn test_run_too_many_arguments() {
    assert_run(
        Resp::Array(Array(vec![
            Resp::SimpleString(SimpleString("PING".to_string())),
            Resp::SimpleString(SimpleString("hello".to_string())),
            Resp::SimpleString(SimpleString("world".to_string())),
        ])),
        Resp::SimpleString(SimpleString("PONG".to_string())),
    )
    .await
    .unwrap_err();
}

#[tokio::test]
async fn test_run_echo() -> Result<()> {
    assert_run(
        Resp::Array(Array(vec![
            Resp::SimpleString(SimpleString("ECHO".to_string())),
            Resp::SimpleString(SimpleString("hello".to_string())),
        ])),
        Resp::SimpleString(SimpleString("hello".to_string())),
    )
    .await
}

#[tokio::test]
async fn test_get_no_key() -> Result<()> {
    assert_run_with_storage(
        Resp::Array(Array(vec![
            Resp::SimpleString(SimpleString("GET".to_string())),
            Resp::SimpleString(SimpleString("key".to_string())),
        ])),
        Resp::BulkString(BulkString(None)),
        Default::default(),
    )
    .await
}

#[tokio::test]
async fn test_get_key() -> Result<()> {
    let storage = Arc::new(RwLock::new(Storage::new()));

    assert_run_with_storage(
        Resp::Array(Array(vec![
            Resp::SimpleString(SimpleString("SET".to_string())),
            Resp::SimpleString(SimpleString("key".to_string())),
            Resp::BulkString(BulkString(Some("value".to_string()))),
        ])),
        Resp::SimpleString(SimpleString("OK".to_string())),
        Arc::clone(&storage),
    )
    .await?;

    assert_run_with_storage(
        Resp::Array(Array(vec![
            Resp::SimpleString(SimpleString("GET".to_string())),
            Resp::SimpleString(SimpleString("key".to_string())),
        ])),
        Resp::BulkString(BulkString(Some("value".to_string()))),
        storage,
    )
    .await
}

#[tokio::test]
async fn test_expiry() -> Result<()> {
    let storage = Arc::new(RwLock::new(Storage::new()));

    assert_run_with_storage(
        Resp::Array(Array(vec![
            Resp::SimpleString(SimpleString("SET".to_string())),
            Resp::SimpleString(SimpleString("key".to_string())),
            Resp::BulkString(BulkString(Some("value".to_string()))),
            Resp::SimpleString(SimpleString("PX".to_string())),
            Resp::Integer(Integer(100)),
        ])),
        Resp::SimpleString(SimpleString("OK".to_string())),
        Arc::clone(&storage),
    )
    .await?;

    assert_run_with_storage(
        Resp::Array(Array(vec![
            Resp::SimpleString(SimpleString("GET".to_string())),
            Resp::SimpleString(SimpleString("key".to_string())),
        ])),
        Resp::BulkString(BulkString(Some("value".to_string()))),
        Arc::clone(&storage),
    )
    .await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    assert_run_with_storage(
        Resp::Array(Array(vec![
            Resp::SimpleString(SimpleString("GET".to_string())),
            Resp::SimpleString(SimpleString("key".to_string())),
        ])),
        Resp::BulkString(BulkString(None)),
        storage,
    )
    .await
}

#[tokio::test]
async fn test_default_info() {
    assert_run(
        Resp::Array(Array(vec![
            Resp::SimpleString(SimpleString("INFO".to_string())),
            Resp::SimpleString(SimpleString("replication".to_string())),
        ])),
        Resp::BulkString(BulkString(Some(Config::default().to_vec().join("\n")))),
    )
    .await
    .unwrap();
}
