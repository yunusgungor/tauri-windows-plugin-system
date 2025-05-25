# Coding Standards for Tauri Windows Plugin System

## General Principles
- Follow Rust idioms and best practices
- Adhere to Single Responsibility Principle (SRP)
- Keep functions small and focused (5-10 lines ideal, maximum 20 lines)
- Maintain modular files (generally under 100 lines)

## Rust-Specific Standards
- Use Rust 2021 edition
- Format code with rustfmt
- Follow the Rust API Guidelines for public interfaces
- Use meaningful error types with thiserror
- Implement comprehensive error handling
- Use strong typing and avoid unnecessary type conversions

## Documentation
- Document all public APIs with rustdoc
- Include examples in documentation
- Document error cases and handling
- Keep documentation up-to-date with code changes

## Testing
- Implement unit tests for all modules
- Use integration tests for cross-module functionality
- Aim for 100% test coverage
- Include test cases for error handling

## Plugin System Specific
- Maintain clean ABI boundaries
- Document all plugin interfaces thoroughly
- Implement proper error handling across ABI boundaries
- Ensure resources are properly cleaned up during plugin unloading
- Validate all input from plugins
- Implement proper permission checking

## Security
- Validate all external inputs
- Apply principle of least privilege
- Implement proper error handling without leaking sensitive information
- Review all dependencies for security vulnerabilities
- Implement proper sandboxing for plugins

## Performance
- Minimize memory allocations across ABI boundaries
- Avoid unnecessary cloning of data
- Use asynchronous operations where appropriate
- Profile and optimize critical paths

## Naming Conventions
- Use snake_case for functions, variables, modules, and packages
- Use PascalCase for types, traits, and enum variants
- Use SCREAMING_SNAKE_CASE for constants
- Use descriptive names that convey purpose and usage
