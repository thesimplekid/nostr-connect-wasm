use std::str::FromStr;

use nostr_sdk::nips::nip46::NostrConnectURI;
use nostr_sdk::secp256k1::XOnlyPublicKey;
use nostr_sdk::url::Url;
use qrcode::render::svg;
use qrcode::QrCode;
use yew::prelude::*;
use yew::virtual_dom::VNode;

#[derive(Debug)]
enum State {
    Connect,
}

/*
pub enum Msg {
    CopyConnect,
}
*/

#[derive(Properties, PartialEq, Default, Clone)]
pub struct Props {
    #[prop_or_default]
    pub pubkey: AttrValue,
    #[prop_or_default]
    pub connect_relay: AttrValue,
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
}
impl Component for Connect {
    type Message = ();
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let connect_uri = NostrConnectURI::new(
            XOnlyPublicKey::from_str(ctx.props().pubkey.as_str()).unwrap(),
            Url::from_str(&ctx.props().connect_relay).unwrap(),
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
        }
    }

    /*
        fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
            match msg {
                Msg::CopyConnect => false,
            }
        }
    */
    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.state {
            State::Connect => self.connect_info(ctx),
        }
    }
}
impl Connect {
    fn connect_info(&self, _ctx: &Context<Self>) -> Html {
        html! {
        <>
        <div class="flex justify-center">
           <div class="mt-10 max-w-sm p-6 bg-white border border-gray-200 rounded-lg shadow dark:bg-gray-800 dark:border-gray-700">
              <div class="relative flex justify-center">
                 { self.connect_qr.clone() }
              </div>
              <div class="relative">
                 <input class="block w-full p-4 text-sm text-gray-900 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" readonly=true value={ self.connect_string.clone() }/>
                 // TODO: should be a button to copy
                 // <button type="button" class="text-white absolute right-2.5 bottom-2.5 bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-4 py-2 dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800" onclick={ctx.link().callback(|_| Msg::CopyConnect)}>{"Copy"}</button>
              </div>
           </div>
        </div>
        </>
        }
    }
}
