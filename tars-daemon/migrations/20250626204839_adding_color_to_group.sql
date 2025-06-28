-- Migration: Add color column to Groups table
ALTER TABLE Groups 
ADD COLUMN color VARCHAR(255) NOT NULL DEFAULT '';

-- Optional: If you want to remove the default after adding the column
-- (uncomment the next line if desired)
-- ALTER TABLE Groups ALTER COLUMN color DROP DEFAULT;
