## MODIFIED Requirements

### Requirement: Decision Engine (Background Thread)
The decision engine SHALL run in a dedicated background thread, polling Home Assistant for telemetry and incorporating historical device state data to fulfill long-term scheduling constraints. It MUST accept an explicit "now" timestamp for deterministic behavior and testing.

#### Scenario: History-aware polling
- **WHEN** the scheduler is invoked
- **THEN** it retrieves current PV/Consumption and the necessary history of ON/OFF events for all Boiler-mode devices

### Requirement: Production/Demand matching (BOILER)
The system SHALL match PV production with device loads using an optimized runtime-aware strategy:
- **Fair Distribution**: When multiple devices are eligible to turn ON, the system SHALL pick the one that has been OFF the longest. When multiple devices must turn OFF, the system SHALL pick the one that has been ON the longest.
- **Improved Hysteresis**: A device SHALL stay ON if `PV_Production > (House_Consumption_Excluding_Device + 0.3 * Device_Load)`. This allows keeping the device ON even if it requires up to 70% grid power, provided the PV supply covers at least 30% of its specific load.
- **Incremental Activation**: The system SHALL support turning on multiple devices across sequential scheduler cycles as production allows.

#### Scenario: Longest idle activation
- **WHEN** two devices are eligible for BOILER mode activation
- **THEN** the device that has been OFF for the longest duration is activated first

#### Scenario: Longest running deactivation
- **WHEN** production drops and a device must be turned OFF
- **THEN** the device that has been ON for the longest duration is deactivated first

#### Scenario: Grid-assisted retention
- **WHEN** a 4kW device is ON, PV is 3kW, and other house consumption is 1.5kW
- **THEN** the device remains ON (since 3kW PV > 1.5kW base + 1.2kW [30% of 4kW])

### Requirement: Mandatory Charging
The system SHALL guarantee minimum charge levels and periodic full charges (defined as 4h of runtime within a 5am-5am window):
- **Full Charge Deadline**: If `full_charge_n_day` days have passed without a 4h charge, the system SHALL force the device ON starting at 1am (4h before the 5am deadline) on the final day.
- **Daily Minimum**: The system SHALL ensure `min_daily_charge` is met within every 5am-5am cycle, forcing the device ON if the deadline approaches and the quota is not met.

#### Scenario: Full charge trigger
- **WHEN** it is 1am and a device requiring a full charge every 1 day has not reached 4h of runtime since 5am the previous day
- **THEN** the device is forced ON regardless of PV production
