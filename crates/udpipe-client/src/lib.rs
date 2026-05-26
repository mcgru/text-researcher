//! UDPipe 2 client — safe Rust wrapper around the UDPipe 2 Python library.
//!
//! Communication via subprocess: spawns `python3 -m udpipe2 --model <path>`,
//! pipes text via stdin, parses CoNLL-U from stdout. No unsafe code.

pub mod conllu;
pub mod error;
pub mod pipeline;

pub use conllu::{Sentence, Token};
pub use error::UdpipelineError;
pub use pipeline::Udpipeline;
