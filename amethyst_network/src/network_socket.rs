//! The network client System

use specs::{Fetch, FetchMut, System};
//use std::net::UdpSocket;
use super::{deserialize_event, send_event, ConnectionState, NetConnection, NetConnectionPool,
            NetEvent, NetFilter, NetReceiveBuffer, NetSendBuffer, NetSourcedEvent};
use mio::{Events, Poll, PollOpt, Ready, Token};
use mio::net::UdpSocket;
use serde::Serialize;
use serde::de::DeserializeOwned;
use shrev::*;
use std::clone::Clone;
use std::io::{Error, ErrorKind};
use std::marker::PhantomData;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::str;
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

const SOCKET: Token = Token(0);


// If a client sends both a connect event and other events,
// only the connect event will be considered valid and all others will be lost.



/// The System managing the network state and connections.
/// The T generic parameter corresponds to the network event enum type.
pub struct NetSocketSystem<T>
where
    T: PartialEq,
{
    /// The network socket
    pub socket: UdpSocket,
    pub send_queue_reader: Option<ReaderId<NetSourcedEvent<T>>>,
    pub filters: Vec<Box<NetFilter<T>>>,
    pub poll: Poll,
}

//TODO: add Unchecked Event type list. Those events will be let pass the client connected filter (Example: NetEvent::Connect).
//TODO: add different Filters that can be added on demand, to filter the event before they reach other systems.
impl<T> NetSocketSystem<T>
where
    T: Serialize + PartialEq,
{
    /// Creates a NetClientSystem and binds the Socket on the ip and port added in parameters.
    pub fn new(
        ip: &str,
        port: u16,
        filters: Vec<Box<NetFilter<T>>>,
    ) -> Result<NetSocketSystem<T>, Error> {
        let socket = UdpSocket::bind(&SocketAddr::new(
            IpAddr::from_str(ip).expect("Unreadable input IP."),
            port,
        ))?;
        //socket.set_nonblocking(true)?;
        let poll = Poll::new()?;
        poll.register(&socket, SOCKET, Ready::readable(), PollOpt::level())?;
        Ok(NetSocketSystem {
            socket,
            send_queue_reader: None,
            filters,
            poll,
        })
    }
    /// Connects to a remote server
    pub fn connect(&mut self, target: SocketAddr, pool: &mut NetConnectionPool, client_uuid: Uuid) {
        info!("Sending connection request to remote {:?}", target);
        let conn = NetConnection {
            target,
            state: ConnectionState::Connecting,
            uuid: None,
        };
        send_event(&NetEvent::Connect::<T> { client_uuid }, &conn, &self.socket);
        pool.connections.push(conn);
    }
}

impl<'a, T> System<'a> for NetSocketSystem<T>
where
    T: Send + Sync + Serialize + Clone + DeserializeOwned + PartialEq + 'static,
{
    type SystemData = (
        FetchMut<'a, NetSendBuffer<T>>,
        FetchMut<'a, NetReceiveBuffer<T>>,
        FetchMut<'a, NetConnectionPool>,
    );
    fn run(&mut self, (mut send_buf, mut receive_buf, mut pool): Self::SystemData) {
        let mut events = Events::with_capacity(2048);
        let mut buf = [0 as u8; 2048];


        // Tx
        if self.send_queue_reader.is_none() {
            self.send_queue_reader = Some(send_buf.buf.register_reader());
        }

        let mut count = 0;
        for ev in send_buf.buf.read(self.send_queue_reader.as_mut().unwrap()) {
            let target = pool.connection_from_address(&ev.socket);
            if let Some(t) = target {
                if t.state == ConnectionState::Connected || t.state == ConnectionState::Connecting {
                    count += 1;
                    send_event(&ev.event, &t, &self.socket);
                } else {
                    println!("Tried to send packet while target is not in a connected or connecting state.");
                }
            }
        }

        //Rx mio2
        loop {
            self.poll
                .poll(&mut events, Some(Duration::from_millis(0)))
                .expect("Failed to poll network socket.");

            if events.is_empty(){
                break;
            }

            for raw_event in events.iter() {
                if raw_event.readiness().is_readable() {
                    match self.socket.recv_from(&mut buf) {
                        //Data received
                        Ok((amt, src)) => {
                            let mut connection_dropped = false;
                            let net_event = deserialize_event::<T>(&buf[..amt]);
                            match net_event {
                                Ok(ev) => {
                                    let mut filtered = false;
                                    for mut f in self.filters.iter_mut() {
                                        if !f.allow(&pool, &src, &ev) {
                                            filtered = true;
                                        }
                                    }
                                    if !filtered {
                                        let owned_event = NetSourcedEvent {
                                            event: ev.clone(),
                                            uuid: pool.connection_from_address(&src).and_then(|c| c.uuid),
                                            socket: src,
                                        };
                                        receive_buf.buf.single_write(owned_event);
                                    }
                                }
                                Err(e) => error!("Failed to read network event: {}", e),
                            }
                            if connection_dropped {
                                pool.connections.pop();
                            }
                        }
                        Err(e) => {
                            //No data
                            if e.kind() == ErrorKind::WouldBlock {
                                println!("Would block!");
                                break; //Safely ignores when no packets are waiting in the queue, and stop checking for this time.
                            }
                            error!("couldn't receive a datagram: {}", e);
                        }
                    }
                }
            }
        }


        //Rx Normal
        /*loop{
            let mut buf = [0; 2048];
            match self.socket.recv_from(&mut buf) {
                //Data received
                Ok((amt, src)) => {
                    let mut connection_dropped = false;
                    let net_event = deserialize_event::<T>(&buf[..amt]);
                    match net_event {
                        Ok(ev) => {
                            let mut filtered = false;
                            for mut f in self.filters.iter_mut() {
                                if !f.allow(&pool, &src, &ev) {
                                    filtered = true;
                                }
                            }
                            if !filtered {
                                let owned_event = NetSourcedEvent {
                                    event: ev.clone(),
                                    uuid: pool.connection_from_address(&src).and_then(|c| c.uuid),
                                    socket: src,
                                };
                                receive_buf.buf.single_write(owned_event);
                            }
                        }
                        Err(e) => error!("Failed to read network event: {}", e),
                    }
                    if connection_dropped {
                        pool.connections.pop();
                    }
                }
                Err(e) => {
                    //No data
                    if e.kind() == ErrorKind::WouldBlock {
                        break; //Safely ignores when no packets are waiting in the queue, and stop checking for this time.
                    }
                    error!("couldn't receive a datagram: {}", e);
                }
            }
        }*/

        // Rx MIO (locks after 20-30 events)
        /*let mut events = Events::with_capacity(128);
        let mut buf = [0; 2048];
        self.poll
            .poll(&mut events, Some(Duration::from_millis(0)))
            .expect("Failed to poll network socket.");

        for raw_event in events.iter() {
            println!("Received event: {:?}",raw_event);
            if raw_event.readiness().is_readable() {
                match self.socket.recv_from(&mut buf) {
                    //Data received
                    Ok((amt, src)) => {
                        let mut connection_dropped = false;

                        println!("Received some proper event");

                        let net_event = deserialize_event::<T>(&buf[..amt]);
                        match net_event {
                            Ok(ev) => {
                                let mut filtered = false;
                                for mut f in self.filters.iter_mut() {
                                    if !f.allow(&pool, &src, &ev) {
                                        filtered = true;
                                    }
                                }
                                if !filtered {
                                    let owned_event = NetSourcedEvent {
                                        event: ev.clone(),
                                        uuid: pool.connection_from_address(&src)
                                            .map(|c| c.uuid)
                                            .expect("Failed to find client from address. Add the FilterConnected filter to prevent this error."),
                                        socket: src,
                                    };
                                    receive_buf.buf.single_write(owned_event);
                                }
                            }
                            Err(e) => error!("Failed to read network event: {}", e),
                        }
                        if connection_dropped {
                            pool.connections.pop();
                        }
                    }
                    Err(e) => {
                        //No data
                        if e.kind() == ErrorKind::WouldBlock {
                            break; //Safely ignores when no packets are waiting in the queue, and stop checking for this time.
                        }
                        error!("couldn't receive a datagram: {}", e);
                    }
                }
            }
        }*/

    }
}
