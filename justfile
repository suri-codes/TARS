
default:
    echo 'Hello, world!'

migrate:
    cd common && sqlx migrate run

prepare:    
    cargo sqlx prepare --check --workspace

cleantest:
    rm -rf /tmp/tars/test-db
