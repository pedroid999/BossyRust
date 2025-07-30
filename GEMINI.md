# Gemini Development Guidelines for BossyRust

This document provides a comprehensive guide for developing, fixing, and refactoring the BossyRust project. The goal is to maintain a high standard of code quality, ensuring the application is robust, maintainable, and scalable, following Clean Code and SOLID principles.

## 1. Project Overview

BossyRust is a Terminal User Interface (TUI) application for macOS that provides process and network port management. It is built in Rust, using `ratatui` for the TUI, `tokio` for its asynchronous runtime, and `clap` for the command-line interface.

The project is structured into the following core modules:
- `src/process`: Handles process information, monitoring, and termination.
- `src/network`: Manages network ports and connections.
- `src/tui`: Contains all UI-related logic, including application state, views, and event handling.
- `src/commands`: Implements the command-line interface logic.
- `src/config`: Manages application configuration from `presets.toml`.

## 2. Core Development Principles

- **SOLID**:
    - **Single Responsibility Principle (SRP)**: Each struct, function, and module should have one, and only one, reason to change. For example, the `process::killer` module should only be responsible for terminating processes, not for fetching their information.
    - **Open/Closed Principle (OCP)**: Software entities should be open for extension but closed for modification. When adding new features, prefer adding new code over changing existing, working code. For instance, to support a new type of process filter, it should be possible to add a new filter implementation without altering the core filtering logic.
    - **Liskov Substitution Principle (LSP)**: Subtypes must be substitutable for their base types. When using traits, ensure that any implementation of a trait can be used wherever the trait is expected, without causing incorrect behavior.
    - **Interface Segregation Principle (ISP)**: Clients should not be forced to depend on interfaces they do not use. Create small, focused traits rather than large, monolithic ones. For example, a trait for "killable" entities is better than a single, large "Process" trait with methods for killing, reading, and monitoring.
    - **Dependency Inversion Principle (DIP)**: High-level modules should not depend on low-level modules. Both should depend on abstractions. Use traits to define contracts between modules, and use dependency injection to provide concrete implementations.

- **DRY (Don't Repeat Yourself)**: Avoid duplicating code. Use functions, modules, and macros to abstract away common patterns.

- **YAGNI (You Ain't Gonna Need It)**: Do not add functionality until it is necessary. Avoid over-engineering and focus on delivering value with the simplest possible solution.

## 3. Architectural Guidelines

- **Maintain Modularity**: Respect the existing module boundaries. Code related to process management belongs in the `process` module, network code in the `network` module, and so on.
- **Separation of Concerns**:
    - **UI vs. Logic**: The `tui` module should be responsible for rendering the UI and handling user input. It should not contain business logic. The core logic should reside in the `process`, `network`, and `commands` modules.
    - **Data Fetching vs. Data Presentation**: The logic for fetching data (e.g., listing processes) should be separate from the logic for presenting it in the TUI.
- **Asynchronous Operations**: Use Tokio's asynchronous capabilities for all I/O-bound operations, such as reading process information or scanning network ports, to ensure the UI remains responsive.

## 4. Module-Specific Guidelines

### `src/process`
- **`info.rs`**: Should only be concerned with fetching information about processes.
- **`killer.rs`**: Should only handle the logic for terminating processes. It should receive the process ID and other necessary information, but not be responsible for fetching it.
- **`monitor.rs`**: Should focus on real-time monitoring of processes.

### `src/network`
- **`ports.rs`**: Should be responsible for fetching information about network ports and the processes using them.
- **`utils.rs`**: Should contain utility functions related to networking, such as checking port availability.

### `src/tui`
- **`app.rs`**: Manages the application's state. It should not perform any I/O operations directly. Instead, it should delegate those tasks to other modules and update its state based on the results.
- **`dashboard.rs`**: Should be responsible for rendering the different UI views. It should be driven by the state in `app.rs`.
- **`events.rs`**: Handles user input and other events. It should translate events into actions that are then handled by the `app` module.

### `src/commands`
- **`cli.rs`**: Implements the logic for the command-line interface. It should parse the command-line arguments and call the appropriate functions in the core logic modules.

### `src/config`
- This module should be the single source of truth for all configuration. All other modules should query this module for configuration values.

## 5. Error Handling

- Use the `anyhow` crate for application-level error handling, where you need to propagate errors up to the user.
- For library-like modules (`process`, `network`), consider defining custom error types to provide more specific error information. This allows the calling code to handle different error cases programmatically.
- Always handle potential errors gracefully. The application should not panic on expected errors, such as a process not being found or a file not being accessible.

## 6. Testing

- **Unit Tests**: Each module should have unit tests for its core logic. Place unit tests in a `tests` submodule within the file they are testing (e.g., `src/process/killer.rs` would have a `#[cfg(test)] mod tests { ... }` section).
- **Integration Tests**: Use the `tests` directory for integration tests that verify the interaction between different parts of the application, including the CLI.
- **Mocking**: Use the `mockall` crate to mock dependencies in unit tests, allowing you to test components in isolation.

## 7. Code Style and Formatting

- **`cargo fmt`**: All code must be formatted with `cargo fmt`. Run `cargo fmt --check` in the CI pipeline to enforce this.
- **`cargo clippy`**: All code should be free of `clippy` warnings. Run `cargo clippy -- -D warnings` in the CI pipeline to enforce this.
- **Naming Conventions**: Follow the official Rust naming conventions (e.g., `snake_case` for variables and functions, `PascalCase` for types).

## 8. Refactoring Candidates

- **Configuration**: The configuration is currently loaded from a `presets.toml` file. Consider making the configuration more robust by allowing user-specific overrides and providing a clear hierarchy (e.g., default -> file -> environment variables -> CLI arguments).
- **Platform-Specific Code**: The application is designed for macOS. If there is any desire to support other platforms in the future, start abstracting away the platform-specific code behind traits.
- **TUI State Management**: As the TUI grows in complexity, consider adopting a more structured state management pattern (e.g., a Redux-like architecture) to make state changes more predictable and easier to debug.

## 9. Contribution Workflow

1.  **Create an Issue**: Before starting work on a new feature or bug fix, create an issue to discuss the proposed changes.
2.  **Fork and Branch**: Fork the repository and create a new branch for your changes.
3.  **Develop**: Write your code, following the guidelines in this document.
4.  **Test**: Add or update tests for your changes.
5.  **Format and Lint**: Run `cargo fmt` and `cargo clippy` to ensure your code meets the project's standards.
6.  **Pull Request**: Open a pull request, linking it to the original issue. Provide a clear description of the changes.
7.  **Code Review**: All pull requests must be reviewed and approved by at least one other contributor before being merged.

## 10. Final Verification Checklist

Before considering a task complete, always perform the following final checks:

1.  **Run All Tests**: Execute the full test suite (`cargo test`) to ensure that no existing functionality has been broken (regressions).
2.  **Update Documentation**: Review and update `README.md` to reflect any changes in functionality, command-line usage, or keyboard shortcuts. Ensure the documentation is clear and accurate for the end-user.
3.  **Self-Correction**: If any tests fail or documentation is outdated, prioritize fixing these issues before finishing the task. This ensures the project remains in a consistently stable and well-documented state.
