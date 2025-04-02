database_url := '"sqlite:///home/suri/.local/share/common/tars.db"'

default:
    echo 'Hello, world!'

migrate:
    cd common && sqlx migrate run --database-url {{database_url}}

