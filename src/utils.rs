use nostr_sdk::prelude::*;

use log::debug;

pub fn handle_keys(private_key: Option<String>, hex: bool) -> Result<Keys> {
    // Parse and validate private key
    let keys = match private_key {
        Some(pk) => {
            // create a new identity using the provided private key
            Keys::from_sk_str(pk.as_str())?
        }
        None => {
            // create a new identity with a new keypair
            println!("No private key provided, creating new identity");
            Keys::generate()
        }
    };

    if !hex {
        debug!("Public key: {}", keys.public_key().to_bech32()?);
    } else {
        debug!("Public key: {}", keys.public_key());
    }
    Ok(keys)
}
