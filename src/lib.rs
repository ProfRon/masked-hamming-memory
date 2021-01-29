//! A crate to count ones and xor bytes, fast (aka popcount, hamming
//! weight and hamming distance).
//!
//! # Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! mhd_mem = "0.1"
//! ```
//!
//! # Examples
//!
//! ```rust
//! assert_eq!(mhd_mem::weight(&[1, 0xFF, 1, 0xFF]), 1 + 8 + 1 + 8);
//! assert_eq!(mhd_mem::distance(&[0xFF, 0xFF], &[1, 0xFF], &[0xFF, 1]), 7 + 7);
//! assert_eq!(mhd_mem::Sample::default().score, 0.0 );
//! assert_eq!(mhd_mem::Sample::new( 42.0 ).get_bit( 7 ), false );
//! ```

//   #![deny(warnings)]
// #![cfg_attr(not(test), no_std)]

extern crate hamming;

extern crate core;
#[cfg(test)] extern crate quickcheck;
#[cfg(test)] extern crate rand;


mod weight_;
pub use weight_::weight;

mod distance_;
pub use distance_::{distance, distance_fast, truncated_distance };

mod sample;
pub use sample::Sample;

mod util;

