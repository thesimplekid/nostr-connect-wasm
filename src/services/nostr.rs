use std::sync::Arc;

use anyhow::Result;
use dashmap::DashSet;
use log::{debug, error, warn};
use nostr_sdk::{
    prelude::*,
    secp256k1::{schnorr::Signature, XOnlyPublicKey},
    Client, Keys, Url,
};
use tokio::sync::Mutex;
use wasm_bindgen_futures::spawn_local;
use yew::{AttrValue, Callback};

#[derive(Clone)]
pub struct DelegationInfo {
    pub delegator_pubkey: XOnlyPublicKey,
    pub conditions: Conditions,
    pub signature: Signature,
}

impl DelegationInfo {
    /// Unix time the delegation expires
    pub fn created_before(&self) -> Option<u64> {
        self.conditions
            .inner()
            .iter()
            .find_map(|condition| match condition {
                Condition::CreatedBefore(time) => Some(*time),
                _ => None,
            })
    }

    /// Unix time the delegation is valid from
    pub fn created_after(&self) -> Option<u64> {
        self.conditions
            .inner()
            .iter()
            .find_map(|condition| match condition {
                Condition::CreatedAfter(time) => Some(*time),
                _ => None,
            })
    }

    /// Event kinds delegation is valid for
    pub fn _kinds(&self) -> Vec<u64> {
        self.conditions
            .inner()
            .iter()
            .filter_map(|condition| match condition {
                Condition::Kind(kind) => Some(*kind),
                _ => None,
            })
            .collect()
    }
}

/// Nostr service
#[derive(Clone)]
pub struct NostrService {
    keys: Keys,
    client: Arc<Mutex<Client>>,
    relays: DashSet<Url>,
    remote_signer: Option<XOnlyPublicKey>,
    delegation_info: Option<DelegationInfo>,
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
            relays,
            keys: keys.clone(),
            delegation_info: None,
            remote_signer: None,
        })
    }

    /// Add new relay to client
    pub fn add_relay(&self, relay: Url) -> Result<()> {
        let client = self.client.clone();
        spawn_local(async move {
            let client = client.lock().await;
            client.add_relay(relay).await.ok();
            client.connect().await;
        });
        Ok(())
    }

    /// Get pubkey of app
    pub fn get_app_pubkey(&self) -> XOnlyPublicKey {
        self.keys.public_key()
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
    pub fn get_delegate(
        &mut self,
        callback: Callback<AttrValue>,
        delegation_info_cb: Callback<DelegationInfo>,
    ) -> Result<()> {
        let client = self.client.clone();

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
                        let delegation_info = DelegationInfo {
                            delegator_pubkey: delegation_result.from,
                            conditions: delegation_result.cond,
                            signature: delegation_result.sig,
                        };

                        delegation_info_cb.emit(delegation_info);

                        callback.emit("".into());
                    }
                }
                Err(err) => error!("Get delegation error: {}", err),
            }
        });

        Ok(())
    }

    /// Set the app delegation info
    pub fn set_delegation_info(&mut self, delegation_info: DelegationInfo) {
        self.delegation_info = Some(delegation_info);
    }

    /// Get the app delegation info
    pub fn get_delegation_info(&self) -> Option<DelegationInfo> {
        self.delegation_info.to_owned()
    }

    /// Wait for pubkey of signer
    pub fn req_signer_pub_key(&self, callback: Callback<Option<XOnlyPublicKey>>) -> Result<()> {
        let client = self.client.clone();
        spawn_local(async move {
            debug!("Waiting for pubkey");
            let client = client.lock().await;

            client.connect().await;

            match client.req_signer_public_key(None).await {
                Ok(_) => {
                    let remote = client.remote_signer().unwrap().signer_public_key().await;
                    callback.emit(remote);
                }
                Err(err) => {
                    warn!("Could not set signer key {}", err);
                }
            }
            debug!("Set signer key");
        });

        Ok(())
    }

    /// Set remote signer pubkey
    pub fn set_remote_pubkey(&mut self, pubkey: Option<XOnlyPublicKey>) {
        self.remote_signer = pubkey;
    }

    /// Get remote signer pubkey
    pub fn get_remote_pubkey(&self) -> Option<XOnlyPublicKey> {
        self.remote_signer
    }

    /// Create delegation `Tag` from service delegation info
    fn delegation_tag(&self) -> Option<Tag> {
        if let Some(delegation) = &self.delegation_info {
            Some(Tag::Delegation {
                delegator_pk: delegation.delegator_pubkey,
                conditions: delegation.conditions.clone(),
                sig: delegation.signature,
            })
        } else {
            None
        }
    }

    /// Publish a text note
    pub fn publish_text_note(&self, content: &str, callback: Callback<AttrValue>) -> Result<()> {
        let client = self.client.clone();
        let content = content.to_owned();
        let delegation_tag = self.delegation_tag();
        spawn_local(async move {
            let tag = match delegation_tag {
                Some(tag) => vec![tag],
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
    // This shouldn't be needed as should be able to use nostr-sdk subscribe
    // Just gonna let it hang around as reference for now
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
