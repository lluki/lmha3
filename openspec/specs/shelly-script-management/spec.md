# Spec: Shelly Script Management

## Overview
Management of scripts on Shelly Gen2+ devices, including listing, starting, stopping, and editing source code via MQTT RPC.

## Requirements

### Requirement: List Shelly Scripts
The system SHALL provide an API endpoint to list all scripts installed on a Shelly Gen2+ device.

#### Scenario: Successfully list scripts
- **WHEN** an authorized user requests the script list for a device
- **THEN** the system sends a `Script.List` RPC to the device via MQTT
- **AND** the system waits for the response (async-to-sync)
- **AND** the system returns the list of scripts to the user

### Requirement: Start/Stop Shelly Scripts
The system SHALL provide API endpoints to start (enable) and stop (disable) specific scripts on a Shelly Gen2+ device.

#### Scenario: Successfully start a script
- **WHEN** an authorized user requests to start a script on a device
- **THEN** the system sends a `Script.Start` RPC to the device via MQTT
- **AND** the system waits for the response (async-to-sync)
- **AND** the system confirms success to the user

#### Scenario: Successfully stop a script
- **WHEN** an authorized user requests to stop a script on a device
- **THEN** the system sends a `Script.Stop` RPC to the device via MQTT
- **AND** the system waits for the response (async-to-sync)
- **AND** the system confirms success to the user

### Requirement: Async-to-Sync MQTT RPC
The system SHALL implement a mechanism to send an MQTT RPC request and synchronously wait for the corresponding response from the device.

#### Scenario: RPC response received within timeout
- **WHEN** an RPC request is sent with a unique ID and `src` set to a response topic
- **AND** the device publishes a response to that topic within 5 seconds
- **THEN** the waiting API thread captures the response and returns it

#### Scenario: RPC timeout
- **WHEN** an RPC request is sent but no response is received within 5 seconds
- **THEN** the system returns a timeout error to the user

### Requirement: Get Shelly Script Code
The system SHALL provide an API endpoint to retrieve the source code of a specific script from a Shelly Gen2+ device.

#### Scenario: Successfully get script code
- **WHEN** an authorized user requests the code for a specific script ID
- **THEN** the system issues a `Script.GetCode` RPC call
- **AND** it returns the `code` field from the response

### Requirement: Update Shelly Script Code
The system SHALL provide an API endpoint to update the source code of a specific script on a Shelly Gen2+ device.

#### Scenario: Successfully update script code
- **WHEN** an authorized user submits new code for a specific script ID
- **THEN** the system issues a `Script.PutCode` RPC call with the new code
- **AND** it returns the result of the operation
