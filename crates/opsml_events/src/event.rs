use futures::Stream;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use opsml_types::contracts::{AuditEvent, SpaceStatsEvent};

use crate::error::EventError;
use opsml_sql::{base::SqlClient, enums::client::SqlClientEnum};

use tracing::{debug, instrument};

use crate::types::Event;
use std::sync::Arc;
use tracing::error;

#[instrument(skip_all)]
pub async fn log_audit_event(
    event: AuditEvent,
    sql_client: Arc<SqlClientEnum>,
) -> Result<(), EventError> {
    debug!("Logging audit event");

    sql_client.insert_audit_event(event).await.map_err(|e| {
        error!("Failed to log audit event: {}", e);
        EventError::LogEventError(e)
    })?;

    Ok(())
}

#[instrument(skip_all)]
pub async fn space_stats_event(
    event: SpaceStatsEvent,
    sql_client: Arc<SqlClientEnum>,
) -> Result<(), EventError> {
    debug!("Logging space stats event");

    sql_client.update_space_stats(event).await.map_err(|e| {
        error!("Failed to log space stats event: {}", e);
        EventError::LogEventError(e)
    })?;

    Ok(())
}

#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<Event>,
}

impl EventBus {
    #[instrument(skip_all)]
    pub fn new(capacity: usize) -> Self {
        debug!("Creating EventBus with capacity: {}", capacity);
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    #[instrument(skip_all)]
    pub fn publish(&self, event: Event) {
        debug!("Publishing event: {:?}", event);
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> impl Stream<Item = Event> {
        let rx = self.tx.subscribe();
        BroadcastStream::new(rx).filter_map(|result| result.ok())
    }
}
