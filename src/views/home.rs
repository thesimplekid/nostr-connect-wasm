use web_sys::HtmlInputElement;
use yew::prelude::*;

use super::settings::DelegationInfoProp;

enum State {
    // NotConnected,
    Connected,
}
pub enum Msg {
    SubmitNote(String),
}

pub struct Home {
    state: State,
    note_text: NodeRef,
}
#[derive(Properties, PartialEq, Default, Clone)]
pub struct Props {
    pub note_cb: Callback<AttrValue>,
    pub delegator: Option<DelegationInfoProp>,
    pub remote_signer: Option<AttrValue>,
}

impl Component for Home {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            state: State::Connected,
            note_text: NodeRef::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SubmitNote(_value) => {}
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.state {
            // State::NotConnected => self.redirect_connect(&ctx),
            State::Connected => self.connected(ctx),
        }
    }
}
impl Home {
    /*
    fn redirect_connect(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <button onclick={ctx.link().callback(|_| Msg::RedirectConnect)}>
                    { "Connect" }
                </button>
            </>

        }
    }
    */

    fn connected(&self, ctx: &Context<Self>) -> Html {
        let h = self.note_text.cast::<HtmlInputElement>();
        let cb = ctx.props().note_cb.clone();
        let onsubmit = ctx.link().callback(move |e: SubmitEvent| {
            e.prevent_default();
            if let Some(input) = &h {
                let value = input.value();
                // do something with value
                cb.emit(value.into());
            }

            Msg::SubmitNote("".to_string())
        });

        html! {
            <>
            <form {onsubmit}>

                <label for="message" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{ "Text note" }</label>
                <textarea id="message" rows="4" class="block p-2.5 w-full text-sm text-gray-900 bg-gray-50 rounded-lg border border-gray-300 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="Write your thoughts here..." ref={self.note_text.clone()}></textarea>

                <br/>
                <button type="submit" value="submit" class="focus:outline-none text-white bg-purple-700 hover:bg-purple-800 focus:ring-4 focus:ring-purple-300 font-medium rounded-lg text-sm px-5 py-2.5 mb-2 dark:bg-purple-600 dark:hover:bg-purple-700 dark:focus:ring-purple-900">{ "Publish" }</button>
            </form>


            // TODO: Show what key events are being sent with

            // Option 1: using the remote signer
            // Show pubkey of signer
            {
            if let Some(delegator) = &ctx.props().delegator {
                html! {
                    <>
                    <p class="text-4xl text-gray-900 font-extralight dark:text-white">{ format!("Publishing events with delegation from: {}", delegator.delegator_pubkey) }</p>
                    <p class="text-4xl text-gray-900 font-extralight dark:text-white">{ format!("Delegated from {} to {}", delegator.valid_from, delegator.valid_to) }</p>
                    <p class="text-4xl text-gray-900 font-extralight dark:text-white">{ format!("Valid for kinds: {}", delegator.kinds) }</p>
                    </>


                }
            } else if let Some(remote_signer) = &ctx.props().remote_signer {
                html! {
                    <>
                    <p class="text-4xl text-gray-900 font-extralight dark:text-white">{ format!("Publishing events with remote signer {}", remote_signer) }</p>
                    </>
                }
            } else {
                html!{"Not sure how you made it here, refresh I guess?"}
            }
            }




            // Option 2: Delegation
            // Show delegator and conditions
            </>

        }
    }
}
