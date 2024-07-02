use anyhow::{anyhow, Result};
use log::debug;
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use std::{fs::File, io::BufReader, path::PathBuf, sync::Arc};

pub struct Store {
    path: PathBuf,
    cookie_store: Arc<CookieStoreMutex>,
}

impl Store {
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let path = path.into();
        let cookie_store = if path.exists() {
            debug!("Opening cookie store located in {}", path.display());
            let file = File::open(&path).map(BufReader::new)?;
            CookieStore::load_json(file)
                .map_err(|e| anyhow!("Failed to load cookie store: {}", e))?
        } else {
            File::create(&path)?;
            CookieStore::default()
        };
        let cookie_store = Arc::new(CookieStoreMutex::new(cookie_store));
        Ok(Store { path, cookie_store })
    }

    pub fn cookie_store(&self) -> Arc<CookieStoreMutex> {
        self.cookie_store.clone()
    }

    pub fn save(&self) -> Result<()> {
        let mut writer = File::create(&self.path)?;
        let store = self.cookie_store.lock().unwrap();
        store
            .save_json(&mut writer)
            .map_err(|e| anyhow!("Failed to save cookie store: {}", e))?;
        Ok(())
    }
}
