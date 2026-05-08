-- Migration 006 — Add interface_name to the packets hypertable.
--
-- Agents now tag each aggregated packet row with the network interface it was
-- captured on (e.g. "eth0", "em0").  Existing rows get NULL, which is fine
-- for historical data; queries should treat NULL as 'unknown'.

ALTER TABLE packets ADD COLUMN IF NOT EXISTS interface_name TEXT;
