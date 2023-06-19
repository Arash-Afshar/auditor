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

#[derive(Deserialize, Default, Clone, Debug)]
pub struct Config {
    pub repository_path: String,
    pub db_path: String,
    pub port: String,
    pub allowed_file_extensions: Vec<String>,
    pub excluded_prefixes: Vec<String>,
}

impl ConfigBuilder {
    pub fn try_from_string(content: String) -> Result<Self, AuditorError> {
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

#[cfg(test)]
mod tests {
    use super::ConfigBuilder;

    #[test]
    fn test_load_from_string() {
        let builder = ConfigBuilder::try_from_string(
            r#"
repository_path = "/path/to/repo"
db_path = "/path/to/db"
port = "3000"
allowed_file_extensions = ".rs,.go"
excluded_prefixes = "/path/1,/path/2"
        "#
            .to_string(),
        )
        .unwrap();
        assert_eq!(builder.repository_path, Some("/path/to/repo".to_string()));
        assert_eq!(builder.db_path, Some("/path/to/db".to_string()));
        assert_eq!(builder.port, Some("3000".to_string()));
        assert_eq!(builder.allowed_file_extensions, Some(".rs,.go".to_string()));
        assert_eq!(
            builder.excluded_prefixes,
            Some("/path/1,/path/2".to_string())
        );
        let c = builder.build().unwrap();
        assert_eq!(c.repository_path, "/path/to/repo".to_string());
        assert_eq!(c.db_path, "/path/to/db".to_string());
        assert_eq!(c.port, "3000".to_string());
        assert_eq!(c.allowed_file_extensions, vec![".rs", ".go"]);
        assert_eq!(c.excluded_prefixes, vec!["/path/1", "/path/2"]);
    }
}
