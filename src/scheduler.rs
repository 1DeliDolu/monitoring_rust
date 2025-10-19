use tokio::time::sleep;
use tracing::{error, warn};

use crate::{
    collector::{self, SystemSnapshot},
    state::SharedState,
    storage,
};

pub fn spawn(state: SharedState) {
    let config = state.config().clone();

    tokio::spawn(async move {
        let mut previous: Option<SystemSnapshot> = None;
        let interval = config.collection_interval();

        loop {
            let prev_for_collect = previous.clone();
            let interval_copy = interval;

            let collection_result =
                tokio::task::spawn_blocking(move || collector::collect_snapshot(prev_for_collect, interval_copy))
                    .await;

            let snapshot = match collection_result {
                Ok(snapshot) => snapshot,
                Err(err) => {
                    error!("collector worker failed: {}", err);
                    sleep(interval).await;
                    continue;
                }
            };

            if let Err(err) = storage::save_snapshot(&snapshot, &config).await {
                warn!("could not persist snapshot: {}", err);
            }

            state.record_snapshot(snapshot.clone()).await;
            previous = Some(snapshot);

            sleep(interval).await;
        }
    });
}
