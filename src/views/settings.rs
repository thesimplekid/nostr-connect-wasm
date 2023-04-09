use std::collections::HashSet;

use log::debug;
use nostr_sdk::{nips::nip19::ToBech32, secp256k1::XOnlyPublicKey, url::Url};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(PartialEq, Default, Clone)]
pub struct DelegationInfoProp {
    pub delegator_pubkey: AttrValue,
    pub valid_from: AttrValue,
    pub valid_to: AttrValue,
    pub kinds: AttrValue,
}

impl DelegationInfoProp {
    pub fn new(
        pubkey: XOnlyPublicKey,
        from: Option<u64>,
        to: Option<u64>,
        kinds: Vec<u64>,
    ) -> Self {
        let valid_from = match from {
            Some(value) => value.to_string().into(),
            None => "Not set".into(),
        };

        let valid_to = match to {
            Some(value) => value.to_string().into(),
            None => "Not set".into(),
        };

        let delegator_pubkey = match pubkey.to_bech32() {
            Ok(key) => key.into(),
            Err(_) => pubkey.to_string().into(),
        };

        let kinds = kinds
            .iter()
            .map(|k| k.to_string())
            .collect::<Vec<_>>()
            .join(", ")
            .into();

        Self {
            delegator_pubkey,
            valid_from,
            valid_to,
            kinds,
        }
    }
}

#[derive(Properties, PartialEq, Default, Clone)]
pub struct Props {
    pub app_pubkey: AttrValue,
    pub delegation_info: Option<DelegationInfoProp>,
    pub connect_relay: AttrValue,
    pub relays: HashSet<Url>,
    pub update_connect_relay_cb: Callback<AttrValue>,
    pub add_relay_cb: Callback<AttrValue>,
    pub logout_cb: Callback<MouseEvent>,
    pub remove_relay_cb: Callback<Url>,
}

pub enum Msg {
    UpdateConnectRelay,
    AddRelay,
    DeleteRelay(Url),
}

pub struct Settings {
    connect_relay: NodeRef,
    new_relay: NodeRef,
}

impl Component for Settings {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            connect_relay: NodeRef::default(),
            new_relay: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateConnectRelay => {
                if let Some(relay) = self.connect_relay.clone().cast::<HtmlInputElement>() {
                    debug!("{}", relay.value());
                    ctx.props()
                        .update_connect_relay_cb
                        .emit(relay.value().into());
                }
            }
            Msg::AddRelay => {
                if let Some(relay) = self.new_relay.clone().cast::<HtmlInputElement>() {
                    debug!("{}", relay.value());
                    ctx.props().add_relay_cb.emit(relay.value().into());
                }
            }
            Msg::DeleteRelay(relay) => {
                ctx.props().remove_relay_cb.emit(relay);
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let update_connect_relay = ctx.link().callback(|_| Msg::UpdateConnectRelay);
        let add_relay = ctx.link().callback(|_| Msg::AddRelay);
        html! {
            <>
            <h2 class="text-4xl font-extrabold dark:text-white">{ "Settings" }</h2>
            // Display name and key of delgator
            if let Some(delegator_info) = ctx.props().clone().delegation_info {
                <div class="flex items-center space-x-4">
                    <img class="w-10 h-10 rounded-full" src="/docs/images/people/profile-picture-5.jpg" alt=""/ >
                    <div class="font-medium dark:text-white">
                        // TODO: Pull this from profile
                        <div>{ "John Doe" }</div>
                        <div class="text-sm text-gray-500 dark:text-gray-400"> {delegator_info.delegator_pubkey } </div>
                    </div>
                </div>

                <div>
                    <p class="text-2xl text-gray-900 dark:text-white">{ "Delegation" }</p>
                    <p class="text-base text-gray-900 dark:text-white">{ format!("Valid: {} - {}", delegator_info.valid_from, delegator_info.valid_to) } </p>

                </div>
            }
            // Text box of connect relay that is editable
            <div class="mb-6">
                <label for="default-input" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{"Connect Relay"}</label>
                <input type="text" id="default-input" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" value={ctx.props().connect_relay.clone()} ref={self.connect_relay.clone()}/>
            <button type="button" class="focus:outline-none text-white bg-purple-700 hover:bg-purple-800 focus:ring-4 focus:ring-purple-300 font-medium rounded-lg text-sm px-5 py-2.5 mb-2 dark:bg-purple-600 dark:hover:bg-purple-700 dark:focus:ring-purple-900" onclick={update_connect_relay}>{ "update connect relay" } </button>

            </div>
            // List of publish relays with delete buttons
            <h2 class="mb-2 text-lg font-semibold text-gray-900 dark:text-white"> { "Publish relays" }</h2>
            <ul class="max-w-md space-y-1 text-gray-500 list-inside dark:text-gray-400">
                {
                    ctx.props().relays.clone().into_iter().map(|relay| {
                       let r = relay.clone();
                        let c = ctx.link().callback(move |_e| Msg::DeleteRelay(r.clone()));
                        html!{
                        <li>
                            <div> {format!("{} ", relay)}
                                <button type="button" class="text-purple-700 border border-purple-700 hover:bg-purple-700 hover:text-white focus:ring-4 focus:outline-none focus:ring-purple-300 font-medium rounded-lg text-sm p-1 text-center inline-flex items-center mr-3 dark:border-purple-500 dark:text-purple-500 dark:hover:text-white dark:focus:ring-purple-800 dark:hover:bg-purple-500" onclick={c}>
                                    <svg aria-hidden="true" fill="none" class="w-5 h-5" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                                        <path d="M15 12H9m12 0a9 9 0 11-18 0 9 9 0 0118 0z" stroke-linecap="round" stroke-linejoin="round"></path>
                                    </svg>
                                    <span class="sr-only">{"Icon description"}</span>
                                </button>
                            </div>
                        </li> }
                    }).collect::<Html>()
                }
            </ul>

            // Text box to add relays
            <div class="mb-6">
                <label for="default-input" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{ "Add Relay" }</label>
                // TODO: add call back
                <input type="text" id="default-input" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" ref={self.new_relay.clone()}/>
            <button type="button" class="focus:outline-none text-white bg-purple-700 hover:bg-purple-800 focus:ring-4 focus:ring-purple-300 font-medium rounded-lg text-sm px-5 py-2.5 mb-2 dark:bg-purple-600 dark:hover:bg-purple-700 dark:focus:ring-purple-900" onclick={add_relay}>{ "Add Relay" } </button>
            </div>

            // Log out button
            // TODO: add call back
            <button type="button" class="focus:outline-none text-white bg-purple-700 hover:bg-purple-800 focus:ring-4 focus:ring-purple-300 font-medium rounded-lg text-sm px-5 py-2.5 mb-2 dark:bg-purple-600 dark:hover:bg-purple-700 dark:focus:ring-purple-900" onclick={ctx.props().logout_cb.clone()}>{ "Log out" } </button>
            </>
        }
    }
}
