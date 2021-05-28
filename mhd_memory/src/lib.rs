//! A crate to implement Klaurer's Masked Hamming Distance (MHD) and
//! his MHD-Memory (an associative memory using the MHD function).
//!
//! # Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! mhd_memory = "0.0.1"
//! ```
//!
//! # Examples
//!
//! ```rust
//! use mhd_memory::*;
//! assert_eq!( weight(&[1, 0xFF, 1, 0xFF]), 1 + 8 + 1 + 8);
//! assert_eq!( distance(&[0xFF, 0xFF], &[1, 0xFF], &[0xFF, 1]), 7 + 7);
//! assert_eq!( Sample::default().score, ZERO_SCORE ); // Sample width is 200 bits
//! assert_eq!( Sample::new( 120, 42 as ScoreType ).get_bit( 7 ), false );
//! assert_eq!( 120, MhdMemory::new( 120 ).width );
//! assert_eq!(   0, MhdMemory::new( 120 ).num_samples() );
//!
//! ```

//   #![deny(warnings)]
// #![cfg_attr(not(test), no_std)]

extern crate core;
extern crate hamming;
extern crate log;
extern crate rand;
extern crate rand_distr;
extern crate rayon;

pub mod util;

pub mod weight_;
pub use self::weight_::weight;

pub mod distance_;
pub use self::distance_::{distance, distance_fast, truncated_distance};

pub mod sample;
pub use self::sample::{Sample, ScoreType, ZERO_SCORE};

pub mod mhdmemory;
pub use self::mhdmemory::MhdMemory;
