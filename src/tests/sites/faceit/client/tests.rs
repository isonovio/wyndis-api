use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use super::*;

#[tokio::test]
async fn closed_worker_is_replaced_once_for_concurrent_requests() {
    let (requests, receiver) = mpsc::channel(1);
    drop(receiver);
    let inner = ClientInner {
        base_url: Url::parse("https://api.faceit.com/").unwrap(),
        connection: tokio::sync::Mutex::new(Connection {
            requests: Some(requests),
            handle: None,
        }),
    };
    let restarts = AtomicUsize::new(0);
    let spawn = || {
        restarts.fetch_add(1, Ordering::SeqCst);
        let (requests, mut receiver) = mpsc::channel::<Request>(16);
        let handle = thread::spawn(move || {
            while let Some(request) = receiver.blocking_recv() {
                let _ = request.reply.send(Ok(serde_json::json!(42)));
            }
        });
        Ok(Connection {
            requests: Some(requests),
            handle: Some(handle),
        })
    };
    let (reply1, response1) = oneshot::channel();
    let (reply2, response2) = oneshot::channel();
    let (first, second) = tokio::join!(
        inner.send(
            Request {
                url: inner.base_url.clone(),
                reply: reply1
            },
            spawn
        ),
        inner.send(
            Request {
                url: inner.base_url.clone(),
                reply: reply2
            },
            spawn
        ),
    );
    first.unwrap();
    second.unwrap();
    assert_eq!(response1.await.unwrap().unwrap(), serde_json::json!(42));
    assert_eq!(response2.await.unwrap().unwrap(), serde_json::json!(42));
    assert_eq!(restarts.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn failed_restart_returns_an_error_and_allows_another_attempt() {
    let (requests, receiver) = mpsc::channel(1);
    drop(receiver);
    let inner = ClientInner {
        base_url: Url::parse("https://api.faceit.com/").unwrap(),
        connection: tokio::sync::Mutex::new(Connection {
            requests: Some(requests),
            handle: None,
        }),
    };
    let attempts = AtomicUsize::new(0);
    for _ in 0..2 {
        let (reply, _) = oneshot::channel();
        let result = inner
            .send(
                Request {
                    url: inner.base_url.clone(),
                    reply,
                },
                || {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    anyhow::bail!("Worker startup failed")
                },
            )
            .await;
        assert!(result.is_err());
    }
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
}

#[test]
fn last_client_clone_closes_and_joins_the_worker() {
    let (requests, mut receiver) = mpsc::channel(1);
    let stopped = Arc::new(AtomicBool::new(false));
    let worker_stopped = stopped.clone();
    let handle = thread::spawn(move || {
        while receiver.blocking_recv().is_some() {}
        worker_stopped.store(true, Ordering::SeqCst);
    });
    let client = Client {
        inner: Arc::new(ClientInner {
            base_url: Url::parse("https://api.faceit.com/").unwrap(),
            connection: tokio::sync::Mutex::new(Connection {
                requests: Some(requests),
                handle: Some(handle),
            }),
        }),
    };
    let clone = client.clone();
    drop(client);
    assert!(!stopped.load(Ordering::SeqCst));
    drop(clone);
    assert!(stopped.load(Ordering::SeqCst));
}

#[test]
fn fetch_shapes_accept_envelopes_and_plain_arrays() {
    assert_eq!(
        deserialize::<u32>(serde_json::json!({"payload": 1993})).unwrap(),
        1993
    );
    assert_eq!(
        deserialize::<Vec<u32>>(serde_json::json!([1, 2])).unwrap(),
        [1, 2]
    );
    assert!(deserialize::<u32>(serde_json::json!({"payload": null})).is_err());
    assert!(deserialize::<u32>(serde_json::json!({"code": "ERROR"})).is_err());
}
