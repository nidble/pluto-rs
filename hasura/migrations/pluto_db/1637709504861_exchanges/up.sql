CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE exchanges (
    id uuid DEFAULT uuid_generate_v4(),
    amount_from MONEY NOT NULL,
    amount_to MONEY NOT NULL,
    currency_from CHAR(3) NOT NULL,
    currency_to CHAR(3) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id)
);
