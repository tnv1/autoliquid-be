DB_NAME=autoliquid-db
DB_USER=postgres
DB_PASSWORD=postgres
DB_PORT=5432

db-up:
	docker run --name $(DB_NAME) \
		-e POSTGRES_DB=$(DB_NAME) \
		-e POSTGRES_USER=$(DB_USER) \
		-e POSTGRES_PASSWORD=$(DB_PASSWORD) \
		-p $(DB_PORT):5432 \
		-d postgres:15

db-clean:
	docker stop $(DB_NAME)
	docker rm $(DB_NAME)