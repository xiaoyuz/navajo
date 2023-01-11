NAVAJO_SERVER_DOCKER_COMPOSE_DEPS_FILE = build/deps.docker-compose.yaml

################################################################################ 
.PHONY: navajo-server-deps navajo-server-deps-down

# development

navajo-server-deps:
	docker-compose -p navajo-server -f ${NAVAJO_SERVER_DOCKER_COMPOSE_DEPS_FILE} up -d

navajo-server-deps-down:
	docker-compose -p navajo-server -f ${NAVAJO_SERVER_DOCKER_COMPOSE_DEPS_FILE} down
	docker-compose -p navajo-server -f ${NAVAJO_SERVER_DOCKER_COMPOSE_DEPS_FILE} rm -fsv