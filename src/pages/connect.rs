use std::str::FromStr;

use log::debug;
use nostr_sdk::nips::nip46::NostrConnectURI;
use nostr_sdk::secp256k1::XOnlyPublicKey;
use nostr_sdk::url::Url;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use qrcode::render::svg;
use qrcode::QrCode;
use yew::virtual_dom::VNode;

#[derive(Debug)]
enum State {
    // SetRelay,
    Connect,
}

pub enum Msg {
    ConnectPress,
    SubmitRelay,
}

#[derive(Properties, PartialEq, Default, Clone)]
pub struct Props {
    #[prop_or_default]
    pub pubkey: AttrValue,
    #[prop_or_default]
    pub relay: Vec<Url>,
    #[prop_or_default]
    pub name: AttrValue,
    pub connected_cb: Callback<AttrValue>,
    pub set_relay_cb: Callback<AttrValue>,
}

#[derive(Debug)]
pub struct Connect {
    state: State,
    connect_string: Option<String>,
    connect_qr: Option<VNode>,
    relay_ref: NodeRef,
}
impl Component for Connect {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let connect_uri = NostrConnectURI::new(
            XOnlyPublicKey::from_str(ctx.props().pubkey.as_str()).unwrap(),
            ctx.props().relay[0].clone(),
            ctx.props().name.to_string(),
        );

        let qr_svg = QrCode::new(connect_uri.to_string().as_bytes())
            .unwrap()
            .render()
            .min_dimensions(200, 200)
            .dark_color(svg::Color("#800000"))
            .light_color(svg::Color("#ffff80"))
            .build();

        // escapes the string to make it html
        let connect_svg = Html::from_html_unchecked(AttrValue::from(qr_svg));
        Self {
            state: State::Connect,
            connect_string: Some(connect_uri.to_string()),
            connect_qr: Some(connect_svg),
            relay_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ConnectPress => {
                let call = ctx.props().connected_cb.clone();
                call.emit("".into());
            }
            Msg::SubmitRelay => {
                debug!("REF: {:?}", self.relay_ref);
                let h = self.relay_ref.cast::<HtmlInputElement>();
                if let Some(input) = &h {
                    let value = input.value();
                    debug!("{value}");

                    // Emit message to add relay to client
                    ctx.props().set_relay_cb.emit(value.into());

                    let connect_uri = NostrConnectURI::new(
                        XOnlyPublicKey::from_str(ctx.props().pubkey.as_str()).unwrap(),
                        ctx.props().relay[0].clone(),
                        ctx.props().name.to_string(),
                    );

                    let qr_svg = QrCode::new(connect_uri.to_string().as_bytes())
                        .unwrap()
                        .render()
                        .min_dimensions(200, 200)
                        .dark_color(svg::Color("#800000"))
                        .light_color(svg::Color("#ffff80"))
                        .build();

                    // escapes the string to make it html
                    let connect_svg = Html::from_html_unchecked(AttrValue::from(qr_svg));

                    self.state = State::Connect;
                    self.connect_qr = Some(connect_svg);
                    self.connect_string = Some(connect_uri.to_string());
                }
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.state {
            // State::SetRelay => self.set_relay(ctx),
            State::Connect => self.connect_info(ctx),
        }
    }
}
impl Connect {
    fn _set_relay(&self, ctx: &Context<Self>) -> Html {
        html! {
        <>
            <input
            type="text"
            ref={self.relay_ref.clone()}
            />
            <button onclick={ctx.link().callback(|_| Msg::SubmitRelay)}> {"Submit Relay"}</button>
        </>
        }
    }

    fn connect_info(&self, ctx: &Context<Self>) -> Html {
        html! {

        <main>
            <h1>{ "Hello World!" }</h1>
            <div>
            { self.connect_qr.clone() }
            <p>{ self.connect_string.clone() }</p>
            </div>
            <button onclick={ctx.link().callback(|_| Msg::ConnectPress)}> {"Connected"}</button>
        </main>

        }
    }
}
