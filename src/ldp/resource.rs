use hyper::{Body, Request};
use log::debug;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use tokio::fs::{metadata, File};
use tokio::prelude::*;

#[derive(Debug)]
pub enum Resource {
    RDFSource(PathBuf),
    NonRDF(PathBuf),
    NotFound,
}

impl Resource {
    pub fn from(request: &Request<Body>) -> Self {
        let file_path = PathBuf::from(request.uri().path().trim_start_matches('/'));
        // Try to open the file.
        if file_path.is_file() {
            debug!("found a file {:?}", file_path);
            if let Some(extension) = file_path.extension() {
                match extension.to_str() {
                    Some("ttl") => return Self::RDFSource(file_path),
                    Some("jsonld") => return Self::RDFSource(file_path),
                    _ => return Self::NonRDF(file_path),
                }
            }
            return Self::NonRDF(file_path);
        } else {
            debug!("not a file {:?}", file_path);
            Self::NotFound
        }
    }

    pub async fn to_body(&mut self) -> Result<Body, std::io::Error> {
        match self {
            Self::RDFSource(path) => {
                let mut file = File::open(path).await?;
                let mut contents = vec![];
                file.read_to_end(&mut contents).await?;
                Ok(Body::from(contents))
            }

            Self::NonRDF(path) => {
                let mut file = File::open(path).await?;
                let mut contents = vec![];
                file.read_to_end(&mut contents).await?;
                Ok(Body::from(contents))
            }

            Self::NotFound => {
                let not_found: &[u8] = b"NOT FOUND";
                Ok(not_found.into())
            }
        }
    }

    pub fn content_type(&self) -> Option<&str> {
        match self {
            Self::NotFound => None,

            Self::RDFSource(file_path) => {
                if let Some(extension) = file_path.extension() {
                    match extension.to_str() {
                        Some("ttl") => return Some("text/turtle"),
                        Some("jsonld") => return Some("application/ld+json"),
                        Some(_) => return Some("binary"), //octet stream?
                        None => return None,              //octet stream?
                    }
                }

                None //octet stream?
            }

            Self::NonRDF(_) => None, //octet stream?
        }
    }

    pub async fn etag(&self) -> String {
        match self {
            Self::NotFound => "".to_owned(),

            Self::NonRDF(file_path) | Self::RDFSource(file_path) => {
                if let Ok(metadata) = metadata(file_path).await {
                    if let Ok(modified) = metadata.modified() {
                        let mut h = DefaultHasher::new();
                        modified.hash(&mut h);
                        return h.finish().to_string();
                    }
                }
                "".to_owned()
            }
        }
    }
}
