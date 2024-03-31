# Raft IK
This is a poc of the raft consensus algorithm. The algorithm is based on the original paper
[In Search of an Understandable Consensus Algorithm](https://dl.acm.org/doi/10.5555/2643634.2643666).
The goal is to understand how Raft works and along the way improve my knowledge about rust.

## Status
* Instead of using RPC, I use a simple api call. The next major improve will be replaced it with the use of RPC.
* At this moment I will only concentrate in the leader election protocol only.

## Instructions on how to use it.
We need at least three instances or nodes to use it. These are examples of the config files you need to have in order to execute.

**Node 1**
```
HOST=127.0.0.1:7001
ID_NODE=1
ID_NODES=2,3
HOST_2=127.0.0.1:7002
HOST_3=127.0.0.1:7003
RUST_LOG=info
NODE_CLIENT_TIMEOUT_MS=10
```
**Node 2**
```
HOST=127.0.0.1:7002
ID_NODE=2
ID_NODES=1,3
HOST_1=127.0.0.1:7001
HOST_3=127.0.0.1:7003
RUST_LOG=info
NODE_CLIENT_TIMEOUT_MS=10
```
**Node 3**
```
HOST=127.0.0.1:7003
ID_NODE=3
ID_NODES=1,2
HOST_1=127.0.0.1:7001
HOST_2=127.0.0.1:7002
RUST_LOG=info
NODE_CLIENT_TIMEOUT_MS=10
```
If you want you can create a tmp folder with the following structure and use the zellij's layout I provide to facilitate the execution and control.
Just create the folder and execute zellij like this:
```shell
mkdir -p tmp/cluster_1;
mkdir -p tmp/cluster_2;
mkdir -p tmp/cluster_3;
zellij -l code-layout.kdl;
```
