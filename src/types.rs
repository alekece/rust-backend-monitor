use crate::{
  schema::{commands, external_monitors, monitors, status},
  Error,
};
use chrono::NaiveDateTime;
use diesel::{
  deserialize::{self, FromSql},
  mysql::Mysql,
  serialize::{self, IsNull, Output, ToSql},
  sql_types::Text,
};
use serde::{Deserialize, Serialize};
use std::{fmt, io::Write, net::IpAddr};

#[derive(Deserialize, Insertable)]
#[table_name = "status"]
pub struct NewStatus {
  pub monitor_id: u64,
  pub succeed: bool,
  pub result: String,
  pub response_time_ms: u32,
}

#[derive(Deserialize, Insertable)]
#[table_name = "commands"]
#[serde(deny_unknown_fields)]
pub struct NewCommand {
  pub serial: String,
}

#[derive(Queryable, Serialize)]
pub struct Command {
  pub id: u64,
  pub serial: String,
  #[serde(skip_serializing)]
  pub completed_at: Option<NaiveDateTime>,
}

#[derive(Deserialize, Insertable)]
#[table_name = "external_monitors"]
#[serde(deny_unknown_fields)]
pub struct NewExternalMonitor {
  pub serial: String,
  pub cpu_usage: u8,
  pub memory_usage: u8,
  pub disk_usage: u8,
  pub status: Option<String>,
}

#[derive(Deserialize, Serialize, Queryable)]
pub struct ExternalMonitor {
  pub id: u64,
  pub created_at: NaiveDateTime,
  pub serial: String,
  pub cpu_usage: u8,
  pub memory_usage: u8,
  pub disk_usage: u8,
  pub status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, FromSqlRow, AsExpression, Clone)]
#[sql_type = "Text"]
#[serde(rename_all = "UPPERCASE")]
pub enum MonitorType {
  Http,
  Https,
  Ssl,
  Ping,
  Dns,
}

impl fmt::Display for MonitorType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      Self::Http => write!(f, "HTTP"),
      Self::Https => write!(f, "HTTPS"),
      Self::Ssl => write!(f, "SSL"),
      Self::Ping => write!(f, "PING"),
      Self::Dns => write!(f, "DNS"),
    }
  }
}

impl ToSql<Text, Mysql> for MonitorType {
  fn to_sql<W: Write>(&self, out: &mut Output<W, Mysql>) -> serialize::Result {
    match *self {
      Self::Http => out.write_all(b"HTTP")?,
      Self::Https => out.write_all(b"HTTPS")?,
      Self::Ssl => out.write_all(b"SSL")?,
      Self::Ping => out.write_all(b"PING")?,
      Self::Dns => out.write_all(b"DNS")?,
    }
    Ok(IsNull::No)
  }
}

impl FromSql<Text, Mysql> for MonitorType {
  fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
    match not_none!(bytes) {
      b"HTTP" => Ok(Self::Http),
      b"HTTPS" => Ok(Self::Https),
      b"SSL" => Ok(Self::Ssl),
      b"PING" => Ok(Self::Ping),
      b"DNS" => Ok(Self::Dns),
      _ => Err("Unrecognized monitor type".into()),
    }
  }
}

#[derive(Deserialize, Insertable)]
#[table_name = "monitors"]
#[serde(deny_unknown_fields)]
pub struct NewMonitor {
  #[serde(rename = "type")]
  pub type_: MonitorType,
  pub frequency_min: u16,
  pub endpoint: String,
  pub max_latency_ms: Option<u32>,
  pub expected_ip: Option<String>,
  pub minimum_expiration_time_d: Option<u32>,
}

impl NewMonitor {
  pub fn validate(&self) -> Result<(), Error> {
    match self.type_ {
      MonitorType::Ping => self
        .max_latency_ms
        .map(|_| ())
        .ok_or_else(|| Error::InvalidMonitoring(String::from("missing 'max_latency_ms'")))
        .and_then(|_| {
          self
            .endpoint
            .parse::<IpAddr>()
            .map_err(|_| Error::InvalidMonitoring(String::from("'endpoint' must be a valid IP")))
            .map(|_| ())
        }),
      MonitorType::Ssl => self.minimum_expiration_time_d.map(|_| ()).ok_or_else(|| {
        Error::InvalidMonitoring(String::from("missing 'minimum_expiration_time_d'"))
      }),
      MonitorType::Dns => self
        .expected_ip
        .clone()
        .map(|_| ())
        .ok_or_else(|| Error::InvalidMonitoring(String::from("missing 'expected_id'"))),
      _ => Ok(()),
    }
  }
}

#[derive(Deserialize, Serialize, Debug, Queryable, Clone)]
pub struct Monitor {
  pub id: u64,
  #[serde(rename = "type")]
  pub type_: MonitorType,
  pub frequency_min: u16,
  pub endpoint: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_latency_ms: Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub expected_ip: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub minimum_expiration_time_d: Option<u32>,
}
