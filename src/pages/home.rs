use web_sys::{HtmlInputElement, Window};
use yew::prelude::*;

enum State {
    // NotConnected,
    Connected,
}
pub enum Msg {
    RedirectConnect,
    SubmitNote(String),
}

pub struct Home {
    state: State,
    note_text: NodeRef,
}
#[derive(Properties, PartialEq, Default, Clone)]
pub struct Props {
    pub note_cb: Callback<AttrValue>,
    pub delegate_cb: Callback<AttrValue>,
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
            Msg::RedirectConnect => {
                // HACK: I think this is a hack and would rather us the router
                let window: Window = web_sys::window().expect("window not available");
                let location = window.location();
                let _ = location.set_href("/connect");
            }
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

        let delegate_cb = ctx.props().delegate_cb.reform(move |_| "".into());

        html! {
            <>
            <form {onsubmit}>
                <label for="name">{ "Content" } </label>
                <textarea type="text" ref={self.note_text.clone()}/>
                <br/>
                  <input type="submit" value="Submit"/>
            </form>

            <button onclick={delegate_cb}>{ "Delegate" } </button>
            </>

        }
    }
}
