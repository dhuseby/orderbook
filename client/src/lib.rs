use iso8601_timestamp::Timestamp;
use oberon::{Proof, Token};
use once_cell::sync::OnceCell;
use orderbook::{
    orderbook_aggregator_client::OrderbookAggregatorClient, Level, OberonProof, Summary, SummaryReq,
};
use rand::prelude::*;
use serde::{
    de::{Error as DError, SeqAccess, Unexpected, Visitor},
    Deserialize, Deserializer,
};
use std::{
    fmt::{self, Formatter},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
};
use tokio::runtime::Builder;
use tokio::sync::{mpsc, Mutex};
use tonic::Streaming;
use uuid::Uuid;

#[derive(Clone, Debug)]
enum Task {
    /// Initialize task
    Initialize { id: u64, endpoint: String },
    /// Shutdown task
    Shutdown { id: u64 },
    /// Get the order book summary
    Summary { id: u64, symbol: String },
}

#[derive(Clone, Debug)]
pub enum OrderbookResponse {
    /// Initialized response
    Initialized,
    /// Shutdown response
    Shutdown,
    /// Order book summary response
    OrderbookSummary { summary: Summary },
    /// Failed
    Failed { reason: String },
}

#[derive(Debug)]
enum TaskResponse {
    Initialized,
    Shutdown,
    SummaryStream { stream: Streaming<Summary> },
}

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    /// not implemented
    #[error("task not implemented yet")]
    NotImplemented,

    /// failed to deserialize oberon token from bytes
    #[error("failed to deserialize oberon token")]
    OberonTokenDeserializationError,

    /// Timestamp error
    #[error("timestamp error")]
    TimestampError(#[from] std::time::SystemTimeError),

    /// Slice conversion error
    #[error("byte slice invalid length")]
    ByteSliceError(#[from] std::array::TryFromSliceError),

    /// oberon::Proof generation error
    #[error("failed to generate oberon proof")]
    ProofGenerationError,

    /// tonic status error
    #[error("tonic status error")]
    TonicStatusError(#[from] tonic::Status),

    /// tonic transport error
    #[error("tonic transport error")]
    TonicTransportError(#[from] tonic::transport::Error),

    /// no client connection error
    #[error("no client connection available")]
    NoConnection,

    /// summary error
    #[error("getting summary failed: {0}")]
    SummaryFailure(String),
}

impl Task {
    async fn get_id(&self) -> u64 {
        match self {
            Task::Initialize { id, .. } | Task::Shutdown { id } | Task::Summary { id, .. } => *id,
        }
    }

    async fn execute(&self) -> Result<TaskResponse, TaskError> {
        match self {
            Task::Initialize { ref endpoint, .. } => {
                // get the client
                let mut client = global_client().lock().await;

                // try to connect to the server
                if let Ok(grpc) = OrderbookAggregatorClient::connect(endpoint.clone()).await {
                    client.grpc = Some(grpc);
                    Ok(TaskResponse::Initialized)
                } else {
                    Err(TaskError::NoConnection)
                }
            }
            Task::Shutdown { .. } => {
                // Do shutdown stuff here
                Ok(TaskResponse::Shutdown)
            }
            Task::Summary { ref symbol, .. } => {
                let mut client = global_client().lock().await;
                let proof = client.token.get_proof().await?;

                // make the request for the orderbook summary stream
                let req = tonic::Request::new(SummaryReq {
                    proof: Some(proof),
                    symbol: symbol.to_string(),
                });

                // get the grpc client
                let grpc = client.grpc.as_mut().ok_or(TaskError::NoConnection)?;

                // send the request
                let summary: Streaming<Summary> = grpc.book_summary(req).await?.into_inner();

                Ok(TaskResponse::SummaryStream { stream: summary })
            }
        }
    }
}

async fn do_task(task: Task, chan: mpsc::Sender<(Task, Result<TaskResponse, TaskError>)>) {
    // execute the task
    let resp = task.execute().await;

    // send the task and the response to the callback task
    let _ = chan.send((task, resp)).await;
}

async fn do_resp(task: Task, resp: Result<TaskResponse, TaskError>) {
    let id = task.get_id().await;
    match resp {
        Err(e) => {
            // NOTE: hold the lock for as little time as possible
            let client = global_client().lock().await;
            // send the error to the completed callback
            client.callbacks.completed(
                id,
                OrderbookResponse::Failed {
                    reason: format!("{}", e),
                },
            );
        }
        Ok(r) => match r {
            TaskResponse::Initialized => {
                // NOTE: hold the lock for as little time as possible
                let client = global_client().lock().await;
                client.callbacks.initialized(id, Arc::new(Orderbook::new()));
            }
            TaskResponse::Shutdown => {
                // NOTE: hold the lock for as little time as possible
                let client = global_client().lock().await;
                client.callbacks.completed(id, OrderbookResponse::Shutdown);
            }

            // NOTE: it is OK to loop here because a new task is spawned for each call to do_resp.
            // The only real concern is that we have to minimize how much time we hold the global
            // client mutex lock. That's why we get the lock only when we need to make a callback
            // and drop it immediately afterwards
            TaskResponse::SummaryStream { mut stream } => loop {
                match stream.message().await {
                    Ok(msg) => {
                        if let Some(summary) = msg {
                            // 🔒
                            let client = global_client().lock().await;

                            // send the next summary update
                            client
                                .callbacks
                                .completed(id, OrderbookResponse::OrderbookSummary { summary });
                            // 🔓
                        } else {
                            // 🔒
                            let client = global_client().lock().await;

                            // let the client know that the stream is ending and why
                            client.callbacks.completed(
                                id,
                                OrderbookResponse::Failed {
                                    reason: "The stream was closed by the server".to_string(),
                                },
                            );
                            // 🔓
                        }
                    }
                    Err(status) => {
                        // 🔒
                        let client = global_client().lock().await;

                        // let the client know the stream is ending and why
                        client.callbacks.completed(
                            id,
                            OrderbookResponse::Failed {
                                reason: format!("gRPC: {}", status),
                            },
                        );
                        // 🔓
                    }
                }
            },
        },
    }
}

#[derive(Clone, Debug, Deserialize)]
struct OberonToken {
    #[serde(deserialize_with = "deserialize_uuid")]
    id: Uuid,
    #[serde(with = "hex::serde")]
    token: Vec<u8>,
}

impl OberonToken {
    async fn get_proof(&self) -> Result<OberonProof, TaskError> {
        let mut rng = thread_rng();

        // Build the OberonProof
        let t = Token::from_bytes(&self.token[..48].try_into()?);
        if t.is_none().unwrap_u8() != 0 {
            return Err(TaskError::OberonTokenDeserializationError);
        }

        let timestamp = Timestamp::now_utc()
            .duration_since(Timestamp::UNIX_EPOCH)
            .whole_milliseconds() as u64;

        let id = &self.id.as_bytes();
        let nonce = &timestamp.to_be_bytes();
        let token = t.unwrap();

        let proof =
            Proof::new(&token, &[], &id, nonce, &mut rng).ok_or(TaskError::ProofGenerationError)?;

        Ok(OberonProof {
            id: self.id.to_string(),
            timestamp,
            proof: proof.to_bytes().to_vec(),
        })
    }
}

/// State on the Rust side of things
struct Client {
    task_send: mpsc::Sender<Task>,
    callbacks: Box<dyn OnOrderbookClientEvents>,
    token: OberonToken,
    grpc: Option<OrderbookAggregatorClient<tonic::transport::Channel>>,
}

impl Client {
    fn new(token: OberonToken, callbacks: Box<dyn OnOrderbookClientEvents>) -> Self {
        // set up the channel for communicating
        let (task_send, mut task_recv) = mpsc::channel::<Task>(16);
        let (resp_send, mut resp_recv) =
            mpsc::channel::<(Task, Result<TaskResponse, TaskError>)>(16);

        // build the runtime
        let rt = Builder::new_multi_thread()
            .worker_threads(64)
            .enable_all()
            .build()
            .unwrap();

        // launch a background thread for the async runtime
        std::thread::spawn(move || {
            let mut handles = Vec::with_capacity(2);

            // spawn task to handle jobs
            handles.push(rt.spawn(async move {
                while let Some(task) = task_recv.recv().await {
                    tokio::spawn(do_task(task, resp_send.clone()));
                }
            }));

            // spawn task to handle callbacks
            handles.push(rt.spawn(async move {
                while let Some((task, resp)) = resp_recv.recv().await {
                    tokio::spawn(do_resp(task, resp));
                }
            }));

            // block on the handles
            for handle in handles {
                rt.block_on(handle).unwrap();
            }

            println!("client thread done");
        });

        Self {
            task_send,
            callbacks,
            token,
            grpc: None,
        }
    }

    fn spawn_task(&self, task: Task) {
        match self.task_send.blocking_send(task) {
            Ok(()) => {}
            Err(_) => panic!("shared runtime shut down"),
        }
    }
}

// so we know if initialize has been called
static INITIALIZE_CALLED: AtomicBool = AtomicBool::new(false);

// convenience function for testing if initialize was called
fn was_initialize_called() -> bool {
    INITIALIZE_CALLED.load(Ordering::SeqCst)
}

// atomic counter for matching up responses with requests
static TASK_COUNTER: AtomicU64 = AtomicU64::new(1);

// convenience function to get the next number
fn get_next_task_id() -> u64 {
    TASK_COUNTER.fetch_add(1, Ordering::SeqCst)
}

// singleton of the client actor
static CLIENT: OnceCell<Mutex<Client>> = OnceCell::new();

// convenience function to get a reference to the client Mutex
fn global_client() -> &'static Mutex<Client> {
    CLIENT.get().unwrap()
}

// set up the client singleton
fn setup_client(client: Client) {
    if CLIENT.get().is_none() {
        if CLIENT.set(Mutex::new(client)).is_err() {
            println!("client already initialized");
        }
    } else {
        let mut lock = CLIENT.get().unwrap().blocking_lock();
        *lock = client;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OrderbookError {
    /// Called initialize when already initialized
    #[error("initialize already called")]
    InitializeAlreadyCalled,

    /// Called shutdown when not initialized
    #[error("initialize not called yet")]
    NotInitialized,

    /// Failed to deserialized the oberon token
    #[error("failed to deserialize token")]
    TokenError(#[from] serde_json::Error),
}

pub trait OnOrderbookClientEvents: Send {
    /// Called after initialization is complete
    fn initialized(&self, id: u64, orderbook: Arc<Orderbook>);

    /// Called when summary is received
    fn summary(&self, id: u64, summary: Summary);

    /// called when an operation completes
    fn completed(&self, id: u64, resp: OrderbookResponse);
}

pub fn orderbook_client_initialize(
    token: String,
    endpoint: String,
    callbacks: Box<dyn OnOrderbookClientEvents>,
) -> Result<u64, OrderbookError> {
    if was_initialize_called() {
        return Err(OrderbookError::InitializeAlreadyCalled);
    }

    // set up the client singleton
    setup_client(Client::new(serde_json::from_str(&token)?, callbacks));

    // mark initialization called
    INITIALIZE_CALLED.store(true, Ordering::SeqCst);

    // get the next task id
    let id = get_next_task_id();

    // run the initialize task
    let client = global_client().blocking_lock();
    client.spawn_task(Task::Initialize { id, endpoint });

    Ok(id)
}

pub fn orderbook_client_shutdown() -> Result<u64, OrderbookError> {
    if !was_initialize_called() {
        return Err(OrderbookError::NotInitialized);
    }

    // reset so we can be initialized again
    INITIALIZE_CALLED.store(false, Ordering::SeqCst);

    // get the next task id
    let id = get_next_task_id();

    // run the shutdown task
    let client = global_client().blocking_lock();
    client.spawn_task(Task::Shutdown { id });

    Ok(id)
}

pub struct Orderbook {}

impl Orderbook {
    pub fn new() -> Self {
        Self {}
    }

    pub fn summary(&self, symbol: String) -> Result<u64, OrderbookError> {
        // get the next task id
        let id = get_next_task_id();

        // do the summary task
        match self.do_call(Task::Summary { id, symbol }) {
            Ok(()) => Ok(id),
            Err(e) => Err(e),
        }
    }

    fn do_call(&self, task: Task) -> Result<(), OrderbookError> {
        if !was_initialize_called() {
            return Err(OrderbookError::NotInitialized);
        }
        let client = global_client().blocking_lock();
        client.spawn_task(task);
        Ok(())
    }
}

/// Method for deserializing Uuids
pub fn deserialize_uuid<'de, D>(d: D) -> std::result::Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    struct UuidVisitor;

    impl<'de> Visitor<'de> for UuidVisitor {
        type Value = Uuid;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            write!(formatter, "a string")
        }

        fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
        where
            E: DError,
        {
            match Uuid::parse_str(v) {
                Err(_) => Err(DError::invalid_value(Unexpected::Str(v), &self)),
                Ok(i) => Ok(i),
            }
        }

        fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut bytes = [0u8; 16];
            let mut i = 0;
            while let Some(b) = seq.next_element()? {
                bytes[i] = b;
                i += 1;
            }
            Ok(Uuid::from_bytes(bytes))
        }
    }

    if d.is_human_readable() {
        d.deserialize_str(UuidVisitor)
    } else {
        d.deserialize_tuple(16, UuidVisitor)
    }
}

#[allow(missing_docs)]
mod ffi {
    use super::*;
    include!("OrderbookClient.uniffi.rs");
}
pub use ffi::*;

#[allow(missing_docs)]
mod orderbook {
    include!("orderbook.rs");
}
