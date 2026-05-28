## ADDED Requirements

### Requirement: Static Type Validation
The system SHALL validate all frontend source code against defined TypeScript schemas before deployment or serving.

#### Scenario: Type error detection
- **WHEN** the source code contains an undefined function call or invalid property access
- **THEN** the TypeScript compiler MUST fail with a descriptive error message
- **AND** the build process MUST be aborted

### Requirement: API Response Mapping
The frontend SHALL define strict interfaces for all backend API models (Houses, Tenants, Devices, Telemetry).

#### Scenario: API contract validation
- **WHEN** a frontend function processes a response from `/api/houses`
- **THEN** it MUST use a typed interface (e.g., `interface House`) to ensure all accessed fields are valid

### Requirement: DOM Element Type Safety
The frontend SHALL use specific DOM element types (e.g., `HTMLButtonElement`, `HTMLDialogElement`) to ensure safe interaction with UI components.

#### Scenario: Modal interaction safety
- **WHEN** code interacts with the `admin-modal` dialog
- **THEN** it MUST cast the element to `HTMLDialogElement` to access native methods like `showModal()` and `close()` safely
