use crate::{Result, Session, Store, UserData};
use anyhow::Error;
use reqwest::Client as ReqwestClient;
use std::path::PathBuf;
use std::sync::Arc;

pub struct ClientBuilder {
    store_path: PathBuf,
    session_cookie_domain: String,
    session_cookie_name: String,
    rpc_addr: String,
    broker_addr: String,
}

pub struct Client {
    store: Arc<Store>,
    client: ReqwestClient,
    session: Option<Session>,
    rpc_addr: String,
    broker_addr: String,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            store_path: PathBuf::from("cookies.json"),
            session_cookie_domain: "127.0.0.1".to_string(),
            session_cookie_name: "id".to_string(),
            rpc_addr: "http://127.0.0.1:8000".to_string(),
            broker_addr: "http://127.0.0.1:3333".to_string(),
        }
    }
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_store<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.store_path = path.into();
        self
    }

    pub fn with_session_cookie_domain(mut self, endpoint: impl Into<String>) -> Self {
        self.session_cookie_domain = endpoint.into();
        self
    }

    pub fn with_session_cookie_name(mut self, name: impl Into<String>) -> Self {
        self.session_cookie_name = name.into();
        self
    }

    pub fn with_rpc_addr(mut self, addr: impl Into<String>) -> Self {
        self.rpc_addr = addr.into();
        self
    }

    pub fn with_broker_addr(mut self, addr: impl Into<String>) -> Self {
        self.broker_addr = addr.into();
        self
    }

    pub fn build(self) -> Result<Client> {
        let store = Store::new(self.store_path)?;
        let cookie_store = store.cookie_store();

        let client = ReqwestClient::builder()
            .cookie_provider(cookie_store.clone())
            .build()?;

        let (has_session, session_id) =
            Session::load(cookie_store, &self.session_cookie_domain, &self.session_cookie_name)?;
        Ok(Client {
            store: Arc::new(store),
            client,
            session: if has_session {
                Some(Session::new(session_id))
            } else {
                None
            },
            rpc_addr: self.rpc_addr,
            broker_addr: self.broker_addr,
        })
    }
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn store(&self) -> Arc<Store> {
        self.store.clone()
    }

    pub fn reqwest_client(&self) -> &ReqwestClient {
        &self.client
    }

    pub fn session(&self) -> Option<Session> {
        self.session.clone()
    }

    pub async fn login(&mut self, email: &str) -> Result<()> {
        let auth_response = Session::login(&self.client, &self.rpc_addr, email).await?;
        self.session = Some(Session::new(auth_response.session));
        Ok(())
    }
    pub async fn logout(&mut self) -> Result<()> {
        Session::logout(&self.client, &self.rpc_addr).await
    }

    pub async fn confirm(&mut self, code: &str) -> Result<()> {
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| Error::msg("No active session"))?;
        let verify_response =
            Session::confirm(&self.client, &self.broker_addr, session.id(), code).await?;
        let new_session =
            Session::claim(&self.client, &self.rpc_addr, &verify_response.id_token).await?;
        self.session = Some(Session::new(new_session));
        Ok(())
    }

    pub async fn whoami(&self) -> Result<UserData> {
        Session::whoami(&self.client, &self.rpc_addr).await
    }

    pub async fn save_session(&self) -> Result<()> {
        self.store.save()
    }
}
