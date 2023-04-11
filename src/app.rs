use std::str::FromStr;

use dashmap::DashSet;
use gloo::storage::SessionStorage;
use gloo::storage::Storage;
use log::{debug, warn};
use nostr_sdk::prelude::ToBech32;
use nostr_sdk::secp256k1::XOnlyPublicKey;
use nostr_sdk::Url;
use yew::prelude::*;
use yew::props;

use crate::components::navbar::{Navbar, Props as NavbarProps};
use crate::services::nostr::{DelegationInfo, NostrService};
use crate::utils::handle_keys;
use crate::views::{
    connect::{Connect, Props as ConnectProps},
    home::Home,
    settings::{DelegationInfoProp, Props as SettingsProps, Settings},
};

pub enum View {
    Home,
    Connect,
    Settings,
}

pub enum Msg {
    /// Publish a nostr note
    SubmitNote(AttrValue),
    /// Completed note broadcast
    BroadcastedEvent(AttrValue),
    /// Update Connect relay
    UpdateConnectRelay(AttrValue),
    /// Add relay to client
    AddRelay(AttrValue),
    /// Remove Relay
    RemoveRelay(Url),
    /// Set remote pubkey
    SetRemotePubkey(Option<XOnlyPublicKey>),
    /// Settings view
    Settings,
    // Home view
    Home,
    /// Send delegation request
    Delegate((u64, Vec<u64>)),
    /// Delegation token received
    DelegationSet,
    /// Got delgation info
    DelegationInfo(DelegationInfo),
    /// Log Out
    LogOut,
}

pub struct App {
    view: View,
    //navbar_active: bool,
    client: NostrService,
    broadcasted_event: Option<AttrValue>,
    name: AttrValue,
}
impl Component for App {
    type Message = Msg;
    type Properties = ConnectProps;

    fn create(ctx: &Context<Self>) -> Self {
        let connect_relay = Url::from_str("ws://localhost:8081").unwrap();
        // let relays: HashSet = HashSet::from_iter(vec![connect_relay.clone()]);

        let key: Option<String> = SessionStorage::get("priv_key").ok();
        let keys = handle_keys(key, true).unwrap();
        let remote_pubkey: Option<String> = SessionStorage::get("remote_pub_key").ok();
        let remote_pubkey = match remote_pubkey {
            Some(key) => XOnlyPublicKey::from_str(&key).ok(),
            None => None,
        };

        let delegation_tag: Option<String> = SessionStorage::get("delegationInfo").ok();

        // TODO: Clean this up
        // If there is a remote pubkey saved to session sotrange then create client with that as remote pubkey
        // If there is a VALID delegation tag saved to storage create a client without a remote and use the tag
        let (client, view) = match (remote_pubkey, delegation_tag) {
            (Some(_remote_key), Some(_tag)) => {
                let relays = DashSet::new();
                // TODO: Dont hard code this
                relays.insert(Url::from_str("ws://localhost:8081").unwrap());

                let client = NostrService::new_without_remote(&keys, relays).unwrap();

                (client, View::Home)
            }
            (Some(_remote_pubkey), None) => {
                let client = NostrService::new(&keys, remote_pubkey, connect_relay).unwrap();
                (client, View::Home)
            }
            _ => {
                let client = NostrService::new(&keys, None, connect_relay).unwrap();
                let signer_pubkey_callback = ctx.link().callback(Msg::SetRemotePubkey);

                client.req_signer_pub_key(signer_pubkey_callback).unwrap();
                (client, View::Connect)
            }
        };

        Self {
            // navbar_active: false,
            client,
            view,
            broadcasted_event: None,
            name: "nostr connect".into(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            /*
            Msg::ToggleNavbar => {
                self.navbar_active = !self.navbar_active;
                true
            }
            */
            Msg::AddRelay(relay) => {
                if let Ok(relay) = Url::from_str(&relay) {
                    self.client.add_relay(relay).ok();
                }
                true
            }
            Msg::RemoveRelay(relay) => {
                self.client.remove_relay(relay);
                true
            }
            Msg::UpdateConnectRelay(relay) => {
                if let Ok(relay) = Url::from_str(&relay) {
                    self.client.set_connect_relay(relay);
                    let delegation_info = match self.client.get_delegation_info() {
                        Ok(Some(info)) => Some(info),
                        _ => None,
                    };
                    if delegation_info.is_none() {
                        self.client.new_client_with_remote_signer();
                        let signer_pubkey_callback = ctx.link().callback(Msg::SetRemotePubkey);
                        self.client.req_signer_pub_key(signer_pubkey_callback).ok();
                    }
                }
                true
            }
            Msg::SubmitNote(note) => {
                debug!("Got note: {note}");
                let event_callback = ctx.link().callback(Msg::BroadcastedEvent);
                self.client.publish_text_note(&note, event_callback).ok();
                true
            }
            Msg::BroadcastedEvent(event_id) => {
                debug!("Broadcasted: {}", event_id);
                self.broadcasted_event = Some(event_id);
                true
            }
            Msg::Delegate((expiration, kinds)) => {
                let delegate_callback = ctx.link().callback(|_| Msg::DelegationSet);
                let delegation_info_callback = ctx.link().callback(Msg::DelegationInfo);

                self.client
                    .get_delegate(
                        expiration,
                        kinds,
                        delegate_callback,
                        delegation_info_callback,
                    )
                    .ok();
                true
            }
            Msg::DelegationSet => {
                debug!("Delegation set");
                // Since there is now a delegation there is no need for remote signer
                self.client.create_client(DashSet::new()).ok();
                false
            }
            Msg::DelegationInfo(delegation_info) => {
                if self.client.set_delegation_info(delegation_info).is_err() {
                    warn!("Could not set delegation info")
                }
                true
            }
            Msg::SetRemotePubkey(pubkey) => {
                self.client.set_remote_pubkey(pubkey);
                self.view = View::Home;
                true
            }
            Msg::Settings => {
                self.view = View::Settings;
                true
            }
            Msg::Home => {
                // If the app is not connected to a remote signer
                // AND does not have a delegation tag
                // Redirect to connect page

                let delegation_info = match self.client.get_delegation_info() {
                    Ok(Some(info)) => Some(info),
                    _ => None,
                };

                let view = if self.client.get_remote_signer().is_none() && delegation_info.is_none()
                {
                    View::Connect
                } else {
                    View::Home
                };
                self.view = view;
                true
            }
            Msg::LogOut => {
                let keys = handle_keys(None, true).unwrap();
                // Clear session
                SessionStorage::clear();
                self.client =
                    NostrService::new(&keys, None, self.client.get_connect_relay()).unwrap();

                self.view = View::Connect;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let settings_cb = ctx.link().callback(|_| Msg::Settings);
        let home_cb = ctx.link().callback(|_| Msg::Home);

        let props = props! {
            NavbarProps {
                settings_cb,
                home_cb
            }
        };

        html! {
        <>
            {

            html! { <Navbar .. props />}
            }

            {

            match self.view {

                View::Home => {
                    let note_cb = ctx.link().callback(Msg::SubmitNote);
                    let delegator = match self.client.get_delegation_info() {
                        Ok(Some(info)) => Some(DelegationInfoProp::new(info.delegator_pubkey, info.created_after(), info.created_before(), info.kinds())),
                        _ => None
                    };

                    debug!("Delegator info: {:?}", delegator);

                    let remote_signer = self.client.get_remote_signer().map(|p| AttrValue::from(p.to_string()));

                    html!{
                    <>
                    if let Some(event_id) = &self.broadcasted_event {
                        <p>{ format!("Broadcasted event: {}", event_id)}</p>
                    }
                    <Home {note_cb} {delegator} {remote_signer}/>
                    </>
                }
            },
                View::Connect => {
                    let connected_cb = ctx.link().callback(|_| Msg::Home);
                    let set_relay_cb = ctx.link().callback(Msg::AddRelay);
                    let props = props! {
                        ConnectProps {
                            pubkey: self.client.get_app_pubkey().to_string(),
                            connect_relay: self.client.get_connect_relay().to_string(),
                            name: self.name.clone(),
                            connected_cb,
                            set_relay_cb
                        }
                    };

                    html! { <Connect .. props /> }
                }
                View::Settings => {

                    let delegation_info = match self.client.get_delegation_info() {
                        Ok(Some(info)) => Some(DelegationInfoProp::new(info.delegator_pubkey, info.created_after(), info.created_before(), info.kinds())),
                        _ => None
                    };
                    let delegation_cb = ctx.link().callback(Msg::Delegate);

                    let update_connect_relay_cb = ctx.link().callback(Msg::UpdateConnectRelay);
                    let add_relay_cb = ctx.link().callback(Msg::AddRelay);
                    let logout_cb = ctx.link().callback(|_| Msg::LogOut);
                    let remove_relay_cb = ctx.link().callback(Msg::RemoveRelay);
                    let props = props! {
                        SettingsProps {
                            app_pubkey: self.client.get_app_pubkey().to_bech32().unwrap(),
                            delegation_info: delegation_info,
                            connect_relay: self.client.get_connect_relay().to_string(),
                            relays: self.client.get_relays(),
                            update_connect_relay_cb,
                            add_relay_cb,
                            logout_cb,
                            remove_relay_cb,
                            delegation_cb
                        }

                    };
                    html! { <Settings .. props />}
                }
            }
        }
        <footer class="footer">
        </footer>
        </>
        }
    }
}
