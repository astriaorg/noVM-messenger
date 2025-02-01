##############################################
## Deploying and Running using Helm and K8s ##
##############################################
defaultNamespace := "astria-dev-cluster"
deploy tool *ARGS:
  @just deploy-{{tool}} {{ARGS}}

delete tool *ARGS:
  @just delete-{{tool}} {{ARGS}}

[private]
deploy-chart chart namespace=defaultNamespace:
  helm install {{chart}}-chart ./charts/{{chart}} --namespace {{namespace}} --create-namespace

[private]
delete-chart chart namespace=defaultNamespace:
  helm uninstall {{chart}}-chart --namespace {{namespace}}

load-image image_name tag=default_docker_tag repo=default_repo_name:
  kind load docker-image {{repo}}/{{image_name}}:{{tag}} --name astria-dev-cluster

deploy-ingress-controller:
  kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/main/deploy/static/provider/kind/deploy.yaml

wait-for-ingress-controller:
  while ! kubectl wait --namespace ingress-nginx --for=condition=ready pod --selector=app.kubernetes.io/component=controller --timeout=600s; do \
    sleep 1; \
  done

hubbleEnabled := "false"
deploy-cluster namespace=defaultNamespace enableHubble=hubbleEnabled:
  kind create cluster --image kindest/node:v1.28.9 --config dev/kubernetes/kind-cluster-config.yml
  @just helm-add-if-not-exist cilium https://helm.cilium.io/ > /dev/null
  helm install cilium cilium/cilium --version 1.15.5 \
      -f ./dev/values/cilium.yml \
      {{ if enableHubble == "true" { '--set hubble.enabled=true --set hubble.relay.enabled=true --set hubble.ui.enabled=true --set hubble.ui.ingress.enabled=true' } else { '' } }} \
      --namespace kube-system
  kubectl create namespace {{namespace}}


[private]
helm-add-if-not-exist repo url:
  helm repo list | grep -q {{repo}} || helm repo add {{repo}} {{url}}

deploy-celestia-local namespace=defaultNamespace:
    helm install celestia-local-chart astria/celestia-local --version 0.7.1 --namespace {{namespace}}

validatorName := "single"
deploy-sequencer name=validatorName:
  helm install \
    -f dev/values/validators/all.yml \
    -f dev/values/validators/{{name}}.yml \
    -n astria-validator-{{name}} --create-namespace \
    {{name}}-sequencer-chart astria/sequencer
deploy-sequencers: (deploy-sequencer "node0") (deploy-sequencer "node1") (deploy-sequencer "node2")

delete-sequencer name=validatorName:
  @just delete chart {{name}}-sequencer astria-validator-{{name}}
delete-sequencers: (delete-sequencer "node0") (delete-sequencer "node1") (delete-sequencer "node2")

deploy-chat-rollup:
  helm dependency update charts/chat-rollup > /dev/null
  helm install chat-rollup-chart ./charts/chat-rollup \
    -n astria-dev-cluster \
    -f dev/values/rollup/chat-dev.yaml

delete-chat-rollup:
  @just delete chart chat-rollup

deploy-astria-local namespace=defaultNamespace: (helm-add-if-not-exist "astria" "https://astriaorg.github.io/charts/")(deploy-celestia-local namespace) (deploy-sequencer)
delete-astria-local namespace=defaultNamespace: (delete-chart "celestia-local" namespace) (delete-sequencer)

clean:
  kind delete cluster --name astria-dev-cluster

clean-persisted-data:
  rm -r /tmp/astria

delete-all: clean clean-persisted-data
