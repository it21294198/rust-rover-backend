## About this Rust Shuttle-Axum JWT enabled Static pages capable, Postgres DB connected project.

[More About Shuttle](https://www.shuttle.dev/)

1. In this project JWT token Auth has been configured.
2. Static web pages for `index.html` and `/auth/index.html` also been configured using HTMX [Read More](https://htmx.org/).
3. Postgres Database is used with Stored Procedures to access data.
4. To setup dev Postgres DB visit [Railway.app](https://railway.app/).
5. To view the Database and Schemas use [Dbeaver.io](https://dbeaver.io/).
6. Run `migration.sql` file on root folder to populate database with todo table only for CRUD operations to perform.
7. Before start, create `Secrets.toml` file in root directory and put secret variables init [Read More](https://docs.shuttle.rs/resources/shuttle-secrets#secrets).
```env
DB_CONNECTION = ''
REDIS_CONNECTION = ''
```
1. To check JWT (for this login values are hard coded IRL those needed to be taken from DB)
    ```bash
    curl -X POST http://127.0.0.1:8000/login \
        -H "Content-Type: application/json" \
        -d '{"client_id": "foo", "client_secret": "bar"}'
    ```
2. To run locally,
    ```bash
    cargo shuttle run
    ```
    or
    ```bash
    shuttle run
    ```
3.  Available static pages [Home Page](http://127.0.0.1:8000) and [Login Page](http://127.0.0.1:8000/auth/index.html).
4.  CORS are configured also.
5.  To deploy on Shuttle.dev,
```bash
shuttle deploy
```

### Finally Enjoy Rust