use dashmap::DashSet;
use nostr_sdk::Url;
use std::str::FromStr;

use yew::html::Scope;
use yew::prelude::*;
use yew_router::prelude::*;

use log::debug;

use crate::pages::connect::Props as ConnectProps;
use crate::pages::{connect::Connect, home::Home};
use crate::services::nostr::NostrService;
use crate::utils::handle_keys;

pub enum View {
    Home,
    Connect,
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
}

pub struct App {
    view: View,
    //navbar_active: bool,
    client: NostrService,
    broadcasted_event: Option<AttrValue>,
    props: ConnectProps,
}
impl Component for App {
    type Message = Msg;
    type Properties = ConnectProps;

    fn create(ctx: &Context<Self>) -> Self {
        let relays = vec![Url::from_str("ws://localhost:8081").unwrap()];
        let keys = handle_keys(None, true).unwrap();

        let client = NostrService::new(&keys, relays[0].clone()).unwrap();

        client.add_relay(relays[0].clone()).ok();

        let signer_pubkey_callaback = ctx.link().callback(|_| Msg::ReceivedPubkey);

        client.get_signer_pub_key(signer_pubkey_callaback).unwrap();

        let co_cb = ctx.link().callback(Msg::NoteView);
        let set_relay_callback = ctx.link().callback(Msg::SetRelay);

        Self {
            // navbar_active: false,
            client,
            view: View::Connect,
            broadcasted_event: None,
            // HACK: Props here feels like a hack but not sure
            props: ConnectProps {
                pubkey: keys.public_key().to_string().into(),
                relay: relays,
                name: "dartstr".into(),
                connected_cb: co_cb,
                set_relay_cb: set_relay_callback,
            },
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
                self.client.get_delegate(delegate_callback).ok();
                true
            }
            Msg::DelegationSet => {
                debug!("Delegation set");
                self.client.create_client(DashSet::new()).ok();
                false
            }
            Msg::ReceivedPubkey => {
                self.view = View::Home;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let note_cb = ctx.link().callback(Msg::SubmitNote);
        let delegate_cb = ctx.link().callback(|_| Msg::Delegate);
        html! {
            <BrowserRouter>
                { self.view_nav(ctx.link()) }

                <main>
                {
                    match self.view {
                        View::Home => html!{

                            <>
                            if let Some(event_id) = &self.broadcasted_event {
                                <p>{ format!("Broadcasted event: {}", event_id)}</p>

                            }
                            <Home {note_cb} {delegate_cb}/>
                            </>
                        },
                        View::Connect => html! { <Connect ..self.props.clone() /> }
                    }
                }

                </main>
                <footer class="footer">
                </footer>
            </BrowserRouter>
        }
    }
}
impl App {
    fn view_nav(&self, _link: &Scope<Self>) -> Html {
        html! {}
    }
}
