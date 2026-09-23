# Sockets

Experimental: depends on the non-standard socket APIs of WAMR and WasmEdge.

## Guest and host

Both extensions live in `wasi_snapshot_preview1`.
The parser tells them apart by import name and declared type, so a guest built against either runtime's socket library loads.
The host Chiwawa calls is fixed by the `socket-wamr` or `socket-wasmedge` feature, because a host refuses a module that imports what it does not provide.
Each call is decoded from the guest's ABI into a common form (`std::net::SocketAddr`, family, type, option) and encoded for the host, so the guest and host ABIs need not match.
Without a feature every extension call returns `ENOTSUP`.

The standard `sock_accept`, `sock_recv`, `sock_send` and `sock_shutdown` are passed through as before on every host.

## Supported

- open, bind, connect, listen, accept, close
- send, recv, send_to, recv_from
- local and peer address
- name resolution, at most 32 results per call
- options: SO_REUSEADDR, SO_KEEPALIVE, SO_BROADCAST, SO_RCVBUF, SO_SNDBUF, SO_RCVTIMEO, SO_SNDTIMEO, SO_LINGER. Any other option is `ENOPROTOOPT`.

## Host requirements

- WAMR: `iwasm --addr-pool=<addr/mask>` for the addresses the guest may use and `--allow-resolve=<domain>` for name lookups.
- WasmEdge: a version with the V2 socket functions.

A checkpoint does not capture sockets; after a restore every socket fd is gone, as with any other fd.
