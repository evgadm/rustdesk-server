use once_cell::sync::Lazy;
use std::{
    collections::HashSet,
    fs,
    path::PathBuf,
    sync::RwLock,
    time::{Duration, Instant},
};

const DEFAULT_BLOCKED_IDS_FILE: &str = "/opt/rustdesk-admin/runtime/blocked_ids.txt";
const REFRESH_INTERVAL: Duration = Duration::from_millis(500);

struct BlockedIdsCache {
    path: PathBuf,
    ids: HashSet<String>,
    last_check: Instant,
}

impl BlockedIdsCache {
    fn new() -> Self {
        let path = std::env::var_os("RUSTDESK_BLOCKED_IDS_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BLOCKED_IDS_FILE));

        Self {
            path,
            ids: HashSet::new(),
            last_check: Instant::now() - Duration::from_secs(60),
        }
    }

    fn refresh_if_needed(&mut self) {
        if self.last_check.elapsed() < REFRESH_INTERVAL {
            return;
        }

        self.last_check = Instant::now();

        // Keep the last successfully loaded list if the file is temporarily unreadable.
        if let Ok(content) = fs::read_to_string(&self.path) {
            self.ids = content
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(ToOwned::to_owned)
                .collect();
        }
    }
}

static BLOCKED_IDS: Lazy<RwLock<BlockedIdsCache>> =
    Lazy::new(|| RwLock::new(BlockedIdsCache::new()));

pub fn is_blocked(id: &str) -> bool {
    let mut cache = match BLOCKED_IDS.write() {
        Ok(cache) => cache,
        Err(poisoned) => poisoned.into_inner(),
    };

    cache.refresh_if_needed();
    cache.ids.contains(id)
}