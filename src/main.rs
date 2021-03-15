use actix_web::{error::ResponseError, guard, middleware, web, App, HttpServer};
use rbm::{daemon, routes, Error, MysqlBackend};
use std::{io::Result, sync::Arc};
use structopt::StructOpt;
use url::Url;

#[derive(StructOpt)]
struct Opt {
  #[structopt(short, long, env = "PORT")]
  pub port: u16,
  #[structopt(long, env = "DATABASE_URL")]
  pub database_url: Url,
  #[structopt(long)]
  pub embed_migration: bool,
}

#[actix_web::main]
async fn main() -> Result<()> {
  env_logger::init();

  let opt = Opt::from_args();
  let backend = Arc::new(MysqlBackend::new(opt.database_url)?);

  if opt.embed_migration {
    backend.migrate()?;
  }

  let channel = daemon::start(backend.clone())?;

  HttpServer::new(move || {
    App::new()
      .wrap(middleware::Logger::default())
      .data(web::JsonConfig::default().limit(4096))
      .data(backend.clone())
      .data(channel.clone())
      .default_service(
        web::route()
          .guard(guard::Not(guard::Get()))
          .to(|| Error::UnknownRoute.error_response()),
      )
      .service(
        web::scope("/api")
          .service(routes::create_command)
          .service(routes::update_command)
          .service(routes::get_stalest_command)
          .service(routes::create_external_monitor)
          .service(routes::create_monitor),
      )
  })
  .bind(format!("127.0.0.1:{}", opt.port))?
  .run()
  .await
}
