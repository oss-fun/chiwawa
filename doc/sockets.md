# Sockets

This function is experimental, because it uses the non-standard socket APIs of WAMR and WasmEdge.

## Guest and host

The two extensions are in `wasi_snapshot_preview1`.
The parser finds each extension from the import name and the declared type.
Thus Chiwawa loads a guest that uses the socket library of one or the other runtime.

The `socket-wamr` or `socket-wasmedge` feature sets the host that Chiwawa calls.
This is necessary because a host does not accept a module that imports a function that the host does not supply.
Chiwawa decodes each call from the ABI of the guest into a shared representation (`std::net::SocketAddr`, family, type, option).
Chiwawa then encodes the call for the host.
Thus the guest ABI and the host ABI can be different.
Without a feature, each extension call returns `ENOTSUP`.

On all hosts, Chiwawa sends the standard `sock_accept`, `sock_recv`, `sock_send`, and `sock_shutdown` calls to the host without change.

## Available operations

- open, bind, connect, listen, accept, close
- send, recv, send_to, recv_from
- local and peer address
- name resolution, with a maximum of 32 results for each call
- options on the two hosts: SO_REUSEADDR, SO_KEEPALIVE, SO_BROADCAST, SO_RCVBUF, SO_SNDBUF, SO_RCVTIMEO, SO_SNDTIMEO, SO_LINGER.
- options on WAMR only: SO_REUSEPORT, IP_TTL, IP_MULTICAST_TTL, IP_MULTICAST_LOOP, IP_ADD_MEMBERSHIP, IP_DROP_MEMBERSHIP.
  WasmEdge has no IP-level options.
  Thus a build for WasmEdge returns `ENOPROTOOPT` for them, and it cannot receive multicast data.
- Chiwawa returns `ENOPROTOOPT` for all other options.

## Necessary host functions

- WAMR: `iwasm --addr-pool=<addr/mask>` for the addresses that the guest can use, and `--allow-resolve=<domain>` for name lookups.
- WasmEdge: a version with the V2 socket functions.

A checkpoint records how the guest opened each socket, and a restore opens the socket again with the same fd.
See [migration.md](migration.md#files-and-sockets) for the data that Chiwawa records and the data that it cannot keep.
