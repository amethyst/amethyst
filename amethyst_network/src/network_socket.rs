//! The network send and receive System

use super::{deserialize_event, send_event, ConnectionState, NetConnection, NetEvent, NetFilter};
use mio::{Events, Poll, PollOpt, Ready, Token};
//use mio::net::UdpSocket;
use serde::Serialize;
use serde::de::DeserializeOwned;
use shrev::*;
use amethyst_core::specs::{Read, Write, ReadStorage, WriteStorage, System,Entity,Entities,Join,Resources,SystemData};
use std::clone::Clone;
use std::io::{Error,ErrorKind};
use std::net::IpAddr;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::str;
use std::str::FromStr;
use std::time::Duration;
use std::collections::HashMap;
use uuid::Uuid;

use std::thread;
use std::sync::mpsc::{channel,Sender,Receiver};

const SOCKET: Token = Token(0);


enum InternalSocketEvent<E>{
    SendEvents{target: NetConnection<E>,events: Vec<E>},
    Stop,
}


// If a client sends both a connect event and other events,
// only the connect event will be considered valid and all others will be lost.

/// The System managing the network state and connections.
/// The T generic parameter corresponds to the network event type.
/// Receives events and filters them.
/// Received events will be inserted into the NetReceiveBuffer resource.
/// To send an event, add it to the NetSendBuffer resource.
///
/// If both a connection (Connect or Connected) event is received at the same time as another event from the same connection,
/// only the connection event will be considered and rest will be filtered out.
// TODO: add Unchecked Event type list. Those events will be let pass the client connected filter (Example: NetEvent::Connect).
// Current behaviour: hardcoded passthrough of Connect and Connected events.
pub struct NetSocketSystem<E>
where
    E: PartialEq,
{
    /// The network socket, currently supports Udp only for demonstration purposes.
    pub socket: UdpSocket,
    
    /// The list of filters applied on the events received.
    //pub filters: Vec<Box<NetFilter<T>>>,
    
    /// The mio's `Poll`.
    //pub poll: Poll,
    pub tx: Sender,
    pub rx: Receiver,
    
    /// Readers corresponding to each of the Connections. Use to keep track of when to send which event to who.
    /// When: When there is a new event that hasn't been read yet.
    /// Which: The event
    /// Who: The NetConnection's SocketAddr attached to the key Entity.
    pub send_queues_readers: HashMap<Entity,ReaderId<NetEvent<E>>>,
}

impl<E> NetSocketSystem<E>
where
    E: Serialize + PartialEq,
{
    /// Creates a `NetSocketSystem` and binds the Socket on the ip and port added in parameters.
    pub fn new(
        ip: &str,
        port: u16,
        //filters: Vec<Box<NetFilter<T>>>,
    ) -> Result<NetSocketSystem<E>, Error> {
        let socket = UdpSocket::bind(&SocketAddr::new(
            IpAddr::from_str(ip).expect("Unreadable input IP."),
            port,
        ))?;
        socket.set_nonblocking(true).unwrap();
        
        // this -> thread
        let (tx1,rx1) = channel();
        // thread -> this
        let (tx2,rx2) = channel();
        
        thread::spawn(move ||{
            //rx1,tx2
            let send_queue = rx1;
            let receive_queue = tx2;
            let mut socket = socket;
            loop 'outer{
                // send
                for control_event in send_queue.try_iter() {
                    match control_event {
                        SendEvents{target,events} => {
                            for ev in events {
                                send_event(ev, &target, &self.socket);
                            }
                        },
                        Stop => break 'outer,
                    }
                }
                // receive
                
            }
        });
        
        //socket.set_write_timeout(Some(Duration::from_millis(100))).unwrap();
        //let poll = Poll::new()?;
        //poll.register(&socket, SOCKET, Ready::readable(), PollOpt::level())?;
        //poll.register(&socket, SOCKET, Ready::readable(), PollOpt::edge())?;
        Ok(NetSocketSystem {
            //socket,
            //filters,
            //poll,
            tx: tx1,
            rx: rx2,
            send_queues_readers: HashMap::new(),
        })
    }
    /*
    /// Connects to a remote server (client-only call)
    pub fn connect(&mut self, target: SocketAddr, pool: &mut NetConnectionPool, client_uuid: Uuid) {
        info!("Sending connection request to remote {:?}", target);
        let conn = NetConnection {
            target,
            state: ConnectionState::Connecting,
            uuid: None,
        };
        send_event(&NetEvent::Connect::<T> { client_uuid }, &conn, &self.socket);
        pool.connections.push(conn);
    }*/
}

impl<'a, E> System<'a> for NetSocketSystem<E>
where
    E: Send + Sync + Serialize + Clone + DeserializeOwned + PartialEq + 'static,
{
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, NetConnection<E>>,
    );
    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        //type InitSystemData = (Entities<'a>,WriteStorage<'a,NetConnection<E>>);
        //for (entity,mut net_connection) in (&*res.fetch::<EntitiesRes>(),&mut res.write_storage::<NetConnection<E>>()).join() {
        
        /*for (entity,mut net_connection) in (&*Entities::fetch(&res),&mut WriteStorage::<NetConnection<E>>::fetch(&res)).join() {
          self.send_queues_readers.insert(entity.clone(),net_connection.send_buffer.register_reader());
        }*/
    }
    fn run(&mut self, (entities,mut net_connections): Self::SystemData) {
        //self.socket.set_nonblocking(false).unwrap();
        for (entity,mut net_connection) in (&*entities,&mut net_connections).join() {
          let mut reader = self.send_queues_readers.entry(entity).or_insert(net_connection.send_buffer.register_reader());
          
          let target = net_connection.target.clone();
          if net_connection.state == ConnectionState::Connected || net_connection.state == ConnectionState::Connecting {
	      //for ev in net_connection.send_buffer_early_read() {
                  //send_event(ev, &target, &self.socket);
                  self.tx.send(InternalSocket::SendEvents{target, net_connection.send_buffer_early_read().collect()).unwrap();
            /*let target = pool.connection_from_address(&ev.socket);
            if let Some(t) = target {
                if t.state == ConnectionState::Connected || t.state == ConnectionState::Connecting {
                    send_event(&ev.event, &t, &self.socket);
                } else {
                    warn!("Tried to send packet while target is not in a connected or connecting state.");
                }
            } else {
                warn!("Targeted address is not in the NetConnection pool.")
            }*/
            //}
          }
        }
        //self.socket.set_nonblocking(true).unwrap();
        // Receives event through mio's `Poll`.
        // I'm not sure if this is the right way to use Poll, but it seems to work.
        //let mut events = Events::with_capacity(2048);
        let mut buf = [0 as u8;2048];
        //loop {
            //self.poll.poll(&mut events, Some(Duration::from_millis(0))).expect("Failed to poll network socket.");
            //self.poll.poll(&mut events, Some(Duration::from_millis(100))).expect("Failed to poll network socket.");
            //self.poll.poll(&mut events, None).expect("Failed to poll network socket.");

            /*if events.is_empty() {
                break;
            }*/
            loop{
            //for raw_event in events.iter() {
                //println!("EVENT!!!");
                //if raw_event.readiness().is_readable() {
                    match self.socket.recv_from(&mut buf) {
                        // Data received
                        Ok((amt, src)) => {
                            let mut matched = false;
                            // Get the NetConnection from the source
                            for mut net_connection in (&mut net_connections).join(){
                                // We found the origin
                                if net_connection.target == src{
                                  matched = true;
                                  // Get the event
                                  let net_event = deserialize_event::<E>(&buf[..amt]);
                                  match net_event {
                                    Ok(ev) => {
                                        // Filter events
                                        let mut filtered = false;
                                        /*for mut f in net_connection.filters.iter_mut() {
                                            if !f.allow(&src, &ev) {
                                                filtered = true;
                                                break;
                                            }
                                        }*/
                                        if !filtered {
                                            net_connection.receive_buffer.single_write(ev);
                                        } else {
                                            info!("Filtered an incoming network packet from source {:?}", src);
                                        }
                                    }
                                    Err(e) => error!("Failed to deserialize an incoming network event: {} From source: {:?}", e, src),
                                  }
                              }
                          }
                          if !matched {
                            println!("Received packet from unknown source");
                          }
                      }
                      Err(e) => {
                          if e.kind() == ErrorKind::WouldBlock{
                              error!("WouldBlock: {}", e);
                                break;
                          } else { 
                              error!("Could not receive datagram: {}... retrying.", e);
                          }
                      }
                    }
            }
                //}
            //}
        //}
    }
}
