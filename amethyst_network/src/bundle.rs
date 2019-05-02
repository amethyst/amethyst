use std::{marker::PhantomData, net::SocketAddr};

use serde::{de::DeserializeOwned, Serialize};

use amethyst_core::{bundle::SystemBundle, shred::DispatcherBuilder};
use amethyst_error::{Error, ResultExt};

use crate::{server::ServerConfig, NetSocketSystem};

/// A convenience bundle to create the infrastructure needed to send and receive network messages.
pub struct NetworkBundle<T> {
    /// the configuration used for the networking crate.
    config: ServerConfig,
    _data: PhantomData<T>,
}

impl<T> NetworkBundle<T> {
    /// Creates a new NetworkBundle.
    ///
    /// `receive_addr`: this is the address on which we will receive incoming packets.
    /// `send_addr`: this is the address from which we will send outgoing packets.
    pub fn new(udp_socket_addr: SocketAddr) -> Self {
        let config = ServerConfig {
            udp_socket_addr,
            max_throughput: 5000,
            create_net_connection_on_connect: true,
        };

        NetworkBundle {
            config,
            _data: PhantomData,
        }
    }

    /// Construct a new `NetworkBundle` with the specified configuration.
    pub fn from_config(config: ServerConfig) -> NetworkBundle<T> {
        NetworkBundle {
            config,
            _data: PhantomData,
        }
    }
}

impl<'a, 'b, T> SystemBundle<'a, 'b> for NetworkBundle<T>
where
    T: Send + Sync + PartialEq + Serialize + Clone + DeserializeOwned + 'static,
{
    /// Build the networking bundle by adding the networking system to the application.
    fn build(self, builder: &mut DispatcherBuilder<'_, '_>) -> Result<(), Error> {
        let socket_system = NetSocketSystem::<T>::new(self.config)
            .with_context(|_| Error::from_string("Failed to open network system."))?;

        builder.add(socket_system, "net_socket", &[]);

        Ok(())
    }
}
