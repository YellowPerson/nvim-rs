* Check what we're doing with outgoing request parameters, there are 2 allocations going on in call_args! and rpc_args!, and we're just reading it in the end.

* Can we use the non-generic `split` methods from tokio for unixstream, tcpstream? Supposedly better performance, but introduces lifetimes...
