Rust Actix Web with SeaORM - REST API Boilerplate
=================================================

This repository provides a comprehensive boilerplate for building scalable and maintainable REST APIs using Rust with Actix Web framework and SeaORM as the ORM layer for PostgreSQL databases.

🚀 Overview
-----------

This project serves as a solid foundation for developers looking to create RESTful web services in Rust with a clean architecture. It implements a complete user management system including CRUD operations and soft delete functionality while following best practices for structuring Rust web applications.

✨ Features
----------

-   **Complete REST API**: Full implementation of user resource with CRUD operations
-   **Clean Architecture**: Well-organized code structure with separation of concerns
-   **Error Handling**: Robust error management with custom error types
-   **Database Integration**: PostgreSQL support via SeaORM
-   **Configuration Management**: Environment-based configuration with dotenv
-   **Deployment Ready**: Shuttle.rs deployment configuration included
-   **Development Tools**: VSCode launch configurations for debugging

🏗️ Project Structure
---------------------

rust-actix-seaorm/

├── .env                           # Environment variables

├── .gitignore                     # Git ignore file

├── Cargo.lock                     # Rust dependency lock file

├── Cargo.toml                     # Rust project configuration

├── .shuttle/                      # Shuttle.rs deployment configuration

│   └── config.toml                # Shuttle-specific configuration

├── .vscode/                       # VSCode configuration

│   └── launch.json                # Debugging configuration

├── src/                           # Source code

│   ├── main.rs                    # Application entry point

│   ├── api/                       # API endpoints and route handlers

│   │   ├── mod.rs                 # API module exports

│   │   └── users.rs               # User API handlers

│   ├── config/                    # Configuration management

│   │   ├── app_config.rs          # Application configuration

│   │   └── mod.rs                 # Config module exports

│   ├── db/                        # Database layer

│   │   ├── mod.rs                 # Database module exports

│   │   ├── migrations/            # Database migrations

│   │   ├── models/                # SeaORM entity models

│   │   └── repositories/          # Data access repositories

│   ├── domain/                    # Domain models and business logic

│   │   ├── mod.rs                 # Domain module exports

│   │   └── user.rs                # User domain model

│   └── error/                     # Error handling

│       ├── app_error.rs           # Custom application error types

│       └── mod.rs                 # Error module exports

└── target/                        # Compiled output (generated)

📚 Key Components
-----------------

### API Layer ([api](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-sandbox/workbench/workbench.html))

Contains HTTP route handlers and request/response models. This layer is responsible for:

-   Parsing incoming HTTP requests
-   Validating request data
-   Calling appropriate domain logic
-   Formatting HTTP responses

The `users.rs` file defines endpoints for user management, including:

-   Getting all users
-   Getting a single user
-   Creating users
-   Updating users
-   Physical deletion of users
-   Logical (soft) deletion of users
-   Restoring soft-deleted users

### Configuration ([config](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-sandbox/workbench/workbench.html))

Manages application settings loaded from environment variables:

-   Database connection strings
-   Server configuration
-   Feature flags
-   Environment-specific settings

### Database Layer ([db](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-sandbox/workbench/workbench.html))

Contains everything related to data persistence:

-   **Migrations**: Database schema evolution
-   **Models**: SeaORM entity definitions that map to database tables
-   **Repositories**: Implements data access patterns to abstract database operations

### Domain Layer ([domain](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-sandbox/workbench/workbench.html))

Core business logic and domain models:

-   Business rules
-   Domain entity definitions
-   Service implementations
-   Domain events

### Error Handling ([error](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-sandbox/workbench/workbench.html))

Custom error types and error handling logic:

-   `AppError`: Custom error type with variants for different error categories
-   Conversion traits for mapping between different error types
-   Error responses formatting

🛠️ Getting Started
-------------------

### Prerequisites

-   Rust 1.70+ (stable)
-   PostgreSQL 13+
-   Docker (optional, for containerized PostgreSQL)

### Environment Setup

1.  Clone the repository:

-   git clone https://github.com/gabrielrmunoz/rust-actix-seaorm.git
-   cd rust-actix-seaorm

2.  Create a [.env](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-sandbox/workbench/workbench.html) file based on the example:

DATABASE_URL=postgres://username:password@localhost:5432/dbname
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
RUST_LOG=info

3.  Setup the database:

# Using Docker (optional)
docker run --name postgres -e POSTGRES_PASSWORD=password -p 5432:5432 -d postgres

# Create the database
psql -U postgres -c "CREATE DATABASE dbname;"

### Running the Application

# Development mode with auto-reload (requires cargo-watch)

cargo watch -x run

# Standard run

shuttle run

# Production build

cargo build --release

### Running Tests

# Run all tests

cargo test

# Run tests with output

cargo test -- --nocapture

🔄 API Endpoints
----------------

### User Management

| Method | Endpoint | Description |
| --- | --- | --- |
| GET | /api/users | Get all users |
| GET | /api/users/{id} | Get user by ID |
| POST | /api/users | Create a new user |
| PUT | /api/users/{id} | Update a user |
| DELETE | /api/users/{id} | Physically delete a user |
| PATCH | /api/users/{id}/soft-delete | Soft delete a user |
| PATCH | /api/users/{id}/restore | Restore a soft deleted user |

📋 Data Models
--------------

### User Model

struct User {
    id: i32,
    username: String,
    first_name: Option<String>,
    last_name: Option<String>,
    email: String,
    phone: Option<String>,
    created_on: NaiveDateTime,
    updated_on: NaiveDateTime,
    deleted_on: Option<NaiveDateTime>,
}

🧩 Architecture
---------------

This project follows a layered architecture pattern:

1.  **HTTP Layer** (API): Handles incoming requests and outgoing responses
2.  **Service Layer** (Domain): Contains business logic
3.  **Data Access Layer** (Repositories): Abstracts database operations
4.  **Database Layer** (SeaORM Entities): Represents database tables

📦 Dependencies
---------------

Major dependencies include:

-   **actix-web**: Web framework for handling HTTP requests
-   **sea-orm**: Async ORM for Rust
-   **sqlx**: SQL toolkit with compile-time checked queries
-   **tokio**: Async runtime
-   **serde**: Serialization/deserialization framework
-   **dotenv**: Environment variable loading
-   **log** and **env_logger**: Logging infrastructure
-   **chrono**: Date and time utilities

🚢 Deployment
-------------

### Using Shuttle.rs

The project includes configuration for deployment with Shuttle.rs:

# Install Shuttle CLI

cargo install cargo-shuttle

# Deploy your application

cargo shuttle deploy

### Manual Deployment

For manual deployment, build a release binary:

cargo build --release

The binary will be available at [rust-actix-seaorm](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-sandbox/workbench/workbench.html).

🔍 Development Tools
--------------------

### VSCode Configuration

The repository includes VSCode launch configurations for debugging the application:

-   **Launch Server**: Runs the application with debugger attached
-   **Run Tests**: Runs tests with debugger

🤝 Contributing
---------------

Contributions are welcome! Please feel free to submit a Pull Request.

1.  Fork the project
2.  Create your feature branch (`git checkout -b feature/amazing-feature`)
3.  Commit your changes (`git commit -m 'Add some amazing feature'`)
4.  Push to the branch (`git push origin feature/amazing-feature`)
5.  Open a Pull Request

📄 License
----------

This project is licensed under the MIT License - see the LICENSE file for details.

📞 Contact
----------

If you have any questions or suggestions about this project, please open an issue in the repository.

* * * * *

*This boilerplate was created to provide a solid foundation for Rust web applications with a focus on maintainability and best practices. Happy coding!*

*\
*