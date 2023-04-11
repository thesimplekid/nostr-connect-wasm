use std::{collections::HashSet, str::FromStr, sync::Arc};

use anyhow::Result;
use dashmap::DashSet;
use gloo::storage::{SessionStorage, Storage};
use log::{debug, error, warn};
use nostr_sdk::{
    prelude::*,
    secp256k1::{schnorr::Signature, XOnlyPublicKey},
    Client, Keys, Url,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use wasm_bindgen_futures::spawn_local;
use yew::{AttrValue, Callback};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn kinds(&self) -> Vec<u64> {
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
    connect_relay: Url,
    relays: DashSet<Url>,
    remote_signer: Option<XOnlyPublicKey>,
}

impl NostrService {
    pub fn new(
        keys: &Keys,
        remote_signer_pubkey: Option<XOnlyPublicKey>,
        connect_relay: Url,
    ) -> Result<Self> {
        SessionStorage::set("priv_key", keys.secret_key().unwrap()).expect("failed to set");
        let relays = DashSet::new();
        relays.insert(connect_relay.clone());

        let remote_signer = RemoteSigner::new(connect_relay.clone(), remote_signer_pubkey);
        let client = Client::with_remote_signer(keys, remote_signer);
        let client = Arc::new(Mutex::new(client));

        let client_clone = client.clone();
        let connect_relay_clone = connect_relay.clone();
        spawn_local(async move {
            let client = client_clone.lock().await;
            client.add_relay(connect_relay_clone).await.unwrap();
            client.connect().await;
        });

        // Spawn an thread that just listens for event
        Ok(Self {
            client,
            connect_relay,
            relays,
            keys: keys.clone(),
            remote_signer: remote_signer_pubkey,
        })
    }

    pub fn new_without_remote(keys: &Keys, relays: DashSet<Url>) -> Result<Self> {
        SessionStorage::set("priv_key", keys.secret_key().unwrap()).expect("failed to set");
        // TODO: Save relays
        let client = Client::new(keys);

        let client = Arc::new(Mutex::new(client));

        let client_clone = client.clone();
        let relays_clone = relays.clone();
        spawn_local(async move {
            let client = client_clone.lock().await;

            if let Err(err) = client
                .add_relays(relays_clone.iter().map(|r| r.to_string()).collect())
                .await
            {
                warn!("Could not add relays {}", err);
            }

            client.connect().await;
        });

        Ok(Self {
            client,
            connect_relay: Url::from_str("ws://localhost:8081").unwrap(),
            keys: keys.clone(),
            remote_signer: None,
            relays,
        })
    }

    /// Add new relay to client
    pub fn add_relay(&self, relay: Url) -> Result<()> {
        let client = self.client.clone();
        self.relays.insert(relay.clone());
        spawn_local(async move {
            let client = client.lock().await;
            client.add_relay(relay).await.ok();
            client.connect().await;
        });
        Ok(())
    }

    /// Remove relay
    pub fn remove_relay(&mut self, relay: Url) {
        let client = self.client.clone();
        self.relays.remove(&relay);
        spawn_local(async move {
            let client = client.lock().await;
            client.remove_relay(relay).await.ok();
        });
    }

    /// Set connect relay
    pub fn set_connect_relay(&mut self, relay: Url) {
        debug!("Setting connect relay");
        self.connect_relay = relay;
    }

    /// Get connect relay
    pub fn get_connect_relay(&self) -> Url {
        self.connect_relay.to_owned()
    }

    /// Get relays
    pub fn get_relays(&self) -> HashSet<Url> {
        HashSet::from_iter(self.relays.iter().map(|r| r.to_owned()))
    }

    /// Get pubkey of app
    pub fn get_app_pubkey(&self) -> XOnlyPublicKey {
        self.keys.public_key()
    }

    /// Create a new nostr client with a remote signer
    pub fn new_client_with_remote_signer(&mut self) {
        let client = self.client.clone();
        let connect_relay = self.connect_relay.clone();

        spawn_local(async move {
            let mut client = client.lock().await;
            let keys = client.keys();
            let remote_signer = RemoteSigner::new(connect_relay, None);
            let new_client = Client::with_remote_signer(&keys, remote_signer);
            client.connect().await;
            *client = new_client;
        });
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
        expiration_unix_time: u64,
        kinds: Vec<u64>,
        callback: Callback<AttrValue>,
        delegation_info_cb: Callback<DelegationInfo>,
    ) -> Result<()> {
        let client = self.client.clone();

        spawn_local(async move {
            let client = client.lock().await;
            let pubkey = client.keys().public_key();

            let mut conditions = Conditions::new();
            // Set valid from time as current time
            conditions.add(Condition::CreatedAfter(Timestamp::now().as_u64()));
            conditions.add(Condition::CreatedBefore(expiration_unix_time));

            for kind in kinds {
                conditions.add(Condition::Kind(kind));
            }

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
    pub fn set_delegation_info(&mut self, delegation_info: DelegationInfo) -> Result<()> {
        SessionStorage::set("delegationInfo", serde_json::to_string(&delegation_info)?)?;
        Ok(())
    }

    /// Get the app delegation info
    pub fn get_delegation_info(&self) -> Result<Option<DelegationInfo>> {
        if let Ok(Some(info)) = SessionStorage::get::<Option<String>>("delegationInfo") {
            let delegation_info: DelegationInfo = serde_json::from_str(info.as_str())?;

            if verify_delegation_signature(
                delegation_info.delegator_pubkey,
                delegation_info.signature,
                self.keys.public_key(),
                delegation_info.conditions.clone(),
            )
            .is_ok()
            {
                return Ok(Some(delegation_info));
            }
        }
        Ok(None)
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

                    if let Some(remote) = remote {
                        if let Err(err) = SessionStorage::set("remote_pub_key", remote.to_string())
                        {
                            warn!("Could not set remote pubkey {}", err);
                        }
                    }
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
    pub fn get_remote_signer(&self) -> Option<XOnlyPublicKey> {
        self.remote_signer
    }

    /// Create delegation `Tag` from service delegation info
    fn delegation_tag(&self) -> Result<Option<Tag>> {
        let delegation_info = self.get_delegation_info()?;

        let tag = delegation_info.map(|delegation| Tag::Delegation {
            delegator_pk: delegation.delegator_pubkey,
            conditions: delegation.conditions.clone(),
            sig: delegation.signature,
        });

        Ok(tag)
    }

    /// Publish a text note
    pub fn publish_text_note(&self, content: &str, callback: Callback<AttrValue>) -> Result<()> {
        let client = self.client.clone();
        let content = content.to_owned();
        let delegation_tag = self.delegation_tag();
        debug!("Tet: {:?}", delegation_tag);
        spawn_local(async move {
            let tag = match delegation_tag {
                Ok(Some(tag)) => {
                    vec![tag]
                }
                Err(err) => {
                    warn!("Could not get delegation tag: {}", err);
                    vec![]
                }
                _ => vec![],
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
