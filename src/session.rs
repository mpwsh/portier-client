use anyhow::{anyhow, Context, Result};
use reqwest::{
    header::{HeaderMap, ACCEPT},
    Client,
};
use reqwest_cookie_store::CookieStoreMutex;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufWriter, Write},
    sync::Arc,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub session: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct UserData {
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub id_token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    id: String,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            id: String::from("0"),
        }
    }
}
impl Session {
    pub fn new(id: String) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn load(
        cookie_store: Arc<CookieStoreMutex>,
        rpc_endpoint: &str,
        session_cookie_name: &str,
    ) -> Result<(bool, String)> {
        let session_id = {
            let store = cookie_store.lock().unwrap();
            store
                .get(rpc_endpoint, "/", session_cookie_name)
                .map(|cookie| cookie.value().to_string())
        };
        Ok(session_id.map_or((false, String::new()), |token| (true, token)))
    }

    pub async fn save(cookies: Arc<CookieStoreMutex>) -> Result<()> {
        let mut writer = File::create("cookies.json").map(BufWriter::new)?;
        let store = cookies.lock().unwrap();
        let _ = store
            .save_json(&mut writer)
            .map_err(|e| anyhow!("Unable to save cookie to json: {}", e));
        writer.flush()?;
        Ok(())
    }

    pub async fn claim(client: &Client, rpc_addr: &str, id_token: &str) -> Result<String> {
        let params = [("id_token", id_token)];
        let mut map = HeaderMap::new();
        map.insert(ACCEPT, "application/json".parse()?);
        let res = client
            .post(format!("{}/claim", rpc_addr))
            .form(&params)
            .headers(map)
            .send()
            .await
            .context("Failed to claim session")?;
        res.text()
            .await
            .context("Failed to read verification response")
    }

    pub async fn login(client: &Client, rpc_addr: &str, email: &str) -> Result<AuthResponse> {
        let params = [("email", email)];
        let mut map = HeaderMap::new();
        map.insert(ACCEPT, "application/json".parse()?);

        let res = client
            .post(format!("{rpc_addr}/login"))
            .form(&params)
            .headers(map)
            .send()
            .await
            .context("Failed to send request")?;
        res.json::<AuthResponse>()
            .await
            .context("Failed to parse response as JSON")
    }
    pub async fn logout(client: &Client, rpc_addr: &str) -> Result<()> {
        let mut map = HeaderMap::new();
        map.insert(ACCEPT, "application/json".parse()?);

        let _ = client
            .post(format!("{rpc_addr}/logout"))
            .headers(map)
            .send()
            .await
            .context("Failed to send request")?;
        Ok(())
    }

    pub async fn confirm(
        client: &Client,
        broker_addr: &str,
        session: &str,
        code: &str,
    ) -> Result<VerifyResponse> {
        let params = [("session", session), ("code", code)];
        let mut map = HeaderMap::new();
        map.insert(ACCEPT, "application/json".parse()?);
        let res = client
            .post(format!("{}/confirm", broker_addr))
            .form(&params)
            .headers(map)
            .send()
            .await
            .context("Failed to confirm session")?;
        res.json::<VerifyResponse>()
            .await
            .context("Failed to parse confirmation response as JSON")
    }

    pub async fn whoami(client: &Client, rpc_addr: &str) -> Result<UserData> {
        let mut map = HeaderMap::new();
        map.insert(ACCEPT, "application/json".parse()?);
        let res = client
            .get(format!("{}/whoami", rpc_addr))
            .headers(map)
            .send()
            .await
            .context("Failed to send request")?;
        res.json::<UserData>()
            .await
            .context("Failed to parse user data as JSON")
    }
}
