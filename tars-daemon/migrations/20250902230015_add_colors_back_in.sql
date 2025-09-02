-- Add migration script here
ALTER TABLE Groups 
ADD COLUMN color VARCHAR(255) NOT NULL DEFAULT 'red';
