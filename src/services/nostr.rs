use std::sync::{Arc, Mutex};

use nostr_sdk::{
    nips::nip46::Message,
    prelude::{decrypt, RemoteSigner},
    secp256k1::XOnlyPublicKey,
    Client, Keys, Kind, RelayPoolNotification,
};
use nostr_sdk::{EventId, UnsignedEvent, Url};
use wasm_bindgen_futures::spawn_local;
use web_sys::console::{self};
use yew::{AttrValue, Callback};

use anyhow::Result;
use log::{debug, warn};

use dashmap::DashMap;

#[derive(Clone)]
pub struct NostrService {
    pub client: Arc<Mutex<Client>>,
    pub signer_pubkey: Arc<Mutex<Option<XOnlyPublicKey>>>,
    pub unsigned_event: DashMap<EventId, UnsignedEvent>,
}

impl NostrService {
    pub fn new(keys: &Keys, relays: Url) -> Result<Self> {
        let remote_signer = RemoteSigner::new(relays.clone(), None);
        let client = Client::with_remote_signer(&keys, remote_signer);
        let client = Arc::new(Mutex::new(client));

        // Spawn an thread that just listens for event
        Ok(Self {
            client,
            signer_pubkey: Arc::new(Mutex::new(None)),
            unsigned_event: DashMap::new(),
        })
    }

    pub fn add_relay(&self, relay: Url) -> Result<()> {
        let client = self.client.clone();
        spawn_local(async move {
            let client = client.lock().unwrap();
            client.add_relay(relay).await.ok();
            client.connect().await;
        });
        Ok(())
    }

    pub fn get_signer_pub_key(&self) -> Result<()> {
        let client = self.client.clone();
        spawn_local(async move {
            debug!("Waiting for pubkey");
            let client = client.lock().unwrap();

            client.connect().await;

            if let Err(err) = client.req_signer_public_key(None).await {
                warn!("Could not set signer key {}", err);
            }
            debug!("Set signer key");
        });

        Ok(())
    }

    pub fn publish_text_note(&self, content: &str, callback: Callback<AttrValue>) -> Result<()> {
        let client = self.client.clone();
        let content = content.to_owned();

        spawn_local(async move {
            let event_id = client
                .lock()
                .unwrap()
                .publish_text_note(content, &[])
                .await
                .unwrap();
            callback.emit(event_id.to_hex().into());
        });

        Ok(())
    }

    /// Wait for event
    pub fn wait_for_event(&self, callback: Callback<Message>) {
        let client = self.client.clone();

        spawn_local(async move {
            debug!("Wait for event");
            let mut notifications = client.lock().unwrap().notifications();
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event(_url, event) = notification {
                    console::log_1(&format!("Got event: {:?}", event).into());
                    match event.kind {
                        Kind::NostrConnect => {
                            // Decrypt  nostr connect message
                            let sk = &client.lock().unwrap().keys().secret_key().unwrap();
                            match decrypt(sk, &event.pubkey, &event.content) {
                                Ok(msg) => {
                                    // NIP46 message from json
                                    let msg = Message::from_json(msg).unwrap();
                                    // Emit message
                                    callback.emit(msg);
                                }
                                Err(e) => eprintln!("Impossible to decrypt NIP46 message: {e}"),
                            }
                        }
                        _ => (),
                    }
                }
            }
        });
    }
}
