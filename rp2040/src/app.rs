//! A basic postcard-rpc/poststation-compatible application

use crate::{handlers::{get_led, set_led, sleep_handler, unique_id}, impls::{RttRx, RttTx}};
use embassy_stm32::gpio::Output;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_executor::{SpawnError, SpawnToken, Spawner};

use postcard_rpc::{
    define_dispatch,
    server::{Server, SpawnContext, WireSpawn},
};
use static_cell::ConstStaticCell;
use template_icd::{
    GetLedEndpoint, GetUniqueIdEndpoint, RebootToPicoBoot, SetLedEndpoint, SleepEndpoint,
};
use template_icd::{ENDPOINT_LIST, TOPICS_IN_LIST, TOPICS_OUT_LIST};

pub type WireRxBuf = &'static mut [u8];

/// Context contains the data that we will pass (as a mutable reference)
/// to each endpoint or topic handler
pub struct Context {
    /// We'll use this unique ID to identify ourselves to the poststation
    /// server. This should be unique per device.
    pub unique_id: u64,
    pub led: Output<'static>,
}

impl SpawnContext for Context {
    type SpawnCtxt = TaskContext;

    fn spawn_ctxt(&mut self) -> Self::SpawnCtxt {
        TaskContext {
            unique_id: self.unique_id,
        }
    }
}

pub struct TaskContext {
    pub unique_id: u64,
}

// Type Aliases
//
// These aliases are used to keep the types from getting too out of hand.

/// BufStorage is the space used for receiving and sending frames. These values
/// control the largest frames we can send or receive.
pub type BufStorage = PacketBuffers<1024, 1024>;
/// AppTx is the type of our sender, which is how we send information to the client
pub type AppTx = RttTx<ThreadModeRawMutex>;
/// AppRx is the type of our receiver, which is how we receive information from the client
pub type AppRx = RttRx;
/// AppServer is the type of the postcard-rpc server we are using
pub type AppServer = Server<AppTx, AppRx, WireRxBuf, MyApp>;

/// Statically store our packet buffers
pub static PBUFS: ConstStaticCell<BufStorage> = ConstStaticCell::new(BufStorage::new());

// This macro defines your application
define_dispatch! {
    // You can set the name of your app to any valid Rust type name. We use
    // "MyApp" here. You'll use this in `main` to create an instance of the
    // app.
    app: MyApp;
    // This chooses how we spawn functions. Here, we use the implementation
    // from the `embassy_usb_v0_3` implementation
    spawn_fn: embassy_spawn;
    // This is our TX impl, which we aliased above
    tx_impl: AppTx;
    // This is our spawn impl, which also comes from `embassy_usb_v0_3`.
    spawn_impl: EUsbWireSpawn;
    // This is the context type we defined above
    context: Context;

    // Endpoints are how we handle request/response pairs from the client.
    //
    // The "EndpointTy" are the names of the endpoints we defined in our ICD
    // crate. The "kind" is the kind of handler, which can be "blocking",
    // "async", or "spawn". Blocking endpoints will be called directly.
    // Async endpoints will also be called directly, but will be await-ed on,
    // allowing you to call async functions. Spawn endpoints will spawn an
    // embassy task, which allows for handling messages that may take some
    // amount of time to complete.
    //
    // The "handler"s are the names of the functions (or tasks) that will be
    // called when messages from this endpoint are received.
    endpoints: {
        // This list comes from our ICD crate. All of the endpoint handlers we
        // define below MUST be contained in this list.
        list: ENDPOINT_LIST;

        | EndpointTy                | kind      | handler                       |
        | ----------                | ----      | -------                       |
        | GetUniqueIdEndpoint       | blocking  | unique_id                     |
        // | RebootToPicoBoot          | blocking  | picoboot_reset                |
        | SleepEndpoint             | spawn     | sleep_handler                 |
        | SetLedEndpoint            | blocking  | set_led                       |
        | GetLedEndpoint            | blocking  | get_led                       |
    };

    // Topics IN are messages we receive from the client, but that we do not reply
    // directly to. These have the same "kinds" and "handlers" as endpoints, however
    // these handlers never return a value
    topics_in: {
        // This list comes from our ICD crate. All of the topic handlers we
        // define below MUST be contained in this list.
        list: TOPICS_IN_LIST;

        | TopicTy                   | kind      | handler                       |
        | ----------                | ----      | -------                       |
    };

    // Topics OUT are the messages we send to the client whenever we'd like. Since
    // these are outgoing, we do not need to define handlers for them.
    topics_out: {
        // This list comes from our ICD crate.
        list: TOPICS_OUT_LIST;
    };
}

//////////////////////////////////////////////////////////////////////////////
// SPAWN
//////////////////////////////////////////////////////////////////////////////

/// A [`WireSpawn`] impl using the embassy executor
#[derive(Clone)]
pub struct EUsbWireSpawn {
    /// The embassy-executor spawner
    pub spawner: Spawner,
}

impl From<Spawner> for EUsbWireSpawn {
    fn from(value: Spawner) -> Self {
        Self { spawner: value }
    }
}

impl WireSpawn for EUsbWireSpawn {
    type Error = SpawnError;

    type Info = Spawner;

    fn info(&self) -> &Self::Info {
        &self.spawner
    }
}

/// Attempt to spawn the given token
pub fn embassy_spawn<Sp, S: Sized>(sp: &Sp, tok: SpawnToken<S>) -> Result<(), Sp::Error>
where
    Sp: WireSpawn<Error = SpawnError, Info = Spawner>,
{
    let info = sp.info();
    info.spawn(tok)
}


/// Static storage for generically sized input and output packet buffers
pub struct PacketBuffers<const TX: usize = 1024, const RX: usize = 1024> {
    /// the transmit buffer
    pub tx_buf: [u8; TX],
    /// thereceive buffer
    pub rx_buf: [u8; RX],
}

impl<const TX: usize, const RX: usize> PacketBuffers<TX, RX> {
    /// Create new empty buffers
    pub const fn new() -> Self {
        Self {
            tx_buf: [0u8; TX],
            rx_buf: [0u8; RX],
        }
    }
}
