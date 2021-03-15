use crate::{daemon::Channel, types::*, Error, MysqlBackend};
use actix_web::{get, post, put, web, HttpResponse, Responder};
use chrono::Utc;
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CommandParams {
  pub serial: String,
}

#[post("/commands")]
pub async fn create_command(
  new_command: web::Json<NewCommand>,
  backend: web::Data<Arc<MysqlBackend>>,
) -> Result<impl Responder, Error> {
  use crate::schema::commands::dsl;

  diesel::insert_into(dsl::commands)
    .values(&*new_command)
    .execute(&backend.get_connection()?)?;

  let command = dsl::commands
    .order_by(dsl::id.desc())
    .first::<Command>(&backend.get_connection()?)?;

  Ok(HttpResponse::Created().json(command))
}

#[get("/commands")]
pub async fn get_stalest_command(
  params: web::Query<CommandParams>,
  backend: web::Data<Arc<MysqlBackend>>,
) -> Result<impl Responder, Error> {
  use crate::schema::commands::dsl;

  let command = dsl::commands
    .filter(dsl::serial.eq(&params.serial))
    .filter(dsl::completed_at.is_null())
    .order_by(dsl::id.asc())
    .first::<Command>(&backend.get_connection()?)
    .map_err(|_| Error::NotFound)?;

  Ok(HttpResponse::Ok().json(command))
}

#[put("/commands/{id}/done")]
pub async fn update_command(
  web::Path(id): web::Path<u64>,
  backend: web::Data<Arc<MysqlBackend>>,
) -> Result<impl Responder, Error> {
  use crate::schema::commands::dsl;
  let conn = backend.get_connection()?;

  let command_entry = dsl::commands.find(id);
  let command = command_entry
    .get_result::<Command>(&conn)
    .map_err(|_| Error::NotFound)?;

  if command.completed_at.is_some() {
    return Err(Error::CommandAlreadyCompleted);
  }

  diesel::update(command_entry)
    .set(dsl::completed_at.eq(Utc::now().naive_utc()))
    .execute(&backend.get_connection()?)?;

  Ok(HttpResponse::NoContent())
}

#[post("/external-monitors")]
pub async fn create_external_monitor(
  new_external_monitor: web::Json<NewExternalMonitor>,
  backend: web::Data<Arc<MysqlBackend>>,
) -> Result<impl Responder, Error> {
  use crate::schema::external_monitors::dsl;

  diesel::insert_into(dsl::external_monitors)
    .values(&*new_external_monitor)
    .execute(&backend.get_connection()?)?;

  let external_monitor = dsl::external_monitors
    .order_by(dsl::id.desc())
    .first::<ExternalMonitor>(&backend.get_connection()?)?;

  Ok(HttpResponse::Created().json(external_monitor))
}

#[post("/monitors")]
pub async fn create_monitor(
  new_monitor: web::Json<NewMonitor>,
  backend: web::Data<Arc<MysqlBackend>>,
  channel: web::Data<Channel>,
) -> Result<impl Responder, Error> {
  use crate::schema::monitors::dsl;

  new_monitor.validate()?;

  diesel::insert_into(dsl::monitors)
    .values(&*new_monitor)
    .execute(&backend.get_connection()?)?;

  let monitor = dsl::monitors
    .order_by(dsl::id.desc())
    .first::<Monitor>(&backend.get_connection()?)?;

  channel.send(monitor.clone()).unwrap();

  Ok(HttpResponse::Created().json(monitor))
}
