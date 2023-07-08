//! Scanning processes the raw source files or input text and returns a
//! structured tree of nodes ready for parsing.
//!
//! This process includes the lexical analysis and tokenization, but includes
//! additional steps such as parsing brackets, lines, and indentation.
//!
//! In essence, the scanning process is responsible for breaking up input
//! sources into their broad structure.
//!
//! The scanner operates like a pipeline. Each step of the pipeline receives
//! a segment of data and generates a list of segments to the next step.
//!
//! The individual scanning steps can be customized, but they always start
//! with a single [`Span`] and end with a [`NodeList`] as a result.
//!

use super::*;

pub mod token;

pub use token::*;
