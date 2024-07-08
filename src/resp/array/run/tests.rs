use crate::resp::array::Array;
use crate::resp::simple_string::SimpleString;
use crate::resp::tests::assert_run;
use crate::resp::Resp;

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
