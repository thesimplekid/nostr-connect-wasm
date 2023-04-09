use std::collections::HashSet;
use std::str::FromStr;

use dashmap::DashSet;
use log::debug;
use nostr_sdk::prelude::ToBech32;
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
    // ToggleNavbar,
    SubmitNote(AttrValue),
    NoteView(AttrValue),
    SetRelay(AttrValue),
    BroadcastedEvent(AttrValue),
    Delegate,
    ReceivedPubkey,
    DelegationSet,
    Settings,
    DelegationInfo(DelegationInfo),
    Home,
}

pub struct App {
    view: View,
    //navbar_active: bool,
    client: NostrService,
    broadcasted_event: Option<AttrValue>,
    connect_relay: Url,
    publish_relays: HashSet<Url>,
    name: AttrValue,
}
impl Component for App {
    type Message = Msg;
    type Properties = ConnectProps;

    fn create(ctx: &Context<Self>) -> Self {
        let connect_relay = Url::from_str("ws://localhost:8081").unwrap();
        let relays = HashSet::from_iter(vec![connect_relay.clone()]);
        let keys = handle_keys(None, true).unwrap();

        let client = NostrService::new(&keys, connect_relay.clone()).unwrap();

        client.add_relay(connect_relay.clone()).ok();

        let signer_pubkey_callback = ctx.link().callback(|_| Msg::ReceivedPubkey);

        client.get_signer_pub_key(signer_pubkey_callback).unwrap();

        Self {
            // navbar_active: false,
            client,
            view: View::Connect,
            broadcasted_event: None,
            connect_relay,
            publish_relays: relays,
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
            Msg::SetRelay(relay) => {
                debug!("Setting relay: {relay}");
                let relay = Url::from_str(&relay).unwrap();
                self.client.add_relay(relay).ok();
                false
            }
            Msg::NoteView(_) => {
                self.view = View::Home;
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
                false
            }
            Msg::ReceivedPubkey => {
                self.view = View::Home;
                true
            }
            Msg::Settings => {
                self.view = View::Settings;
                true
            }
            Msg::Home => {
                self.view = View::Home;
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

                    html!{
                    <>
                    if let Some(event_id) = &self.broadcasted_event {
                        <p>{ format!("Broadcasted event: {}", event_id)}</p>
                    }
                    <Home {note_cb} {delegate_cb}/>
                    </>
                }
            },
                View::Connect => {
                    let connected_cb = ctx.link().callback(Msg::NoteView);
                    let set_relay_cb = ctx.link().callback(Msg::SetRelay);
                    let props = props! {
                        ConnectProps {
                            pubkey: self.client.get_app_pubkey().to_string(),
                            connect_relay: self.connect_relay.to_string(),
                            name: self.name.clone(),
                            connected_cb,
                            set_relay_cb
                        }
                    };

                    html! { <Connect .. props /> }
                }
                View::Settings => {

                    let delegation_info = match self.client.get_delegation_info() {
                        Some(info) => Some(DelegationInfoProp::new(info.delegator_pubkey, info.created_after(), info.created_before())),
                        None => None
                    };
                    let props = props! {
                        SettingsProps {
                            app_pubkey: self.client.get_app_pubkey().to_bech32().unwrap(),
                            delegation_info: delegation_info,
                            connect_relay: self.connect_relay.to_string(),
                            relays: self.publish_relays.clone()
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
