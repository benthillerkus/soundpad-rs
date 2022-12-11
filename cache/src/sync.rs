use soundpad_remote_client::Client;
use tokio::time;

use crate::Pool;

pub(crate) async fn sync_task(pool: Pool, client: Client) {
    loop {
        time::sleep(time::Duration::from_secs(10)).await;
    }
}
