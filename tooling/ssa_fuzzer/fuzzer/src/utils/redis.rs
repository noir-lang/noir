pub(crate) use redis::RedisResult;
use redis::{Client, Commands, Connection};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub(crate) struct RedisManager {
    client: Client,
    connection: Option<Connection>,
    reconnect_attempts: u32,
    max_reconnect_attempts: u32,
}

impl RedisManager {
    fn new(redis_url: &str) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        Ok(RedisManager {
            client,
            connection: None,
            reconnect_attempts: 0,
            max_reconnect_attempts: 5,
        })
    }

    fn get_connection(&mut self) -> RedisResult<&mut Connection> {
        // If we don't have a connection or need to reconnect, try to establish one
        if self.connection.is_none() || self.reconnect_attempts > 0 {
            match self.client.get_connection() {
                Ok(conn) => {
                    self.connection = Some(conn);
                    self.reconnect_attempts = 0;
                    log::info!("Redis connection established successfully");
                }
                Err(e) => {
                    self.reconnect_attempts += 1;
                    log::warn!(
                        "Failed to connect to Redis (attempt {}): {}",
                        self.reconnect_attempts,
                        e
                    );

                    if self.reconnect_attempts >= self.max_reconnect_attempts {
                        log::error!("Max reconnection attempts reached, giving up");
                        return Err(e);
                    }

                    std::thread::sleep(Duration::from_millis(
                        100 * u64::from(self.reconnect_attempts),
                    ));
                    return self.get_connection();
                }
            }
        }

        self.connection.as_mut().ok_or_else(|| {
            redis::RedisError::from((redis::ErrorKind::IoError, "No connection available"))
        })
    }

    pub(crate) fn execute_command<T, F>(&mut self, mut operation: F) -> RedisResult<T>
    where
        F: FnMut(&mut Connection) -> RedisResult<T>,
    {
        match self.get_connection() {
            Ok(conn) => match operation(conn) {
                Ok(result) => Ok(result),
                Err(e) => {
                    log::warn!("Redis operation failed: {e}");
                    self.connection = None;
                    self.reconnect_attempts = 1;

                    match self.get_connection() {
                        Ok(conn) => operation(conn),
                        Err(reconnect_err) => {
                            log::error!(
                                "Failed to reconnect after operation failure: {reconnect_err}",
                            );
                            Err(e)
                        }
                    }
                }
            },
            Err(e) => Err(e),
        }
    }

    pub(crate) fn push_to_queue(&mut self, queue_name: &str, value: &str) -> RedisResult<()> {
        self.execute_command(|conn| conn.rpush(queue_name, value)) //cSpell:disable-line
    }
}

lazy_static::lazy_static! {
    static ref REDIS_MANAGER: Arc<Mutex<Option<RedisManager>>> = Arc::new(Mutex::new(None));
}

pub(crate) fn get_redis_manager() -> Arc<Mutex<Option<RedisManager>>> {
    REDIS_MANAGER.clone()
}

pub(crate) fn ensure_redis_connection() -> bool {
    let redis_url = match std::env::var("REDIS_URL") {
        Ok(url) => url,
        Err(_) => {
            log::debug!("REDIS_URL environment variable not set");
            return false;
        }
    };

    let mut manager_guard = REDIS_MANAGER.lock().unwrap();

    if manager_guard.is_none() {
        match RedisManager::new(&redis_url) {
            Ok(manager) => {
                *manager_guard = Some(manager);
                log::info!("Redis manager initialized successfully");
                true
            }
            Err(e) => {
                log::error!("Failed to create Redis manager: {e}");
                false
            }
        }
    } else {
        true
    }
}

pub(crate) fn push_to_redis_queue(queue_name: &str, value: &str) -> RedisResult<()> {
    let manager = get_redis_manager();
    let mut manager_guard = manager.lock().unwrap();

    if let Some(ref mut redis_manager) = manager_guard.as_mut() {
        redis_manager.push_to_queue(queue_name, value)
    } else {
        Err(redis::RedisError::from((redis::ErrorKind::IoError, "Redis manager not initialized")))
    }
}
