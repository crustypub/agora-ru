ALTER TABLE posts 
  ALTER COLUMN rating_minus TYPE UUID[]
  USING ARRAY[]::UUID[];

  ALTER COLUMN rating_plus TYPE UUID[]
  USING ARRAY[]::UUID[];
  
