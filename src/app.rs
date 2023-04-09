use std::str::FromStr;

use dashmap::DashSet;
use log::debug;
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
    /// Set remote pubkey
    SetRemotePubkey(Option<XOnlyPublicKey>),
    /// Settings view
    Settings,
    // Home view
    Home,
    /// Send delegation request
    Delegate,
    /// Delegation token received
    DelegationSet,
    /// Got delgation info
    DelegationInfo(DelegationInfo),
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
        let keys = handle_keys(None, true).unwrap();

        let client = NostrService::new(&keys, connect_relay.clone()).unwrap();

        client.add_relay(connect_relay.clone()).ok();

        let signer_pubkey_callback = ctx.link().callback(Msg::SetRemotePubkey);

        client.req_signer_pub_key(signer_pubkey_callback).unwrap();

        Self {
            // navbar_active: false,
            client,
            view: View::Connect,
            broadcasted_event: None,
            // publish_relays: relays,
            name: "dartstr".into(),
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
                debug!("Setting relay: {relay}");
                if let Ok(relay) = Url::from_str(&relay) {
                    self.client.add_relay(relay).ok();
                }
                true
            }
            Msg::UpdateConnectRelay(relay) => {
                if let Ok(relay) = Url::from_str(&relay) {
                    self.client.set_connect_relay(relay);
                    if self.client.get_delegation_info().is_none() {
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
            Msg::Delegate => {
                debug!("Delegate");
                let delegate_callback = ctx.link().callback(|_| Msg::DelegationSet);
                let delegation_info_callback = ctx.link().callback(Msg::DelegationInfo);

                self.client
                    .get_delegate(delegate_callback, delegation_info_callback)
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
                self.client.set_delegation_info(delegation_info);
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
                let view;

                // If the app is not connected to a remote signer
                // AND does not have a delegation tag
                // Redirect to connect page
                if self.client.get_remote_signer().is_none()
                    && self.client.get_delegation_info().is_none()
                {
                    view = View::Connect;
                } else {
                    view = View::Home;
                }
                self.view = view;
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
                    let delegate_cb = ctx.link().callback(|_| Msg::Delegate);
                    let delegator = match self.client.get_delegation_info() {
                        Some(info) => Some(DelegationInfoProp::new(info.delegator_pubkey, info.created_after(), info.created_before(), info.kinds())),
                        None => None
                    };

                    let remote_signer = self.client.get_remote_signer().map(|p| AttrValue::from(p.to_string()));

                    html!{
                    <>
                    if let Some(event_id) = &self.broadcasted_event {
                        <p>{ format!("Broadcasted event: {}", event_id)}</p>
                    }
                    <Home {note_cb} {delegate_cb} {delegator} {remote_signer}/>
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
                        Some(info) => Some(DelegationInfoProp::new(info.delegator_pubkey, info.created_after(), info.created_before(), info.kinds())),
                        None => None
                    };

                    let update_connect_relay_cb = ctx.link().callback(Msg::UpdateConnectRelay);
                    let add_relay_cb = ctx.link().callback(Msg::AddRelay);
                    let props = props! {
                        SettingsProps {
                            app_pubkey: self.client.get_app_pubkey().to_bech32().unwrap(),
                            delegation_info: delegation_info,
                            connect_relay: self.client.get_connect_relay().to_string(),
                            relays: self.client.get_relays().clone(),
                            update_connect_relay_cb,
                            add_relay_cb
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
