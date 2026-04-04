use keyring::Entry;
use log::{info, error};
use thiserror::Error;

const SERVICE_NAME: &str = "promptvault";

#[derive(Error, Debug)]
pub enum KeychainError {
    #[error("Keychain entry error: {0}")]
    EntryError(#[from] keyring::Error),
    #[error("API key not found for provider: {0}")]
    NotFound(String),
}

pub struct Keychain;

impl Keychain {
    pub fn store_api_key(provider: &str, api_key: &str) -> Result<(), KeychainError> {
        let entry = Entry::new(SERVICE_NAME, provider)?;
        entry.set_password(api_key)?;
        info!("Stored API key for provider: {}", provider);
        Ok(())
    }

    pub fn get_api_key(provider: &str) -> Result<String, KeychainError> {
        let entry = Entry::new(SERVICE_NAME, provider)?;
        match entry.get_password() {
            Ok(key) => Ok(key),
            Err(keyring::Error::NoEntry) => Err(KeychainError::NotFound(provider.to_string())),
            Err(e) => Err(KeychainError::EntryError(e)),
        }
    }

    pub fn delete_api_key(provider: &str) -> Result<(), KeychainError> {
        let entry = Entry::new(SERVICE_NAME, provider)?;
        match entry.delete_credential() {
            Ok(_) => {
                info!("Deleted API key for provider: {}", provider);
                Ok(())
            }
            Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
            Err(e) => Err(KeychainError::EntryError(e)),
        }
    }

    pub fn has_api_key(provider: &str) -> bool {
        Self::get_api_key(provider).is_ok()
    }
}