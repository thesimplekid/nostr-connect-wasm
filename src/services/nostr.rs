use std::sync::Arc;
use tokio::sync::Mutex;

use nostr_sdk::{
    prelude::*,
    prelude::{Conditions, RemoteSigner},
    secp256k1::XOnlyPublicKey,
    Client, Keys,
};
use nostr_sdk::{EventId, UnsignedEvent, Url};
use wasm_bindgen_futures::spawn_local;
use yew::{AttrValue, Callback};

use anyhow::Result;
use log::{debug, error, warn};

use dashmap::{DashMap, DashSet};

#[derive(Clone)]
pub struct NostrService {
    pub client: Arc<Mutex<Client>>,
    pub signer_pubkey: Arc<Mutex<Option<XOnlyPublicKey>>>,
    pub relays: DashSet<Url>,
    pub unsigned_event: DashMap<EventId, UnsignedEvent>,
    pub delegation_token: Arc<Mutex<Option<Tag>>>,
}

impl NostrService {
    pub fn new(keys: &Keys, relay: Url) -> Result<Self> {
        let relays = DashSet::new();
        relays.insert(relay.clone());

        let remote_signer = RemoteSigner::new(relay, None);
        let client = Client::with_remote_signer(keys, remote_signer);
        let client = Arc::new(Mutex::new(client));

        // Spawn an thread that just listens for event
        Ok(Self {
            client,
            signer_pubkey: Arc::new(Mutex::new(None)),
            unsigned_event: DashMap::new(),
            delegation_token: Arc::new(Mutex::new(None)),
            relays,
        })
    }

    pub fn add_relay(&self, relay: Url) -> Result<()> {
        let client = self.client.clone();
        spawn_local(async move {
            let client = client.lock().await;
            client.add_relay(relay).await.ok();
            client.connect().await;
        });
        Ok(())
    }

    /// Create a new nostr client without a remote signer
    pub fn create_client(&mut self, new_relays: DashSet<Url>) -> Result<()> {
        let client = self.client.clone();
        let mut relays = self.relays.clone();
        relays.extend(new_relays);
        spawn_local(async move {
            let mut client = client.lock().await;
            let keys = client.keys();
            let new_client = Client::new(&keys);
            if (new_client.add_relays(relays.into_iter().collect()).await).is_ok() {
                new_client.connect().await;
                *client = new_client;
            } else {
                warn!("Could not create new client")
            }
        });

        Ok(())
    }

    /// Get delegation from remote signer
    pub fn get_delegate(&mut self, callback: Callback<AttrValue>) -> Result<()> {
        let client = self.client.clone();
        let delegation_token = self.delegation_token.clone();

        spawn_local(async move {
            let client = client.lock().await;
            let pubkey = client.keys().public_key();

            let mut conditions = Conditions::new();
            conditions.add(Condition::CreatedAfter(Timestamp::now().as_u64()));
            conditions.add(Condition::CreatedBefore(Timestamp::now().as_u64() + 7200));
            conditions.add(Condition::Kind(1));
            conditions.add(Condition::Kind(77));

            let req = Request::Delegate {
                public_key: pubkey,
                conditions,
            };
            match client.send_req_to_signer(req, None).await {
                Ok(res) => {
                    if let Response::Delegate(delegation_result) = res {
                        let delegation_tag = Tag::Delegation {
                            delegator_pk: delegation_result.from,
                            conditions: delegation_result.cond,
                            sig: delegation_result.sig,
                        };

                        debug!("{:?}", delegation_tag);
                        let mut delegation = delegation_token.lock().await;
                        *delegation = Some(delegation_tag);

                        // TODO: Since there is now a delegation there is no need for remote signer
                        callback.emit("".into());
                    }
                }
                Err(err) => error!("Get delegation error: {}", err),
            }
        });

        Ok(())
    }

    pub fn get_signer_pub_key(&self, callback: Callback<AttrValue>) -> Result<()> {
        let client = self.client.clone();
        spawn_local(async move {
            debug!("Waiting for pubkey");
            let client = client.lock().await;

            client.connect().await;

            match client.req_signer_public_key(None).await {
                Ok(_) => {
                    callback.emit("".into());
                }
                Err(err) => {
                    warn!("Could not set signer key {}", err);
                }
            }
            debug!("Set signer key");
        });

        Ok(())
    }

    pub fn publish_text_note(&self, content: &str, callback: Callback<AttrValue>) -> Result<()> {
        let client = self.client.clone();
        let content = content.to_owned();
        let delegation_tag = self.delegation_token.clone();
        spawn_local(async move {
            let delegation_tag = delegation_tag.lock().await;
            let tag = match &*delegation_tag {
                Some(tag) => vec![tag.to_owned()],
                None => vec![],
            };

            let event_id = client
                .lock()
                .await
                .publish_text_note(content, &tag)
                .await
                .unwrap();
            callback.emit(event_id.to_hex().into());
        });

        Ok(())
    }

    /*
    /// Wait for event
    pub fn wait_for_event(&self, callback: Callback<Message>) {
        let client = self.client.clone();

        spawn_local(async move {
            debug!("Wait for event");
            let mut notifications = client.lock().await.notifications();
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event(_url, event) = notification {
                    console::log_1(&format!("Got event: {:?}", event).into());
                    match event.kind {
                        Kind::NostrConnect => {
                            // Decrypt  nostr connect message
                            let sk = &client.lock().await.keys().secret_key().unwrap();
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
    */
}
