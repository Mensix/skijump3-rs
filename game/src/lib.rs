pub mod app;
pub mod competition;
pub mod components;
pub mod content;
pub mod controllers;
pub mod data;
pub(crate) mod error;
pub mod files;
pub mod gfx;
pub mod jump;
pub mod rng;
pub mod route;
pub mod save;
pub mod store;
pub mod text;
pub mod views;

pub use app::run;
