-- Migration 007 — Add system_info JSONB column to devices.
--
-- Stores OS name/version, kernel, hostname, CPU, memory, disk and
-- network interface inventory sent by the agent in the Hello handshake.
-- NULL for devices that enrolled before this migration.

ALTER TABLE devices ADD COLUMN IF NOT EXISTS system_info JSONB;
