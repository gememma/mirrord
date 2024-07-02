use std::{
    collections::{hash_map::Entry, HashMap},
    fmt,
    future::Future,
    hash::{Hash, Hasher},
    net::Ipv4Addr,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{stream::FuturesUnordered, StreamExt};
use mirrord_protocol::{MeshVendor, Port};
use pnet::packet::tcp::TcpFlags;
use tcp_capture::TcpCapture;
use tokio::{
    select,
    sync::{
        broadcast,
        mpsc::{error::TrySendError, Receiver, Sender},
    },
};
use tokio_util::sync::CancellationToken;
use tracing::Level;

use self::{
    messages::{SniffedConnection, SnifferCommand, SnifferCommandInner},
    tcp_capture::RawSocketTcpCapture,
};
use crate::{
    error::AgentError,
    http::HttpVersion,
    util::{ClientId, Subscriptions},
};

pub(crate) mod api;
pub(crate) mod messages;
pub(crate) mod tcp_capture;

/// [`Future`] that resolves to [`ClientId`] when the [`TcpConnectionSniffer`] client drops their
/// [`TcpSnifferApi`](api::TcpSnifferApi).
struct ClientClosed {
    /// [`Sender`] used by [`TcpConnectionSniffer`] to send data to the client.
    /// Here used only to poll [`Sender::closed`].
    client_tx: Sender<SniffedConnection>,
    /// Id of the client.
    client_id: ClientId,
}

impl Future for ClientClosed {
    type Output = ClientId;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let client_id = self.client_id;

        let future = std::pin::pin!(self.get_mut().client_tx.closed());
        std::task::ready!(future.poll(cx));

        Poll::Ready(client_id)
    }
}

#[derive(Debug, Eq, Copy, Clone)]
pub(crate) struct TcpSessionIdentifier {
    /// The remote address that is sending a packet to the impersonated pod.
    ///
    /// ## Details
    ///
    /// If you were to `curl {impersonated_pod_ip}:{port}`, this would be the address of whoever
    /// is making the request.
    pub(crate) source_addr: Ipv4Addr,

    /// Local address of the impersonated pod.
    ///
    /// ## Details
    ///
    /// You can get this IP by checking `kubectl get pod -o wide`.
    ///
    /// ```sh
    /// $ kubectl get pod -o wide
    /// NAME        READY   STATUS    IP
    /// happy-pod   1/1     Running   1.2.3.4   
    /// ```
    pub(crate) dest_addr: Ipv4Addr,
    pub(crate) source_port: u16,
    pub(crate) dest_port: u16,
}

impl PartialEq for TcpSessionIdentifier {
    /// It's the same session if 4 tuple is same/opposite.
    fn eq(&self, other: &TcpSessionIdentifier) -> bool {
        self.source_addr == other.source_addr
            && self.dest_addr == other.dest_addr
            && self.source_port == other.source_port
            && self.dest_port == other.dest_port
            || self.source_addr == other.dest_addr
                && self.dest_addr == other.source_addr
                && self.source_port == other.dest_port
                && self.dest_port == other.source_port
    }
}

impl Hash for TcpSessionIdentifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.source_addr > self.dest_addr {
            self.source_addr.hash(state);
            self.dest_addr.hash(state);
        } else {
            self.dest_addr.hash(state);
            self.source_addr.hash(state);
        }
        if self.source_port > self.dest_port {
            self.source_port.hash(state);
            self.dest_port.hash(state);
        } else {
            self.dest_port.hash(state);
            self.source_port.hash(state);
        }
    }
}

type TCPSessionMap = HashMap<TcpSessionIdentifier, broadcast::Sender<Vec<u8>>>;

const fn is_new_connection(flags: u8) -> bool {
    0 != (flags & TcpFlags::SYN) && 0 == (flags & (TcpFlags::ACK | TcpFlags::RST | TcpFlags::FIN))
}

fn is_closed_connection(flags: u8) -> bool {
    0 != (flags & (TcpFlags::FIN | TcpFlags::RST))
}

#[derive(Debug)]
pub(crate) struct TcpPacketData {
    bytes: Vec<u8>,
    flags: u8,
}

/// Main struct implementing incoming traffic mirroring feature.
/// Utilizes [`TcpCapture`] for sniffing on incoming TCP packets. Transforms them into
/// incoming TCP data streams and sends copy of the traffic to all subscribed clients.
///
/// Can be easily used via [`api::TcpSnifferApi`].
///
/// # Notes on behavior under high load
///
/// Because this struct does not talk directly with the remote peers, we can't apply any back
/// pressure on the incoming connections. There is no reliable mechanism to ensure that all
/// subscribed clients receive all of the traffic. If we wait too long when distributing data
/// between the clients, raw socket's recv buffer will overflow and we'll lose packets.
///
/// Having this in mind, this struct distributes incoming data using [`broadcast`] channels. If the
/// clients are not fast enough to pick up TCP packets, they will lose them
/// ([`broadcast::error::RecvError::Lagged`]).
///
/// At the same time, notifying clients about new connections (and distributing
/// [`broadcast::Receiver`]s) is done with [`tokio::sync::mpsc`] channels (one per client).
/// To prevent global packet loss, this struct does not use the blocking [`Sender::send`] method. It
/// uses the non-blocking [`Sender::try_send`] method, so if the client is not fast enough to pick
/// up the [`broadcast::Receiver`], they will miss the whole connection.
pub(crate) struct TcpConnectionSniffer<T> {
    command_rx: Receiver<SnifferCommand>,
    tcp_capture: T,

    port_subscriptions: Subscriptions<Port, ClientId>,
    sessions: TCPSessionMap,

    client_txs: HashMap<ClientId, Sender<SniffedConnection>>,
    clients_closed: FuturesUnordered<ClientClosed>,
}

impl<T> fmt::Debug for TcpConnectionSniffer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TcpConnectionSniffer")
            .field("clients", &self.client_txs.keys())
            .field("port_subscriptions", &self.port_subscriptions)
            .field("open_tcp_sessions", &self.sessions.keys())
            .finish()
    }
}

impl TcpConnectionSniffer<RawSocketTcpCapture> {
    /// Creates and prepares a new [`TcpConnectionSniffer`] that uses BPF filters to capture network
    /// packets.
    ///
    /// The capture uses a network interface specified by the user, if there is none, then it tries
    /// to find a proper one by starting a connection. If this fails, we use "eth0" as a last
    /// resort.
    #[tracing::instrument(level = Level::TRACE, skip(command_rx), err)]
    pub async fn new(
        command_rx: Receiver<SnifferCommand>,
        network_interface: Option<String>,
        mesh: Option<MeshVendor>,
    ) -> Result<Self, AgentError> {
        let tcp_capture = RawSocketTcpCapture::new(network_interface, mesh).await?;

        Ok(Self {
            command_rx,
            tcp_capture,

            port_subscriptions: Default::default(),
            sessions: TCPSessionMap::new(),

            client_txs: HashMap::new(),
            clients_closed: Default::default(),
        })
    }
}

impl<R> TcpConnectionSniffer<R>
where
    R: TcpCapture,
{
    pub const TASK_NAME: &'static str = "Sniffer";

    /// Capacity of [`broadcast`] channels used to distribute incoming TCP packets between clients.
    const CONNECTION_DATA_CHANNEL_CAPACITY: usize = 512;

    /// Runs the sniffer loop, capturing packets.
    #[tracing::instrument(level = Level::DEBUG, skip(cancel_token), err)]
    pub async fn start(mut self, cancel_token: CancellationToken) -> Result<(), AgentError> {
        loop {
            select! {
                command = self.command_rx.recv() => {
                    let Some(command) = command else {
                        tracing::debug!("command channel closed, exiting");
                        break;
                    };

                    self.handle_command(command)?;
                },

                Some(client_id) = self.clients_closed.next() => {
                    self.handle_client_closed(client_id)?;
                }

                result = self.tcp_capture.next() => {
                    let (identifier, packet_data) = result?;
                    self.handle_packet(identifier, packet_data)?;
                }

                _ = cancel_token.cancelled() => {
                    tracing::debug!("token cancelled, exiting");
                    break;
                }
            }
        }

        Ok(())
    }

    /// New layer is connecting to this agent sniffer.
    #[tracing::instrument(level = Level::TRACE, skip(sender))]
    fn handle_new_client(&mut self, client_id: ClientId, sender: Sender<SniffedConnection>) {
        self.client_txs.insert(client_id, sender.clone());
        self.clients_closed.push(ClientClosed {
            client_tx: sender.clone(),
            client_id,
        });
    }

    /// Removes the client with `client_id`, and also unsubscribes its port.
    /// Adjusts BPF filter if needed.
    #[tracing::instrument(level = Level::TRACE, err)]
    fn handle_client_closed(&mut self, client_id: ClientId) -> Result<(), AgentError> {
        self.client_txs.remove(&client_id);

        if self.port_subscriptions.remove_client(client_id) {
            self.update_packet_filter()?;
        }

        Ok(())
    }

    /// Updates BPF filter used by [`Self::tcp_capture`] to match state of
    /// [`Self::port_subscriptions`].
    #[tracing::instrument(level = Level::TRACE, err)]
    fn update_packet_filter(&mut self) -> Result<(), AgentError> {
        let ports = self.port_subscriptions.get_subscribed_topics();

        let filter = if ports.is_empty() {
            tracing::trace!("No ports subscribed, setting dummy bpf");
            rawsocket::filter::build_drop_always()
        } else {
            rawsocket::filter::build_tcp_port_filter(&ports)
        };

        self.tcp_capture.set_filter(filter)?;

        Ok(())
    }

    #[tracing::instrument(level = Level::TRACE, err)]
    fn handle_command(&mut self, command: SnifferCommand) -> Result<(), AgentError> {
        match command {
            SnifferCommand {
                client_id,
                command: SnifferCommandInner::NewClient(sender),
            } => {
                self.handle_new_client(client_id, sender);
            }

            SnifferCommand {
                client_id,
                command: SnifferCommandInner::Subscribe(port, tx),
            } => {
                if self.port_subscriptions.subscribe(client_id, port) {
                    self.update_packet_filter()?;
                }

                let _ = tx.send(port);
            }

            SnifferCommand {
                client_id,
                command: SnifferCommandInner::UnsubscribePort(port),
            } => {
                if self.port_subscriptions.unsubscribe(client_id, port) {
                    self.update_packet_filter()?;
                }
            }
        }

        Ok(())
    }

    /// First it checks the `tcp_flags` with [`is_new_connection`], if that's not the case, meaning
    /// we have traffic from some existing connection from before mirrord started, then it tries to
    /// see if `bytes` contains an HTTP request of some sort. When an HTTP request is
    /// detected, then the agent should start mirroring as if it was a new connection.
    ///
    /// tl;dr: checks packet flags, or if it's an HTTP packet, then begins a new sniffing session.
    #[tracing::instrument(level = Level::TRACE, ret, skip(bytes), fields(bytes = bytes.len()), ret)]
    fn treat_as_new_session(tcp_flags: u8, bytes: &[u8]) -> bool {
        is_new_connection(tcp_flags)
            || matches!(
                HttpVersion::new(bytes),
                Some(HttpVersion::V1 | HttpVersion::V2)
            )
    }

    /// Handles TCP packet sniffed by [`Self::tcp_capture`].
    #[tracing::instrument(
        level = Level::TRACE,
        ret,
        skip(self),
        fields(
            destination_port = identifier.dest_port,
            source_port = identifier.source_port,
            tcp_flags = tcp_packet.flags,
            bytes = tcp_packet.bytes.len(),
        )
    )]
    fn handle_packet(
        &mut self,
        identifier: TcpSessionIdentifier,
        tcp_packet: TcpPacketData,
    ) -> Result<(), AgentError> {
        let data_tx = match self.sessions.entry(identifier) {
            Entry::Occupied(e) => e,
            Entry::Vacant(e) => {
                // Performs a check on the `tcp_flags` and on the packet contents to see if this
                // should be treated as a new connection.
                if !Self::treat_as_new_session(tcp_packet.flags, &tcp_packet.bytes) {
                    // Either it's an existing session, or some sort of existing traffic we don't
                    // care to start mirroring.
                    return Ok(());
                }

                let Some(client_ids) = self
                    .port_subscriptions
                    .get_topic_subscribers(identifier.dest_port)
                    .filter(|ids| !ids.is_empty())
                else {
                    return Ok(());
                };

                tracing::trace!(
                    ?client_ids,
                    "TCP packet should be treated as new session and start connections for clients"
                );

                let (data_tx, _) = broadcast::channel(Self::CONNECTION_DATA_CHANNEL_CAPACITY);

                for client_id in client_ids {
                    let Some(client_tx) = self.client_txs.get(client_id) else {
                        tracing::error!(
                            client_id,
                            destination_port = identifier.dest_port,
                            source_port = identifier.source_port,
                            tcp_flags = tcp_packet.flags,
                            bytes = tcp_packet.bytes.len(),
                            "Failed to find client while handling new sniffed TCP connection, this is a bug",
                        );

                        continue;
                    };

                    let connection = SniffedConnection {
                        session_id: identifier,
                        data: data_tx.subscribe(),
                    };

                    match client_tx.try_send(connection) {
                        Ok(()) => {}

                        Err(TrySendError::Closed(..)) => {
                            // Client closed.
                            // State will be cleaned up when `self.clients_closed` picks it up.
                        }

                        Err(TrySendError::Full(..)) => {
                            tracing::warn!(
                                client_id,
                                destination_port = identifier.dest_port,
                                source_port = identifier.source_port,
                                tcp_flags = tcp_packet.flags,
                                bytes = tcp_packet.bytes.len(),
                                "Client queue of new sniffed TCP connections is full, dropping",
                            );

                            continue;
                        }
                    }
                }

                e.insert_entry(data_tx)
            }
        };

        tracing::trace!("Resolved data broadcast channel");

        if !tcp_packet.bytes.is_empty() && data_tx.get().send(tcp_packet.bytes).is_err() {
            tracing::trace!("All data receivers are dead, dropping data broadcast sender");
            data_tx.remove();
            return Ok(());
        }

        if is_closed_connection(tcp_packet.flags) {
            tracing::trace!("TCP packet closes connection, dropping data broadcast channel");
            data_tx.remove();
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::Duration,
    };

    use api::TcpSnifferApi;
    use mirrord_protocol::{
        tcp::{DaemonTcp, LayerTcp, NewTcpConnection, TcpClose, TcpData},
        ConnectionId, LogLevel,
    };
    use tcp_capture::test::TcpPacketsChannel;
    use tokio::sync::mpsc;

    use super::*;
    use crate::watched_task::{TaskStatus, WatchedTask};

    struct TestSnifferSetup {
        command_tx: Sender<SnifferCommand>,
        task_status: TaskStatus,
        packet_tx: Sender<(TcpSessionIdentifier, TcpPacketData)>,
        times_filter_changed: Arc<AtomicUsize>,
        next_client_id: ClientId,
    }

    impl TestSnifferSetup {
        async fn get_api(&mut self) -> TcpSnifferApi {
            let client_id = self.next_client_id;
            self.next_client_id += 1;
            TcpSnifferApi::new(client_id, self.command_tx.clone(), self.task_status.clone())
                .await
                .unwrap()
        }

        fn times_filter_changed(&self) -> usize {
            self.times_filter_changed.load(Ordering::Relaxed)
        }

        fn new() -> Self {
            let (packet_tx, packet_rx) = mpsc::channel(128);
            let (command_tx, command_rx) = mpsc::channel(16);
            let times_filter_changed = Arc::new(AtomicUsize::default());

            let sniffer = TcpConnectionSniffer {
                command_rx,
                tcp_capture: TcpPacketsChannel {
                    times_filter_changed: times_filter_changed.clone(),
                    receiver: packet_rx,
                },
                port_subscriptions: Default::default(),
                sessions: Default::default(),
                client_txs: Default::default(),
                clients_closed: Default::default(),
            };
            let watched_task = WatchedTask::new(
                TcpConnectionSniffer::<TcpPacketsChannel>::TASK_NAME,
                sniffer.start(CancellationToken::new()),
            );
            let task_status = watched_task.status();
            tokio::spawn(watched_task.start());

            Self {
                command_tx,
                task_status,
                packet_tx,
                times_filter_changed,
                next_client_id: 0,
            }
        }
    }

    /// Simulates two sniffed connections, only one matching client's subscription.
    #[tokio::test]
    async fn one_client() {
        let mut setup = TestSnifferSetup::new();
        let mut api = setup.get_api().await;

        api.handle_client_message(LayerTcp::PortSubscribe(80))
            .await
            .unwrap();

        assert_eq!(
            api.recv().await.unwrap(),
            (DaemonTcp::SubscribeResult(Ok(80)), None),
        );

        for dest_port in [80, 81] {
            setup
                .packet_tx
                .send((
                    TcpSessionIdentifier {
                        source_addr: "1.1.1.1".parse().unwrap(),
                        dest_addr: "127.0.0.1".parse().unwrap(),
                        source_port: 3133,
                        dest_port,
                    },
                    TcpPacketData {
                        bytes: b"hello_1".into(),
                        flags: TcpFlags::SYN,
                    },
                ))
                .await
                .unwrap();

            setup
                .packet_tx
                .send((
                    TcpSessionIdentifier {
                        source_addr: "1.1.1.1".parse().unwrap(),
                        dest_addr: "127.0.0.1".parse().unwrap(),
                        source_port: 3133,
                        dest_port: 80,
                    },
                    TcpPacketData {
                        bytes: b"hello_2".into(),
                        flags: TcpFlags::FIN,
                    },
                ))
                .await
                .unwrap();
        }

        let (message, log) = api.recv().await.unwrap();
        assert_eq!(
            message,
            DaemonTcp::NewConnection(NewTcpConnection {
                connection_id: 0,
                remote_address: "1.1.1.1".parse().unwrap(),
                destination_port: 80,
                source_port: 3133,
                local_address: "127.0.0.1".parse().unwrap(),
            }),
        );
        assert_eq!(log, None);

        let (message, log) = api.recv().await.unwrap();
        assert_eq!(
            message,
            DaemonTcp::Data(TcpData {
                connection_id: 0,
                bytes: b"hello_1".into(),
            }),
        );
        assert_eq!(log, None);

        let (message, log) = api.recv().await.unwrap();
        assert_eq!(
            message,
            DaemonTcp::Data(TcpData {
                connection_id: 0,
                bytes: b"hello_2".into(),
            }),
        );
        assert_eq!(log, None);

        let (message, log) = api.recv().await.unwrap();
        assert_eq!(message, DaemonTcp::Close(TcpClose { connection_id: 0 }),);
        assert_eq!(log, None);
    }

    /// Tests that [`TcpCapture`] filter is replaced only when needed.
    ///
    /// # Note
    ///
    /// Due to fact that [`LayerTcp::PortUnsubscribe`] request does not generate any response, this
    /// test does some sleeping to give the sniffer time to process.
    #[tokio::test]
    async fn filter_replace() {
        let mut setup = TestSnifferSetup::new();

        let mut api_1 = setup.get_api().await;
        let mut api_2 = setup.get_api().await;

        api_1
            .handle_client_message(LayerTcp::PortSubscribe(80))
            .await
            .unwrap();
        assert_eq!(
            api_1.recv().await.unwrap(),
            (DaemonTcp::SubscribeResult(Ok(80)), None),
        );
        assert_eq!(setup.times_filter_changed(), 1);

        api_2
            .handle_client_message(LayerTcp::PortSubscribe(80))
            .await
            .unwrap();
        assert_eq!(
            api_2.recv().await.unwrap(),
            (DaemonTcp::SubscribeResult(Ok(80)), None),
        );
        assert_eq!(setup.times_filter_changed(), 1); // api_1 already subscribed `80`

        api_2
            .handle_client_message(LayerTcp::PortSubscribe(81))
            .await
            .unwrap();
        assert_eq!(
            api_2.recv().await.unwrap(),
            (DaemonTcp::SubscribeResult(Ok(81)), None),
        );
        assert_eq!(setup.times_filter_changed(), 2);

        api_1
            .handle_client_message(LayerTcp::PortSubscribe(81))
            .await
            .unwrap();
        assert_eq!(
            api_1.recv().await.unwrap(),
            (DaemonTcp::SubscribeResult(Ok(81)), None),
        );
        assert_eq!(setup.times_filter_changed(), 2); // api_2 already subscribed `81`

        api_1
            .handle_client_message(LayerTcp::PortUnsubscribe(80))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(setup.times_filter_changed(), 2); // api_2 still subscribes `80`

        api_2
            .handle_client_message(LayerTcp::PortUnsubscribe(81))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(setup.times_filter_changed(), 2); // api_1 still subscribes `81`

        api_1
            .handle_client_message(LayerTcp::PortUnsubscribe(81))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(setup.times_filter_changed(), 3);

        api_2
            .handle_client_message(LayerTcp::PortUnsubscribe(80))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(setup.times_filter_changed(), 4);
    }

    /// Simulates scenario where client does not read connection data fast enough.
    /// Packet buffer should overflow in the [`broadcast`] channel and the client should see the
    /// connection being closed.
    #[tokio::test]
    async fn client_lagging_on_data() {
        let mut setup = TestSnifferSetup::new();
        let mut api = setup.get_api().await;

        api.handle_client_message(LayerTcp::PortSubscribe(80))
            .await
            .unwrap();

        assert_eq!(
            api.recv().await.unwrap(),
            (DaemonTcp::SubscribeResult(Ok(80)), None),
        );

        let session_id = TcpSessionIdentifier {
            source_addr: "1.1.1.1".parse().unwrap(),
            dest_addr: "127.0.0.1".parse().unwrap(),
            source_port: 3133,
            dest_port: 80,
        };

        setup
            .packet_tx
            .send((
                session_id,
                TcpPacketData {
                    bytes: b"hello".into(),
                    flags: TcpFlags::SYN,
                },
            ))
            .await
            .unwrap();

        let (message, log) = api.recv().await.unwrap();
        assert_eq!(
            message,
            DaemonTcp::NewConnection(NewTcpConnection {
                connection_id: 0,
                remote_address: session_id.source_addr.into(),
                destination_port: session_id.dest_port,
                source_port: session_id.source_port,
                local_address: session_id.dest_addr.into(),
            }),
        );
        assert_eq!(log, None);

        let (message, log) = api.recv().await.unwrap();
        assert_eq!(
            message,
            DaemonTcp::Data(TcpData {
                connection_id: 0,
                bytes: b"hello".to_vec(),
            }),
        );
        assert_eq!(log, None);

        for _ in 0..TcpConnectionSniffer::<TcpPacketsChannel>::CONNECTION_DATA_CHANNEL_CAPACITY + 2
        {
            setup
                .packet_tx
                .send((
                    session_id,
                    TcpPacketData {
                        bytes: vec![0],
                        flags: 0,
                    },
                ))
                .await
                .unwrap();
        }

        // Wait until sniffer consumes all messages.
        setup
            .packet_tx
            .reserve_many(setup.packet_tx.max_capacity())
            .await
            .unwrap();

        let (message, log) = api.recv().await.unwrap();
        assert_eq!(message, DaemonTcp::Close(TcpClose { connection_id: 0 }),);
        let log = log.unwrap();
        assert_eq!(log.level, LogLevel::Error);
    }

    /// Simulates scenario where client does not read notifications about new connections fast
    /// enough. Client should miss new connections.
    #[tokio::test]
    async fn client_lagging_on_new_connections() {
        let mut setup = TestSnifferSetup::new();
        let mut api = setup.get_api().await;

        api.handle_client_message(LayerTcp::PortSubscribe(80))
            .await
            .unwrap();

        assert_eq!(
            api.recv().await.unwrap(),
            (DaemonTcp::SubscribeResult(Ok(80)), None),
        );

        let source_addr = "1.1.1.1".parse().unwrap();
        let dest_addr = "127.0.0.1".parse().unwrap();

        // First send `TcpSnifferApi::CONNECTION_CHANNEL_SIZE` + 2 first connections.
        let session_ids =
            (0..=TcpSnifferApi::CONNECTION_CHANNEL_SIZE).map(|idx| TcpSessionIdentifier {
                source_addr,
                dest_addr,
                source_port: 3000 + idx as u16,
                dest_port: 80,
            });
        for session in session_ids {
            setup
                .packet_tx
                .send((
                    session,
                    TcpPacketData {
                        bytes: Default::default(),
                        flags: TcpFlags::SYN,
                    },
                ))
                .await
                .unwrap();
        }

        // Wait until sniffer processes all packets.
        let permit = setup
            .packet_tx
            .reserve_many(setup.packet_tx.max_capacity())
            .await
            .unwrap();
        std::mem::drop(permit);

        // Verify that we picked up `TcpSnifferApi::CONNECTION_CHANNEL_SIZE` first connections.
        for i in 0..TcpSnifferApi::CONNECTION_CHANNEL_SIZE {
            let (msg, log) = api.recv().await.unwrap();
            assert_eq!(log, None);
            assert_eq!(
                msg,
                DaemonTcp::NewConnection(NewTcpConnection {
                    connection_id: i as ConnectionId,
                    remote_address: source_addr.into(),
                    destination_port: 80,
                    source_port: 3000 + i as u16,
                    local_address: dest_addr.into(),
                })
            )
        }

        // Send one more connection.
        setup
            .packet_tx
            .send((
                TcpSessionIdentifier {
                    source_addr,
                    dest_addr,
                    source_port: 3222,
                    dest_port: 80,
                },
                TcpPacketData {
                    bytes: Default::default(),
                    flags: TcpFlags::SYN,
                },
            ))
            .await
            .unwrap();

        // Verify that we missed the last connections from the first batch.
        let (msg, log) = api.recv().await.unwrap();
        assert_eq!(log, None);
        assert_eq!(
            msg,
            DaemonTcp::NewConnection(NewTcpConnection {
                connection_id: TcpSnifferApi::CONNECTION_CHANNEL_SIZE as ConnectionId,
                remote_address: source_addr.into(),
                destination_port: 80,
                source_port: 3222,
                local_address: dest_addr.into(),
            }),
        );
    }
}
