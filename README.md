# Simple chat rollup 

## Run in a local cluster
### preparation
Clone the repository and then build a local docker image with
```sh
docker buildx build --load --build-arg up -f ./Dockerfile -t chat-rollup:local-v0.0.1
```
In a differnet terminal open the astria monorepo [chat-rollup chart branch](https://github.com/astriaorg/astria/tree/quasystaty1/chat-rollup/oracle-chart-connection) which will be used to deploy the chat rollup on astria local dev cluster.

### deploy to cluster
In the astria monorepo terminal to deploy the cluster run
```sh
just deploy cluster
just deploy ingress-controller
just wait-for-ingress-controller
```
then run celestia network and sequencer network with
```sh
just deploy astria-local
```
and finally deploy the chat-rollup
```sh
just deploy dev-rollup
```
on deployment the chat-rollup rest endpoint will serve at (http://rest.astria.localdev.me).

## Interact using the cli
Install the rollup cli by running 
```sh
just install-cli
```
Current supported commands:
- rollup-cli rollup transfer --amount <AMOUNT> --private-key <PRIVATE_KEY> <TO_ADDRESS>
- rollup-cli rollup text --private-key <PRIVATE_KEY> <TEXT>

query the rollup state:
curl http://rest.astria.localdev.me/get_text_from_id/{message_id}
curl http://rest.astria.localdev.me/get_account_balance/{address}/{asset}
