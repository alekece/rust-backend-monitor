# rust-backend-monitor

`rbm` is developed and implemented using [Rust](https://www.rust-lang.org/) and uses [MySQL](https://www.mysql.com/) database as a backend.  

## Setup environment

A `.env.example` at the root directory exposes environment both used by [diesel](https://diesel.rs/) and `rbm` itself.  
Rename it to `.env` then set all the environment variables before running the following commands :

``` sh
source .env
```

## Build within a docker container

Simply run `docker-compose up -d` and that's it !

## Build locally

### Requirements

Run the following command to fulfill the requirements :

``` sh
# install MySQL dependencies
apt install libmysqlclient-dev
# install rust and cargo alongside rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# install ORM and query builder
cargo install diesel_cli --no-default-features --features mysql
```

### Setup MySQL database


``` sh
# start MySQL server
systemctl start mysql
# wait until the service is up then create the setup the database
diesel setup
```

### Launch `rbm` locally

1. build the sources

    ``` sh
    cargo build --release
    # fastping-rs requires the ability to create raw sockets
    sudo setcap cap_net_raw=eip ./target/release/rbm
    ```

    If the build fails, make sure to use an up-to-date rust version

    ``` sh
    rustup update
    ```

2. Launch `rbm` by running :

    ``` sh
    cargo run --release -- <arguments>
    ```

    Or by executing the `rbm` binary directly :

    ``` sh
    ./target/release/rbm <arguments>
    ```

To print the `rbm` usage, an option `-h` is available.

 
## Usage

```
rbm 0.1.0

USAGE:
    rbm [FLAGS] --database-url <database-url> --port <port>

FLAGS:
    -h, --help               Prints help information
    -V, --version            Prints version information

OPTIONS:
        --database-url <database-url>     [env: DATABASE_URL=]
    -p, --port <port>                     [env: PORT=]
```

## API routes

| Method | Path                          | Request                               | Response                              | Code |
|--------|-------------------------------|---------------------------------------|---------------------------------------|------|
| POST   | /api/commands                 | `{`                                   | `{`                                   | 201  |
|        |                               | `"serial" : string`                   | `"id" : u64,`                         |      |
|        |                               | `}`                                   | `"serial" : string,`                  |      |
|        |                               |                                       | `}`                                   |      |
| PUT    | /api/commands/{id}/done       |                                       |                                       | 204  |
| GET    | /api/commands?serial={serial} |                                       | `{`                                   | 200  |
|        |                               |                                       | `"id" : u64,`                         |      |
|        |                               |                                       | `"serial" : string,`                  |      |
|        |                               |                                       | `}`                                   |      |
| POST   | /api/external_monitors        | `{`                                   | `{`                                   | 201  |
|        |                               | `"serial" : string,`                  | `"id" : u64,`                         |      |
|        |                               | `"cpu_usage" : u8,`                   | `"created_at" : datetime,"`           |      |
|        |                               | `"memory_usage" : u8,`                | `"serial" : string,`                  |      |
|        |                               | `"disk_usage" : u8,`                  | `"cpu_usage" : u8,`                   |      |
|        |                               | `"status" : string`                   | `"memory_usage" : u8,`                |      |
|        |                               | `}`                                   | `"disk_usage" : u8,`                  |      |
|        |                               |                                       | `"status" : string`                   |      |
|        |                               |                                       | `}`                                   |      |
| POST   | /api/monitors                 | `{`                                   | `{`                                   | 201  |
|        |                               | `"type" : "HTTP,HTTPS,SSL,PING,DNS",` | `"id" : u64,`                         |      |
|        |                               | `"frequency_min" : u16,`              | `"type" : "HTTP,HTTPS,SSL,PING,DNS",` |      |
|        |                               | `"endpoint" : string,`                | `"frequency_min" : u16,`              |      |
|        |                               | `"minimum_expiration_time_d" : u32`   | `"endpoint" : string,`                |      |
|        |                               | `"max_latency_ms" : u32`              | `"minimum_expiration_time_d" : u32`   |      |
|        |                               | `"expected_ip" : string`              | `"max_latency_ms" : u32`              |      |
|        |                               | `}`                                   | `"expected_ip" : string`              |      |
|        |                               |                                       | `}`                                   |      |

An error may occurred when calling this API routes then the HTTP response  
will be a JSON containing the reason of such a failure :

``` json
{
    "code": u16,
    "error": string,
    "message": string
}
```
