
default:
    echo 'Hello, world!'

migrate:
    cd common && sqlx migrate run

