pub mod types;
pub mod scanner;
pub mod evm;
pub mod config;
pub mod discovery;
pub mod filter;
pub mod inspect;
pub mod errors;
pub mod alloy_evm;

// helper
fn hex_to_bytes4(s: &str) -> [u8; 4] {
    let bytes = hex::decode(s).expect("invalid hex selector");
    [bytes[0], bytes[1], bytes[2], bytes[3]]
}