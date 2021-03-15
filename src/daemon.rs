use crate::{
  types::{Monitor, MonitorType, NewStatus},
  Error, MysqlBackend,
};
use clokwerk::{Interval, Scheduler};
use diesel::prelude::*;
use fastping_rs::{
  PingResult::{Idle, Receive},
  Pinger,
};
use log::*;
use ssl_expiration::SslExpiration;
use std::{
  string::ToString,
  sync::{
    mpsc::{self, Sender},
    Arc,
  },
  time::{Duration, Instant},
};

pub type Channel = Sender<Monitor>;

pub fn start(backend: Arc<MysqlBackend>) -> Result<Channel, Error> {
  use crate::schema::monitors::dsl;

  info!("Starting daemon");

  let mut scheduler = Scheduler::new();
  let (sender, receiver) = mpsc::channel();

  for monitor in dsl::monitors.load::<Monitor>(&backend.get_connection()?)? {
    schedule_monitoring(monitor, &mut scheduler, backend.clone());
  }

  actix::spawn(async move {
    loop {
      scheduler.run_pending();

      while let Ok(monitor) = receiver.try_recv() {
        schedule_monitoring(monitor, &mut scheduler, backend.clone());
      }

      actix::clock::delay_for(Duration::from_secs(1)).await;
    }
  });

  Ok(sender)
}

fn schedule_monitoring(monitor: Monitor, scheduler: &mut Scheduler, backend: Arc<MysqlBackend>) {
  info!(
    "Scheduling {} {} monitoring every {} minute(s)",
    monitor.type_, monitor.endpoint, monitor.frequency_min
  );

  scheduler
    .every(Interval::Minutes(monitor.frequency_min as u32))
    .run(move || {
      use crate::schema::status::dsl;

      info!("Starting {}-{} monitoring", monitor.type_, monitor.endpoint);

      let start_time = Instant::now();

      let (succeed, result) = match monitor.type_ {
        MonitorType::Http | MonitorType::Https => reqwest::blocking::get(&monitor.endpoint)
          .map(|response| {
            let response_status = response.status();

            (response_status.is_success(), response_status.to_string())
          })
          .unwrap_or_else(|e| (false, format!("Could not reach endpoint: {}", e))),
        MonitorType::Ssl => SslExpiration::from_domain_name(&monitor.endpoint)
          .map(|certificate| {
            let expiration_time = certificate.days();

            (
              expiration_time >= monitor.minimum_expiration_time_d.unwrap() as i32,
              expiration_time.to_string(),
            )
          })
          .unwrap_or_else(|e| {
            (
              false,
              format!("Could not check SSL certificate expiration: {}", e),
            )
          }),

        MonitorType::Ping => Pinger::new(None, Some(56))
          .and_then(|(pinger, results)| {
            pinger.add_ipaddr(monitor.endpoint.as_str());
            pinger.ping_once();

            results
              .recv()
              .map_err(|e| e.to_string())
              .and_then(|result| match result {
                Idle { .. } => Err(String::from("Idle address")),
                Receive { rtt, .. } => Ok((
                  rtt < Duration::from_millis(monitor.max_latency_ms.unwrap() as u64),
                  rtt.as_millis().to_string(),
                )),
              })
          })
          .unwrap_or_else(|e| (false, format!("Could not ping endpoint: {}", e))),
        MonitorType::Dns => dns_lookup::lookup_host(&monitor.endpoint)
          .map(|ips| ips.into_iter().map(|ip| ip.to_string()).collect::<Vec<_>>())
          .map(|ips| {
            (
              ips.contains(monitor.expected_ip.as_ref().unwrap()),
              ips.join(", "),
            )
          })
          .unwrap_or_else(|e| (false, format!("Could not lookup on endpoint: {}", e))),
      };

      let response_time_ms = start_time.elapsed().as_millis() as u32;

      let status = NewStatus {
        monitor_id: monitor.id,
        result: result.clone(),
        response_time_ms,
        succeed,
      };

      info!(
        "Monitoring {} {} {} in {} ms: Result {}",
        monitor.type_,
        monitor.endpoint,
        if succeed { "succeed" } else { "failed" },
        response_time_ms,
        result
      );

      if let Err(e) = backend.get_connection().and_then(|conn| {
        Ok(
          diesel::insert_into(dsl::status)
            .values(&status)
            .execute(&conn)?,
        )
      }) {
        warn!("Could not store monitoring status: {}", e);
      }
    });
}
