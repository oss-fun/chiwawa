# wasi-threads testsuite

`run.sh` fetches the wasi-threads proposal's own testsuite from
<https://github.com/WebAssembly/wasi-threads/tree/main/test/testsuite> and runs
it against chiwawa.

Each module runs as its own process, because it reports its result through
`proc_exit` -- which an in-process `cargo test` cannot observe.
