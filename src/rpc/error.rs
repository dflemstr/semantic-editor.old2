//! Error type definitions for errors that can occur during RPC interactions.

use prost;

error_chain!{
    foreign_links {
        Decode(prost::DecodeError) #[doc="An error occurred during input decoding."];
        Encode(prost::EncodeError) #[doc="An error occurred during output encoding."];
    }
}
