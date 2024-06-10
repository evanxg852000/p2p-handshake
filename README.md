# p2p-handshake

A rust implementation of a handshake protocol with an [Ergo](https://ergoplatform.org/en/) platform node.


## How to run

> [!NOTE]  
> The following was tested on Ubuntu 20.04, other platforms might work but they have not been tested using these steps.

### Running an Ergo node
You first need to have a target node against which you want to connect and perform a handshake. it can be a local node or a publicly known node.

The easiest way would be to run the official Ergo node docker image. I have already provided a default configuration that will run on the `testnet` network on `0.0.0.0:9020`. All you need is execute the following command from the project directory.

```bash
docker run --rm -d \
    -p 9020:9020 \
    -v ./ergo.conf:/etc/ergo.conf \
    ergoplatform/ergo:v5.0.21 --testnet -c /etc/ergo.conf
```
You can check the container has started by checking the output of `docker ps`.

If you don't have docker on your machine, you can download the `.jar` binary file and manually run on your machine using the instruction [here](https://docs.ergoplatform.com/node/install/manual/). Just remember to use the provide config file in this repos (e.i binding to the correct port is important).

```bash
java -jar -Xmx2G ergo-*.jar --testnet -c ergo.conf
```

### Executing the handshake command

You will need a rust toolchain to build and execute the source code. Assuming you already have a rust toolchain, just cargo run it. 

```bash
cargo run -- --target 0.0.0.0:9020 --name evan --version 3.3.6
```

You can also build a release version and run the resulting binary located at `./target/release/p2p-handshake`.

```bash
cargo build --release
./target/release/p2p-handshake --help
./target/release/p2p-handshake --target 0.0.0.0:9020 --name evan --version 3.3.6
```

## References

- Protocol docs: https://docs.ergoplatform.com/dev/p2p/p2p-handshake/
- Node setup: https://docs.ergoplatform.com/node/install/manual/

