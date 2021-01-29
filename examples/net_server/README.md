## Net Server

Server application using laminar.  Use in conjunction with the [Net Client](../net_client) example.

Usage:

- Run the server. (You may alternately run the client first.)
  - After initial debugging information, the server will wait silently.
- Run the client in a separate terminal.
  - The client will silently connect to the server and then noisily send messages to the server with a debug message like:

```log
[INFO][net_client] Sending message for sim frame 1.
```

- When the client receives a message back from the server it will print a debug message like:

```log
[INFO][net_client] Payload: b"ok"
```

- The server, in the meantime will print a message when a client connects or disconnects:

```log
[INFO][net_server] New client connection: 127.0.0.1:63334
...
[INFO][net_server] Client Disconnected: 127.0.0.1:63334
```

- The server will also print address and payload information for each message it receives, though it won't print anything about it's responses.

```log
[INFO][net_server] 127.0.0.1:63334: b"CL: sim_frame:6,abs_time:0.202314849"
```

- If a second client is started the message the server receives will be interleaved, as can be seen by the distinct source ports (the number following `127.0.0.1:`).
