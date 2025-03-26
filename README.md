![Actix Web + SeaORM](assets/images/cover.png)

Rust Actix Web with SeaORM - REST API Boilerplate
=================================================

This repository provides a comprehensive boilerplate for building scalable and maintainable REST APIs using Rust with Actix Web framework and SeaORM as the ORM layer for PostgreSQL databases.

🚀 Overview
-----------

This project serves as a solid foundation for developers looking to create RESTful web services in Rust with a clean architecture. It implements a complete user management system including CRUD operations, authentication, role-based access control, and soft delete functionality while following best practices for structuring Rust web applications.

✨ Features
----------

- **Complete REST API**: Full implementation of user resource with CRUD operations
- **JWT Authentication**: Token-based authentication system
- **Role-Based Access Control**: Admin, User, and Guest role capabilities
- **Clean Architecture**: Well-organized code structure with separation of concerns
- **Password Security**: Secure password hashing and validation
- **Error Handling**: Robust error management with custom error types
- **Database Integration**: PostgreSQL support via SeaORM
- **Logical Deletion**: Soft delete implementation for users
- **Configuration Management**: Environment-based configuration with dotenv
- **Deployment Ready**: Simple deployment configuration for various platforms
- **Development Tools**: VSCode launch configurations for debugging
- **Input Validation**: Centralized validation rules for consistent data processing

🏗️ Project Structure
---------------------

```
rust-actix-seaorm/
├── .env                           # Environment variables
├── .gitignore                     # Git ignore file
├── Cargo.lock                     # Rust dependency lock file
├── Cargo.toml                     # Rust project configuration
├── .vscode/                       # VSCode configuration
│   └── launch.json                # Debugging configuration
├── src/                           # Source code
│   ├── main.rs                    # Application entry point
│   ├── api/                       # API endpoints and route handlers
│   │   ├── mod.rs                 # API module exports
│   │   ├── users.rs               # User API handlers
│   │   └── auth.rs                # Authentication handlers
│   ├── auth/                      # Authentication components
│   │   ├── mod.rs                 # Auth module exports  
│   │   ├── jwt.rs                 # JWT token generation and validation
│   │   └── password.rs            # Password hashing and verification
│   ├── config/                    # Configuration management
│   │   ├── app_config.rs          # Application configuration
│   │   └── mod.rs                 # Config module exports
│   ├── db/                        # Database layer
│   │   ├── mod.rs                 # Database module exports
│   │   ├── migrations/            # Database migrations
│   │   ├── models/                # SeaORM entity models
│   │   └── repositories/          # Data access repositories
│   ├── validators/                # Input validation rules
│   │   ├── mod.rs                 # Validators module exports
│   │   └── user_validators.rs     # User-specific validation rules
│   ├── utils/                     # Utility functions
│   │   ├── mod.rs                 # Utils module exports
│   │   └── validation.rs          # Shared validation helpers
│   └── error/                     # Error handling
│       ├── app_error.rs           # Custom application error types
│       └── mod.rs                 # Error module exports
└── target/                        # Compiled output (generated)
```

📚 Key Components
-----------------

### API Layer ([api](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-sandbox/workbench/workbench.html))

Contains HTTP route handlers and request/response models. This layer is responsible for:

- Parsing incoming HTTP requests
- Validating request data
- Calling appropriate domain logic
- Formatting HTTP responses

Key files include:
- `users.rs`: Defines endpoints for user management
- `auth.rs`: Implements authentication and JWT token management

### Authentication Layer ([auth](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-sandbox/workbench/workbench.html))

Handles all aspects related to authentication and security:

- **JWT**: Token generation and verification
- **Password Security**: Secure password hashing and verification
- **Role Management**: Role-based access guards implementation
- **Middleware**: Middleware for protecting authenticated routes

### Configuration ([config](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-sandbox/workbench/workbench.html))

Manages application settings loaded from environment variables:

- Database connection strings
- Server configuration
- JWT secrets
- Environment-specific settings

### Database Layer ([db](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-sandbox/workbench/workbench.html))

Contains everything related to data persistence:

- **Migrations**: Database schema evolution
- **Models**: SeaORM entity definitions that map to database tables
- **Repositories**: Implements data access patterns to abstract database operations

### Validators Layer ([validators](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-sandbox/workbench/workbench.html))

Contains reusable validation logic:

- **Regular expressions**: For format validation (phone numbers, passwords, etc.)
- **Custom validators**: Functions for complex validation rules
- **Shared validation logic**: Common validation patterns used across the application

The validators provide a centralized location for all validation rules, ensuring consistency across the application and making it easier to update validation logic in one place.

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

#### 1. Clone the repository:
```bash
git clone https://github.com/gabrielrmunoz/rust-actix-seaorm.git
cd rust-actix-seaorm
```

#### 2. Create a [.env](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-sandbox/workbench/workbench.html) file based on the example:
```
DATABASE_URL=postgres://username:password@localhost:5432/dbname
SERVER_HOST=127.0.0.1
SERVER_PORT=8000
RUST_LOG=info
JWT_SECRET=your_secret_key_here
```

#### 3. Setup the database:

##### Using Docker (optional)
```bash
docker run --name postgres -e POSTGRES_PASSWORD=password -p 5432:5432 -d postgres
```

##### Create the database
```bash
psql -U postgres -c "CREATE DATABASE dbname;"
```

### Running the Application

#### Development mode with auto-reload (requires cargo-watch)
```bash
cargo watch -x run
```

#### Standard run
```bash
cargo run
```

#### Production build
```bash
cargo build --release
```

### Running Tests

#### Run all tests
```bash
cargo test
```

#### Run tests with output
```bash
cargo test -- --nocapture
```

🔄 API Endpoints
----------------

### Authentication

| Method | Endpoint | Description | Authentication |
| --- | --- | --- | --- |
| POST | /api/auth/login | Login and get JWT token | No |
| POST | /api/register | Register new user | No |

### User Management

| Method | Endpoint | Description | Authentication | Role Required |
| --- | --- | --- | --- | --- |
| GET | /api/users | Get all users | Yes | User or Admin |
| GET | /api/users/{id} | Get user by ID | Yes | User or Admin |
| PUT | /api/users/{id} | Update a user | Yes | User or Admin |
| DELETE | /api/users/{id} | Physically delete a user | Yes | Admin |
| PATCH | /api/users/{id}/soft-delete | Soft delete a user | Yes | Admin |
| PATCH | /api/users/{id}/restore | Restore a soft deleted user | Yes | Admin |

📋 Data Models
--------------

### User Model

```rust
struct User {
    id: i32,
    username: String,
    password: String, // Stored as hash, never returned in API responses
    first_name: String,
    last_name: String,
    email: String,
    phone: String,
    role: String, // "admin", "user", "guest"
    created_on: NaiveDateTime,
    updated_on: NaiveDateTime,
    deleted_on: Option<NaiveDateTime>,
}
```

### Login Request/Response

```rust
// Request
struct LoginRequest {
    username: String,
    password: String,
}

// Response
struct LoginResponse {
    token: String,
    user_id: i32,
    username: String,
    role: String,
}
```

### Validation Rules

The system includes multiple validators to ensure data integrity:

```rust
// Password validation
// - 10-20 characters
// - At least one lowercase letter
// - At least one uppercase letter
// - At least one number
// - At least one special character (@$!%*?&_-)

// Phone number validation
static PHONE_REGEX: Lazy<Regex> = 
    Lazy::new(|| Regex::new(r"^(\+\d{1,3})?-\d{6,14}$").unwrap());

// Username validation (no spaces allowed)
fn validate_no_spaces(username: &str) -> Result<(), ValidationError> {
    if username.contains(' ') {
        // Return validation error
    }
    Ok(())
}

// Role validation
fn validate_role(role: &str) -> Result<(), ValidationError> {
    if UserRole::is_valid_role(role) {
        Ok(())
    } else {
        // Return validation error
    }
}
```

## 🧩 Security Architecture

### JWT Authentication

The system implements JWT token-based authentication:
- Tokens signed with a configurable secret
- Role information included in claims
- Middleware for automatic token validation
- Role-based guards for access control

### Password Security

Passwords are handled securely:
- Stored in database using secure hashing algorithms
- Never returned in API responses 
- Robust validation to ensure strong passwords

🧩 Architecture
---------------

This project follows a layered architecture pattern:

- **HTTP Layer** (API): Handles incoming requests and outgoing responses
- **Auth Layer**: Provides authentication and authorization services
- **Service Layer** (Domain): Contains business logic
- **Data Access Layer** (Repositories): Abstracts database operations
- **Database Layer** (SeaORM Entities): Represents database tables
- **Validation Layer**: Centralizes input validation rules

📦 Dependencies
---------------

Major dependencies include:

- **actix-web**: Web framework for handling HTTP requests
- **jsonwebtoken**: JWT implementation for Rust
- **sea-orm**: Async ORM for Rust
- **sqlx**: SQL toolkit with compile-time checked queries
- **tokio**: Async runtime
- **serde**: Serialization/deserialization framework
- **validator**: Input validation framework
- **dotenv**: Environment variable loading
- **log**: Logging infrastructure
- **chrono**: Date and time utilities
- **argon2**: Secure hashing algorithm for passwords

🚢 Deployment
-------------

### Manual Deployment

For manual deployment, build a release binary:
```bash
cargo build --release
```

The binary will be available at [rust-actix-seaorm](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-sandbox/workbench/workbench.html).

🔍 Development Tools
--------------------

### VSCode Configuration

The repository includes VSCode launch configurations for debugging the application:

- **Launch Server**: Runs the application with debugger attached
- **Run Tests**: Runs tests with debugger

🤝 Contributing
---------------

Contributions are welcome! Please feel free to submit a Pull Request.

- Fork the project
- Create your feature branch (`git checkout -b feature/amazing-feature`)
- Commit your changes (`git commit -m 'Add some amazing feature'`)
- Push to the branch (`git push origin feature/amazing-feature`)
- Open a Pull Request

📄 License
----------

This project is licensed under the MIT License - see the LICENSE file for details.

📞 Contact
----------

If you have any questions or suggestions about this project, please open an issue in the repository.

---

*This boilerplate was created to provide a solid foundation for Rust web applications with a focus on maintainability and best practices. Happy coding!*
