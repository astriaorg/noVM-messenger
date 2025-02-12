# Simple chat rollup 

Rollups on Astria are virtual machine (VM) agnostic, which means that they can
function without relying on a VM — a concept we refer to as a noVM rollup.

Using Astria's Execution API, developers can build rollups tailored to any
application type that operates on transactions, messages, and blocks.

This repo contains a simplified noVM messenger rollup app which users can
interact with using a cli to submit messages and access message history from the
rollup.

## ⚠️ Development Notice ⚠️

This is an example project intended for demonstration and learning purposes
only. It is not production-ready and should not be used in a production
environment.

### Important Notes
- This code may contain incomplete features
- Security measures may not be fully implemented
- Performance optimizations are not included
- Documentation may be limited

Please use this project only as a reference or starting point for your own implementation.


## Run in a local cluster
### deploy to cluster
For more details on deploying the development cluster, please refer to Astria's [charts repo](https://github.com/astriaorg/charts).

To deploy the full stack in one command run
```sh
just deploy-all
```

To deploy components sequentially run
```sh
just deploy cluster
just deploy ingress-controller
just wait-for-ingress-controller
```
build the frontend and load it to the cluster
```sh
just install-frontend
just build-and-load-frontend
```
then run celestia network and sequencer network with
```sh
just deploy astria-local
```
and finally deploy the chat rollup
```sh
just deploy astria-chat
just wait-for-astria-chat
```
on deployment the chat-rollup rest endpoint will serve at (http://rest.astria-chat.localdev.me) and the frontend at (http://chat.astria-chat.localdev.me).

To delete the cluster run
```sh
just delete-all
```
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
