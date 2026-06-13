# Spec: House Deadline Configuration

## Overview
Configuration of the "energy day" boundary for each house.

## Requirements

### Requirement: Per-house deadline configuration
Each house SHALL have a configurable "day deadline" time (HH:MM format). This deadline defines the end of the energy day (transition from catch-up window to new day) and the default expiration for vacation mode.

#### Scenario: Configurable morning deadline
- **WHEN** an administrator sets the "Day Deadline" for House A to "06:00"
- **THEN** House A's energy day ends at 06:00 local time instead of the default 05:00

### Requirement: Default house deadline
Newly created houses SHALL default to a "Day Deadline" of "05:00".

#### Scenario: Default deadline on creation
- **WHEN** a new house is created
- **THEN** its deadline is set to "05:00" automatically
