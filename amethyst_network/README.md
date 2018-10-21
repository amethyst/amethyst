# Amethyst Networking

The networking crate for the `amethyst` game engine. The main engine can be found at https://amethyst.rs.

## Status

Right now this crate is very simple and it just supports sending messages through specs to other clients. It uses a combination of `EventChannel`s
to send events between the network thread. In its current state it is not ready to be used in a game but there is work going into [Laminar](https://github.com/amethyst/laminar) that will soon be the underlying socket implemenation for this crate.

For more information or help, please come find us on the discord server's #net channel.
