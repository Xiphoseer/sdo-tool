#![allow(non_snake_case)]

pub mod mappings {
    include!(concat!(env!("OUT_DIR"), "/mappings.rs"));
}
