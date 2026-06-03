## MODIFIED Requirements

### Requirement: Shelly ID Discovery
The system SHALL scan MQTT logs for unregistered Shelly device IDs and provide them as suggestions in the Admin panel when creating new devices. The interaction SHALL be integrated into the unified entity creation dialog.

#### Scenario: Admin sees suggested Shelly IDs
- **WHEN** an administrator opens the "Create Entity" dialog and selects "Device"
- **THEN** the system displays a list of recently discovered MQTT topics/IDs that are not yet registered, which can be selected to pre-fill the creation form
