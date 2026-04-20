pub mod decoder;
pub mod encoder;
pub mod error;
pub mod item;
pub mod traits;

#[cfg(test)]
mod roundtrip;

pub use error::RlpError;
pub use traits::{RlpDecodable, RlpEncodable};
pub use item::RlpItem;
pub use encoder::encode;
pub use decoder::decode;