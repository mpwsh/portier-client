# Description

You'll need to run the backend server [actix-portier](https://github.com/mpwsh/actix-portier) in order to make this library work.
In there you'll find a `docker-compose.yml` file to spin the required infrastructure (portier,redis, mailcrab)

## Try out the example

```bash
git clone https://github.com/mpwsh/actix-portier && cd actix-portier
docker-compose up -d
cd ../
git clone https://github.com/mpwsh/portier-client && cd portier-client
cargo run --example login
```
