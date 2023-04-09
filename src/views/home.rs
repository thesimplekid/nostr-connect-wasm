use web_sys::HtmlInputElement;
use yew::prelude::*;

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
    pub delegate_cb: Callback<MouseEvent>,
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

            <button class="px-8 py-3 font-semibold rounded dark:bg-gray-100 dark:text-gray-800" onclick={ctx.props().delegate_cb.clone()}>{ "Delegate" } </button>
            </>

        }
    }
}
