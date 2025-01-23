use anyhow::{anyhow, Result};
use log::debug;
#[cfg(feature = "cookies")]
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use std::{fs::File, io::BufReader, path::PathBuf, sync::Arc};

#[cfg(feature = "cookies")]
pub struct Store {
    path: PathBuf,
    data: Arc<CookieStoreMutex>,
}

#[cfg(not(feature = "cookies"))]
pub struct Store {
    data: String,
}

impl Store {
    #[cfg(feature = "cookies")]
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let path = path.into();
        let data = if path.exists() {
            debug!("Opening cookie store located in {}", path.display());
            let file = File::open(&path).map(BufReader::new)?;
            CookieStore::load_json(file)
                .map_err(|e| anyhow!("Failed to load cookie store: {}", e))?
        } else {
            File::create(&path)?;
            CookieStore::default()
        };
        let data = Arc::new(CookieStoreMutex::new(data));
        Ok(Store { path, data })
    }

    pub fn get(&self) -> Arc<CookieStoreMutex> {
        self.data.clone()
    }

    pub fn save(&self) -> Result<()> {
        let mut writer = File::create(&self.path)?;
        let store = self.data.lock().unwrap();
        store
            .save_json(&mut writer)
            .map_err(|e| anyhow!("Failed to save cookie store: {}", e))?;
        Ok(())
    }
}
