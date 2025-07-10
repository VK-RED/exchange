# Rust Crypto Exchange

This project is a high-performance cryptocurrency exchange built entirely in Rust. It features a modular architecture with separate components for the API, matching engine, WebSocket server, and database services.

## Project Structure

The exchange is a monorepo containing the following crates:

*   `api`: The public-facing RESTful API built with `actix-web`. It handles user requests, such as placing orders, checking balances, and getting market data.
*   `engine`: The core matching engine of the exchange. It processes orders, matches trades, and maintains the order book for each market. It's designed for high performance and low latency.
*   `wss`: A WebSocket server built with `tokio-tungstentine` that provides real-time data streams to clients. Users can subscribe to channels to receive live updates on trades, order book changes, and their own user data.
*   `db_filler`: Fills the market data such as place new orders, updating orders, cancelling orders and adding trades by getting messages from engine. Uses `tokio` as the runtime.
*   `store`: Handles all database interactions using `sqlx` with a PostgreSQL database. It's responsible for persisting trades, orders, and user data.
*   `common`: A shared library containing common data structures, types, and utilities used across all other crates.

## Getting Started

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

#### Screencast
https://github.com/user-attachments/assets/e69883c6-62d6-4534-aa59-e7369253c2a0

## Architecture

### NOTE: Each orderbook runs individually on separate thread !

#### Order Placing :

*   Initially client subscribes to the WSS server at channels like trade@SOL_USDC , depth@SOL_USDC. Then WSS subscribes to those channels on `WSS_PUB_SUB`.
*   Then client sends a POST req to create an order to `api`
*   `api` creates an order_id for the order. Then subscribes to that `order_id` on `API_PUB_SUB` and pushes the order to `orders QUEUE`.
*   `manager` or the main core of the engine constantly get's order from `orders QUEUE` and sends the order to the correct orderbook.
*   `Orderbook` validates and locks user funds. Then process against opposing orders and may sit on the orderbook if unfilled incase of limit order. Finally Settles the balance of makers and taker.
*   Then `Orderbook` sends:
     - Order and Trade details to `DB_Filler QUEUE`,
     - Order Status and Filled Quantity of Order to `API_PUB_SUB`.
     - Trade and Depth details to `WSS_PUB_SUB`.
*   `api` receives the order details from `API_PUB_SUB` and sends response to the client.
*   `DB_Filler` then gets the trade and order details and updates to the DB
*   `WSS server` gets updates about trade and depth details to the Subscribed Clients

<img width="1917" height="883" alt="Image" src="https://github.com/user-attachments/assets/9aab8a13-e5bb-4c3a-96cc-17606c9d32b5" />

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

#### Note
* UserId's - random1, random2, random32 are set with initial balances for ease.

## Pending Features

*   **OrderBook Recovery**
*   **Testing**

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request 🙏.