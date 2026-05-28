# Spec: Admin UI

## Overview
Administrative interface for managing system entities (Houses, Tenants, and Devices).

## Requirements

### Requirement: Structured Admin Entity Management
The admin interface SHALL organize Houses, Tenants, and Devices into distinct sections using a summary-detail interaction pattern to minimize clutter and accidental edits.

#### Scenario: Admin views summary list
- **WHEN** the administrator navigates to the Admin panel
- **THEN** they see a list of compact summary cards for Houses, Tenants, and Devices, displaying only essential identifying information

#### Scenario: Admin views entity details
- **WHEN** the administrator clicks on a summary card
- **THEN** a detail view (modal or slide-over) opens showing all configuration fields for that entity in a read-only state

### Requirement: Admin Entity Edit Mode
The admin detail view SHALL provide an explicit "Edit" mode to allow modifications to entity configurations.

#### Scenario: Admin toggles edit mode
- **WHEN** the administrator clicks the "Edit" button in a detail view
- **THEN** all editable fields become active inputs, and "Save" and "Cancel" buttons are displayed

#### Scenario: Admin saves changes
- **WHEN** the administrator clicks "Save" after making changes in Edit mode
- **THEN** the system updates the entity via the API and returns to the read-only detail view with the updated information

### Requirement: Unified Entity Creation
The admin panel SHALL provide a prominent, unified creation button for adding new Houses, Tenants, or Devices.

#### Scenario: Admin opens creation dialog
- **WHEN** the administrator clicks the "+" button on the admin overview
- **THEN** they are prompted to choose the type of entity to create, followed by a dialog containing the necessary creation fields
