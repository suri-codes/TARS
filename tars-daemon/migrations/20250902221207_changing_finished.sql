
-- Step 1: Create new Tasks table with finished_at column
CREATE TABLE Tasks_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pub_id VARCHAR(255) NOT NULL UNIQUE,
    group_id VARCHAR(255) NOT NULL, 
    name VARCHAR(255) NOT NULL,
    priority INTEGER NOT NULL,
    description TEXT NOT NULL,
    due DATETIME,
    finished_at DATETIME,  -- New column: NULL if not finished, DATETIME if finished
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY (group_id) REFERENCES Groups (pub_id) ON UPDATE CASCADE ON DELETE CASCADE
);

-- Step 2: Copy data from old table to new table
-- Convert completed=TRUE to current timestamp, completed=FALSE to NULL
INSERT INTO Tasks_new (
    id, pub_id, group_id, name, priority, description, due, 
    finished_at, created_at, updated_at
)
SELECT 
    id, pub_id, group_id, name, priority, description, due,
    CASE 
        WHEN completed = TRUE THEN CURRENT_TIMESTAMP  -- Set to current time if completed
        ELSE NULL  -- Set to NULL if not completed
    END as finished_at,
    created_at, updated_at
FROM Tasks;

-- Step 3: Drop the old table
DROP TABLE Tasks;

-- Step 4: Rename new table to original name
ALTER TABLE Tasks_new RENAME TO Tasks;

-- Step 5: Recreate indexes
CREATE INDEX idx_tasks_pub_id ON Tasks (pub_id);
CREATE INDEX idx_tasks_group_id ON Tasks (group_id);

-- Optional: Create index on finished_at for queries filtering by completion status
CREATE INDEX idx_tasks_finished_at ON Tasks (finished_at);


