use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use obscura::Browser;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};
use url::Url;

use crate::error::Error;

const TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

struct ClientInner {
    base_url: Url,
    connection: tokio::sync::Mutex<Connection>,
}

struct Connection {
    requests: Option<mpsc::Sender<Request>>,
    handle: Option<thread::JoinHandle<()>>,
}

struct Worker {
    browser: Browser,
    requests: mpsc::Receiver<Request>,
}

struct Request {
    url: Url,
    reply: oneshot::Sender<Result<Value, Error>>,
}

impl Client {
    pub fn new() -> anyhow::Result<Self> {
        Self::with_base_url(Url::parse("https://api.faceit.com/")?)
    }

    pub fn with_base_url(base_url: Url) -> anyhow::Result<Self> {
        let connection = Connection::spawn()?;
        Ok(Self {
            inner: Arc::new(ClientInner {
                base_url,
                connection: tokio::sync::Mutex::new(connection),
            }),
        })
    }

    pub async fn fetch<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, Error> {
        let url = self.build_url(path, query)?;
        let value = self.request(url).await?;
        deserialize(value)
    }

    fn build_url(&self, path: &str, query: &[(&str, String)]) -> Result<Url, Error> {
        let mut url = self.inner.base_url.join(path).map_err(|_| unavailable())?;
        if url.origin() != self.inner.base_url.origin() {
            return Err(unavailable());
        }
        if !query.is_empty() {
            url.query_pairs_mut()
                .extend_pairs(query.iter().map(|(key, value)| (*key, value)));
        }
        Ok(url)
    }

    async fn request(&self, url: Url) -> Result<Value, Error> {
        let (reply, response) = oneshot::channel();
        tokio::time::timeout(TIMEOUT, async {
            self.inner
                .send(Request { url, reply }, Connection::spawn)
                .await?;
            response.await.map_err(|_| unavailable())?
        })
        .await
        .map_err(|_| timed_out())?
    }
}

impl ClientInner {
    async fn send(
        &self,
        request: Request,
        spawn: impl FnOnce() -> anyhow::Result<Connection>,
    ) -> Result<(), Error> {
        let requests = self
            .connection
            .lock()
            .await
            .requests
            .as_ref()
            .unwrap()
            .clone();
        let request = match requests.send(request).await {
            Ok(()) => return Ok(()),
            Err(error) => error.0,
        };
        let replacement = self.replace(&requests, spawn).await?;
        replacement.send(request).await.map_err(|_| unavailable())
    }

    async fn replace(
        &self,
        requests: &mpsc::Sender<Request>,
        spawn: impl FnOnce() -> anyhow::Result<Connection>,
    ) -> Result<mpsc::Sender<Request>, Error> {
        let mut connection = self.connection.lock().await;
        if connection.requests.as_ref().unwrap().same_channel(requests) {
            tracing::warn!("Restarting FACEIT browser worker");
            *connection = spawn().map_err(|error| {
                tracing::error!(%error, "Failed to restart FACEIT browser worker");
                unavailable()
            })?;
        }
        Ok(connection.requests.as_ref().unwrap().clone())
    }
}

impl Connection {
    fn spawn() -> anyhow::Result<Self> {
        let (requests, receiver) = mpsc::channel(16);
        let handle = Worker::spawn(receiver)?;
        Ok(Self {
            requests: Some(requests),
            handle: Some(handle),
        })
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.requests.take();
        if let Some(handle) = self.handle.take()
            && handle.join().is_err()
        {
            tracing::error!("FACEIT browser worker panicked");
        }
    }
}

impl Worker {
    fn spawn(requests: mpsc::Receiver<Request>) -> anyhow::Result<thread::JoinHandle<()>> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let browser = {
            let _guard = runtime.enter();
            Browser::builder().stealth(true).build()?
        };
        let worker = Self { browser, requests };
        Ok(thread::Builder::new()
            .name("faceit-browser".into())
            .spawn(move || runtime.block_on(worker.run()))?)
    }

    async fn run(mut self) {
        while let Some(Request { url, mut reply }) = self.requests.recv().await {
            let result = tokio::select! {
                biased;
                _ = reply.closed() => continue,
                result = tokio::time::timeout(TIMEOUT, self.fetch(url)) => {
                    result.unwrap_or_else(|_| Err(timed_out()))
                }
            };
            let _ = reply.send(result);
        }
    }

    async fn fetch(&self, url: Url) -> Result<Value, Error> {
        let mut page = self.browser.new_page().await.map_err(|_| unavailable())?;
        let response = Arc::new(Mutex::new(None));
        let captured = response.clone();
        let target = url.clone();
        page.on_response(Arc::new(move |_, response| {
            if response.url == target {
                *captured.lock().unwrap() = Some((response.status, response.body.clone()));
            }
        }));
        page.goto(url.as_str()).await.map_err(|_| unavailable())?;
        page.settle(3000).await;
        let (status, body) = response.lock().unwrap().take().ok_or_else(unavailable)?;
        match status {
            200 => serde_json::from_slice(&body)
                .map_err(|_| Error::bad_gateway("Invalid FACEIT response")),
            404 => Err(Error::not_found("FACEIT resource not found")),
            429 => Err(Error::service_unavailable("FACEIT rate limit exceeded")),
            _ => Err(unavailable()),
        }
    }
}

fn deserialize<T: DeserializeOwned>(mut value: Value) -> Result<T, Error> {
    if let Some(payload) = value.get_mut("payload") {
        value = payload.take();
    }
    serde_json::from_value(value).map_err(|_| Error::bad_gateway("Invalid FACEIT response"))
}

fn unavailable() -> Error {
    Error::bad_gateway("FACEIT unavailable")
}

fn timed_out() -> Error {
    Error::gateway_timeout("FACEIT request timed out")
}

#[cfg(test)]
mod tests {
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
}
