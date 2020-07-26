CREATE TABLE searches (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  url TEXT,
  result_url TEXT,
  email VARCHAR,
  created_at timestamptz DEFAULT NOW(),
  updated_at timestamptz DEFAULT NOW()
)
