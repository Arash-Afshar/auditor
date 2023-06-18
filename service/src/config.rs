use crate::errors::AuditorError;
use serde::Deserialize;
use std::{env, fs::read_to_string};

macro_rules! update_from_toml {
    ($self:ident, $c:ident, $( $e:ident ).+) => {
        if let Some(value) = $c.$($e).+ {
            $self.$($e).+ = Some(value);
        }
    };
}

macro_rules! update_from_env {
    ($self:ident, $c:literal, $( $e:ident ).+) => {
        let value = env::var($c);
        if let Ok(value) = value {
            $self.$($e).+ = Some(value)
        }
    };
}

#[derive(Deserialize, Default)]
pub struct ConfigBuilder {
    repository_path: Option<String>,
    db_path: Option<String>,
    port: Option<String>,
    allowed_file_extensions: Option<String>,
    excluded_prefixes: Option<String>,
}

#[derive(Deserialize, Default, Clone)]
pub struct Config {
    pub repository_path: String,
    pub db_path: String,
    pub port: String,
    pub allowed_file_extensions: Vec<String>,
    pub excluded_prefixes: Vec<String>,
}

impl ConfigBuilder {
    fn try_from_string(content: String) -> Result<Self, AuditorError> {
        let c: ConfigBuilder = toml::from_str(&content)?;
        Ok(c)
    }

    pub fn toml(&mut self, path: &str) -> Result<&mut Self, AuditorError> {
        if let Ok(content) = read_to_string(path) {
            let c = Self::try_from_string(content)?;

            update_from_toml!(self, c, repository_path);
            update_from_toml!(self, c, db_path);
            update_from_toml!(self, c, port);
            update_from_toml!(self, c, allowed_file_extensions);
            update_from_toml!(self, c, excluded_prefixes);
        }

        Ok(self)
    }

    pub fn env(&mut self) -> Result<&mut Self, AuditorError> {
        update_from_env!(self, "REPO_PATH", repository_path);
        update_from_env!(self, "DB_PATH", db_path);
        update_from_env!(self, "PORT", port);
        update_from_env!(self, "ALLOWED_EXTENSIONS", allowed_file_extensions);
        update_from_env!(self, "EXCLUDED_PREFIXES", excluded_prefixes);
        Ok(self)
    }

    pub fn build(&self) -> Result<Config, AuditorError> {
        let split = |comma_separated: Option<String>| {
            if let Some(comma_separated) = comma_separated {
                comma_separated.split(',').map(|s| s.into()).collect()
            } else {
                vec![]
            }
        };
        Ok(Config {
            repository_path: self
                .repository_path
                .clone()
                .ok_or(AuditorError::MissingConfig("repository path".to_string()))?,
            db_path: self
                .db_path
                .clone()
                .ok_or(AuditorError::MissingConfig("database path".to_string()))?,
            port: self
                .port
                .clone()
                .or(Some("3000".to_string()))
                .expect("Will never fail"),
            allowed_file_extensions: split(self.allowed_file_extensions.clone()),
            excluded_prefixes: split(self.excluded_prefixes.clone()),
        })
    }
}
