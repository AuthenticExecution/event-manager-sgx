REPO           ?= authexec/event-manager-sgx
TAG            ?= latest

LOG            ?= info
THREADS         = 16
PERIODIC_TASKS ?= false
MEASURE_TIME   ?= false
SGX_DEVICE     ?= /dev/isgx

install:
	cargo install --debug --path .

uninstall:
	cargo uninstall event_manager

build:
	docker build -t $(REPO):$(TAG) .

push: login
	docker push $(REPO):$(TAG)

pull:
	docker pull $(REPO):$(TAG)

run_sgx: check_port
	docker run --rm -v /var/run/aesmd/:/var/run/aesmd/ --network=host --device=$(SGX_DEVICE) -e EM_PORT=$(PORT) -e EM_LOG=$(LOG) -e EM_THREADS=$(THREADS) -e EM_PERIODIC_TASKS=$(PERIODIC_TASKS) -e EM_MEASURE_TIME=$(MEASURE_TIME) $(REPO):$(TAG)

run_native: check_port
	docker run --rm --network=host -e EM_PORT=$(PORT) -e EM_LOG=$(LOG) -e EM_THREADS=$(THREADS) -e EM_PERIODIC_TASKS=$(PERIODIC_TASKS) -e EM_SGX=false -e EM_MEASURE_TIME=$(MEASURE_TIME) $(REPO):$(TAG)

login:
	docker login

check_port:
	@test $(PORT) || (echo "PORT variable not defined. Run make <target> PORT=<port>" && return 1)
