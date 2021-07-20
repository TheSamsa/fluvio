//!
//! # Delete object
//!
//!

use std::io::Error;
use std::io::ErrorKind;

use tracing::trace;

use dataplane::core::{Encoder, Decoder, Version};
use dataplane::bytes::{Buf, BufMut};
use dataplane::api::Request;
use fluvio_controlplane_metadata::topic::TopicSpec;
use fluvio_controlplane_metadata::spu::CustomSpuSpec;
use fluvio_controlplane_metadata::spu::CustomSpuKey;
use fluvio_controlplane_metadata::spg::SpuGroupSpec;
use fluvio_controlplane_metadata::core::Spec;
use fluvio_controlplane_metadata::core::Removable;

use crate::Status;
use crate::AdminPublicApiKey;
use crate::AdminRequest;

pub trait DeleteSpec: Removable {
    /// convert delete key into request
    #[allow(clippy::wrong_self_convention)]
    fn into_request<K>(key: K) -> DeleteRequest
    where
        K: Into<Self::DeleteKey>;
}

// This can be auto generated by enum derive later
#[derive(Debug)]
pub enum DeleteRequest {
    Topic(String),
    CustomSpu(CustomSpuKey),
    SpuGroup(String),
}

impl Default for DeleteRequest {
    fn default() -> Self {
        DeleteRequest::CustomSpu(CustomSpuKey::Id(0))
    }
}

impl DeleteRequest {
    /// type represent as string
    fn type_string(&self) -> &'static str {
        match self {
            Self::Topic(_) => TopicSpec::LABEL,
            Self::CustomSpu(_) => CustomSpuSpec::LABEL,
            Self::SpuGroup(_) => SpuGroupSpec::LABEL,
        }
    }
}

impl AdminRequest for DeleteRequest {}

impl Request for DeleteRequest {
    const API_KEY: u16 = AdminPublicApiKey::Delete as u16;
    const DEFAULT_API_VERSION: i16 = 1;
    type Response = Status;
}

impl Encoder for DeleteRequest {
    fn write_size(&self, version: Version) -> usize {
        let type_size = self.type_string().to_owned().write_size(version);

        type_size
            + match self {
                Self::Topic(s) => s.write_size(version),
                Self::CustomSpu(s) => s.write_size(version),
                Self::SpuGroup(s) => s.write_size(version),
            }
    }

    // encode match
    fn encode<T>(&self, dest: &mut T, version: Version) -> Result<(), Error>
    where
        T: BufMut,
    {
        self.type_string().to_owned().encode(dest, version)?;

        match self {
            Self::Topic(s) => s.encode(dest, version)?,
            Self::CustomSpu(s) => s.encode(dest, version)?,
            Self::SpuGroup(s) => s.encode(dest, version)?,
        }

        Ok(())
    }
}

impl Decoder for DeleteRequest {
    fn decode<T>(&mut self, src: &mut T, version: Version) -> Result<(), Error>
    where
        T: Buf,
    {
        let mut typ = "".to_owned();
        typ.decode(src, version)?;
        trace!("decoded type: {}", typ);

        match typ.as_ref() {
            TopicSpec::LABEL => {
                let mut response = String::default();
                response.decode(src, version)?;
                *self = Self::Topic(response);
                Ok(())
            }

            CustomSpuSpec::LABEL => {
                let mut response = CustomSpuKey::default();
                response.decode(src, version)?;
                *self = Self::CustomSpu(response);
                Ok(())
            }

            SpuGroupSpec::LABEL => {
                let mut response = String::default();
                response.decode(src, version)?;
                *self = Self::SpuGroup(response);
                Ok(())
            }

            // Unexpected type
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!("invalid spec type {}", typ),
            )),
        }
    }
}
