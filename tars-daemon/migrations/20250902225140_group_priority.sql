-- Add migration script here
CREATE TABLE Groups_new(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pub_id VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    parent_id VARCHAR(255),
    priority INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY (parent_id) REFERENCES Groups (pub_id)  ON UPDATE CASCADE ON DELETE CASCADE    
    -- FOREIGN KEY (parent_id) REFERENCES Groups (pub_id) ON UPDATE CASCADE
);

INSERT INTO Groups_new(
    id, pub_id, name, parent_id, priority, created_at, updated_at
)
SELECT id, pub_id, name, parent_id, created_at, updated_at, 0 AS priority
FROM Groups;

DROP TABLE Groups;

ALTER TABLE Groups_new RENAME TO Groups;

CREATE INDEX idx_groups_pub_id ON Groups (pub_id);



