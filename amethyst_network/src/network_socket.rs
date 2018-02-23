//! The network client System

use specs::{System,Fetch,FetchMut};
use std::net::UdpSocket;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::io::{Error,ErrorKind};
use std::str;
use std::str::FromStr;
use std::clone::Clone;
use shrev::*;
use std::marker::PhantomData;
use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;
use super::{NetFilter,NetSourcedEvent,NetSendBuffer,NetReceiveBuffer,NetConnectionPool,ConnectionState,NetEvent,NetConnection,send_event,deserialize_event};

/// The System managing the network state and connections.
/// The T generic parameter corresponds to the network event enum type.
pub struct NetSocketSystem<T> where T: PartialEq{
    /// The network socket
    pub socket: UdpSocket,
    pub send_queue_reader: Option<ReaderId<NetSourcedEvent<T>>>,
    pub filters: Vec<Box<NetFilter<T>>>,
}

//TODO: add Unchecked Event type list. Those events will be let pass the client connected filter (Example: NetEvent::Connect).
//TODO: add different Filters that can be added on demand, to filter the event before they reach other systems.
impl<T> NetSocketSystem<T> where T:Serialize+PartialEq{
    /// Creates a NetClientSystem and binds the Socket on the ip and port added in parameters.
    pub fn new(ip:&str,port:u16,filters:Vec<Box<NetFilter<T>>>)->Result<NetSocketSystem<T>,Error>{
        let socket = UdpSocket::bind(SocketAddr::new(IpAddr::from_str(ip).expect("Unreadable input IP."),port))?;
        socket.set_nonblocking(true)?;
        Ok(
            NetSocketSystem{
                socket,
                send_queue_reader: None,
                filters,
            }
        )
    }
    /// Connects to a remote server
    pub fn connect(&mut self,target:SocketAddr,pool: &mut NetConnectionPool, client_uuid: Uuid){
        info!("Sending connection request to remote {:?}",target);
        let conn = NetConnection{
            target,
            state:ConnectionState::Connecting,
            uuid: None,
        };
        send_event(&NetEvent::Connect::<T>{client_uuid},&conn,&self.socket);
        pool.connections.push(conn);
    }
}

impl<'a, T> System<'a> for NetSocketSystem<T> where T:Send+Sync+Serialize+Clone+DeserializeOwned+PartialEq+'static{
    type SystemData = (
        FetchMut<'a, NetSendBuffer<T>>,
        FetchMut<'a, NetReceiveBuffer<T>>,
        FetchMut<'a, NetConnectionPool>,
    );
    fn run(&mut self, (mut send_buf,mut receive_buf,mut pool): Self::SystemData) {
        //Tx
        if self.send_queue_reader.is_none(){
            self.send_queue_reader = Some(send_buf.buf.register_reader());
        }

        for ev in send_buf.buf.read(self.send_queue_reader.as_mut().unwrap()){
            let target = pool.connection_from_address(&ev.socket);
            if let Some(t) = target{
                if t.state == ConnectionState::Connected || t.state == ConnectionState::Connecting{
                    send_event(&ev.event,&t,&self.socket);
                }
            }
        }

        // Rx
        let mut buf = [0; 2048];
        'read: loop {
            match self.socket.recv_from(&mut buf) {
                //Data received
                Ok((amt, src)) => {
                    let mut connection_dropped = false;

                    let net_event = deserialize_event::<T>(&buf[..amt]);
                    match net_event{
                        Ok(ev)=>{
                            for mut f in self.filters.iter_mut(){
                                if !f.allow(&pool,&src,&ev){
                                    continue 'read;
                                }
                            }
                            let owned_event = NetSourcedEvent {
                                event: ev.clone(),
                                uuid: pool.connection_from_address(&src).map(|c| c.uuid),
                                socket: src,
                            };
                            receive_buf.buf.single_write(owned_event);
                        },
                        Err(e)=>error!("Failed to read network event: {}",e),
                    }
                    if connection_dropped{
                        pool.connections.pop();
                    }
                },
                Err(e) => { //No data
                    if e.kind() == ErrorKind::WouldBlock{
                        break;//Safely ignores when no packets are waiting in the queue, and stop checking for this time.
                    }
                    error!("couldn't receive a datagram: {}", e);
                },
            }
        }
    }
}
