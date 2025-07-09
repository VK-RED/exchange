# Rust Crypto Exchange

This project is a high-performance cryptocurrency exchange built entirely in Rust. It features a modular architecture with separate components for the API, matching engine, WebSocket server, and database services.

## Project Structure

The exchange is a monorepo containing the following crates:

*   `api`: The public-facing RESTful API built with `actix-web`. It handles user requests, such as placing orders, checking balances, and getting market data.
*   `engine`: The core matching engine of the exchange. It processes orders, matches trades, and maintains the order book for each market. It's designed for high performance and low latency.
*   `wss`: A WebSocket server that provides real-time data streams to clients. Users can subscribe to channels to receive live updates on trades, order book changes, and their own user data.
*   `store`: Handles all database interactions using `sqlx` with a PostgreSQL database. It's responsible for persisting trades, orders, and user data.
*   `db_filler`: A utility to populate the database with initial data, such as creating markets or funding user accounts.
*   `common`: A shared library containing common data structures, types, and utilities used across all other crates.

## Getting Started

### Prerequisites

*   **Rust:** Install the latest stable version of Rust using `rustup`.
*   **PostgreSQL:** A relational database for storing persistent data.
*   **Redis:** An in-memory data store used for message passing between the different components of the exchange.

### How to Run

1.  **Set up the environment:**
    *   Clone the repository.
    *   Create a `.env` file in the root of the project and add the necessary environment variables for database connections, Redis URLs, and other configurations. You can use the `.env.example` file as a template. Or you can run `cp .env.example .env` for the default values.

2.  **Starting Services (via Docker):**
    *   Run `docker run -e POSTGRES_PASSWORD=postgres -p 5432:5432 postgres` to start a postgres database.
    *   Run `docker run -d -p 6379:6379 redis` to start a redis server.

3.  **Run DB migrations:**
    *   First of all, you need to have a `sqlx-cli` installed to run the database migrations.
    *   Navigate to the `store` directory `cd store`.
    *   Now run migrations using the command `sqlx migrate run`

4.  **Run the exchange components:**
    *   Each component can be run separately. Open a new terminal for each service.
    *   **Engine:** `cargo run --bin engine`
    *   **Database Filler:** `cargo run --bin db_filler`
    *   **WebSocket Server:** `cargo run --bin wss`
    *   **API Server:** `cargo run --bin api`


## API Endpoints

The following is a summary of the available API endpoints based on the code structure.

*   `GET /health`: Checks the health of the API server.
*   `POST /order`: Create a new order.
*   `DELETE /order`: Cancel an existing order.
*   `GET /orders/open`: Get all open orders for a user.
*   `POST /order/cancel_all`: Cancel all open orders for a user.
*   `GET /depth`: Get the order book depth for a market.
*   `GET /balance`: Get the user's account balance.
*   `GET /trade/history`: Get the trade history for a market.

## WebSocket API

The WebSocket server provides real-time data streams. Connect to `ws://127.0.0.1:8081` and subscribe to the following channels:

*   **Order Book:** Get real-time updates on the order book for a specific market.
*   **Trades:** Receive live trade updates for a market.

## Pending Features

*   **OrderBook Recovery**
*   **Testing**

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request 🙏.
