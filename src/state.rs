use std::sync::Arc;

use crate::models::Persons;
use crate::sites::faceit::Client;

#[derive(Clone)]
pub struct State {
    pub(crate) client: Client,
    pub(crate) persons: Arc<Persons>,
}

impl State {
    pub fn new(persons: Persons) -> anyhow::Result<Self> {
        Ok(Self::with_client(Client::new()?, persons))
    }

    pub fn with_client(client: Client, persons: Persons) -> Self {
        Self {
            client,
            persons: Arc::new(persons),
        }
    }
}
