## ADDED Requirements

### Requirement: Fixed Daily Runtime Algorithm
The system SHALL ensure each device in `BOILER` mode runs for a configurable `device_runtime` (default 3 hours) exactly once during each 24-hour cycle (5:00 AM to 5:00 AM).

#### Scenario: PV-based activation
- **WHEN** it is between 5:00 AM and 1:00 AM, and PV surplus (PV Production - House Consumption) > 70% of a device's expected load, AND the device has not yet run during the current cycle
- **THEN** the system SHALL select one such device at random and turn it ON

#### Scenario: Guaranteed duration
- **WHEN** a device has been activated via the PV-based activation or catch-up window
- **THEN** the system SHALL keep it ON for the full `device_runtime` regardless of subsequent PV production changes

#### Scenario: Catch-up window
- **WHEN** it is 1:00 AM and a device has not yet run during the current cycle
- **THEN** the system SHALL turn the device ON for its full `device_runtime`

## MODIFIED Requirements

### Requirement: Decision Engine (Background Thread)
The decision engine SHALL implement the logic for matching load consumption with solar production and handling manual tenant overrides.
- The decision engine SHALL run in a dedicated background thread within the server process.
- The decision engine SHALL run as a background process that iterates through all configured houses. For each house, it MUST:
    - Retrieve current PV production and house consumption from its specific Home Assistant instance.
    - Incorporate house-scoped historical device state data to track runtimes within the 5:00 AM to 5:00 AM window.
    - MUST accept an explicit "now" timestamp for deterministic behavior and testing.

#### Scenario: Background execution
- **WHEN** the server starts
- **THEN** it SHALL spawn the decision engine thread

#### Scenario: Multi-house scheduling cycle
- **WHEN** the scheduler triggers a new cycle
- **THEN** it sequentially processes "House A" and "House B" without cross-contamination of state

#### Scenario: History-aware polling
- **WHEN** the scheduler is invoked
- **THEN** it retrieves current PV/Consumption and the necessary history of ON/OFF events for the current 5 AM - 5 AM cycle to determine which devices have already run

### Requirement: Logic Priority
The system SHALL apply scheduling logic according to the following strict priority:
1. **Manual / Forced States:** `FORCE_ON` / `FORCE_OFF` MUST take absolute precedence until `scheduling_until` is reached.
2. **Auto-Transition:** When `now() > scheduling_until`, the device SHALL automatically transition from `FORCE_*` back to `BOILER`.
3. **In-Progress Runtime:** If a device is currently running its `device_runtime`, it SHALL remain ON until the duration is met.
4. **Catch-up Window:** If it is after 1:00 AM and a device hasn't run today, it SHALL be forced ON.
5. **Production/Demand matching (BOILER):** PV-based activation SHALL be used for devices that haven't run today.

#### Scenario: Priority of catch-up over PV
- **WHEN** it is 1:05 AM and a device has not run
- **THEN** the system forces it ON even if PV is zero

## MODIFIED Requirements
