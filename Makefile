REPO							?= gianlu33/reactive-event-manager
TAG								?= latest

BACKUP						?= registry.rosetta.ericssondevops.com/gftl-er-5g-hosts/authentic-execution/fosdem-21-images/reactive-event-manager

LOG								?= info
THREADS						?= 16
PERIODIC_TASKS		?= false
MEASURE_TIME			?= false

install:
	cargo install --debug --path .

uninstall:
	cargo uninstall event_manager

build:
	docker build -t $(REPO):$(TAG) .

push: login
	docker push $(REPO):$(TAG)

push_backup:
		docker tag $(REPO):$(TAG) $(BACKUP):$(TAG)
		docker push $(BACKUP):$(TAG)

pull:
	docker pull $(REPO):$(TAG)

run_sgx: check_port
	docker run --rm -v /var/run/aesmd/:/var/run/aesmd/ --network=host --device=/dev/isgx -e EM_PORT=$(PORT) -e EM_LOG=$(LOG) -e EM_THREADS=$(THREADS) -e EM_PERIODIC_TASKS=$(PERIODIC_TASKS) -e EM_MEASURE_TIME=$(MEASURE_TIME) $(REPO):$(TAG)

run_native: check_port
	docker run --rm --network=host -e EM_PORT=$(PORT) -e EM_LOG=$(LOG) -e EM_THREADS=$(THREADS) -e EM_PERIODIC_TASKS=$(PERIODIC_TASKS) -e EM_SGX=false -e EM_MEASURE_TIME=$(MEASURE_TIME) $(REPO):$(TAG)

login:
	docker login

clean:
	docker rm $(shell docker ps -a -q) 2> /dev/null || true
	docker image prune -f

check_port:
	@test $(PORT) || (echo "PORT variable not defined. Run make <target> PORT=<port>" && return 1)
