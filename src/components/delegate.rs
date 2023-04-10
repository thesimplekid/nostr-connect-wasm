use log::debug;
use nostr_sdk::Timestamp;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq, Default, Clone)]
pub struct Props {
    pub delegate_cb: Callback<(u64, Vec<u64>)>,
}

pub struct Delegate {
    for_hours: NodeRef,
    for_min: NodeRef,
    // kinds: NodeRef,
}

pub enum Msg {
    Delegate,
}

impl Component for Delegate {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            for_hours: NodeRef::default(),
            for_min: NodeRef::default(),
            // kinds: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Delegate => {
                debug!("Delegate");
                if let (Some(hours), Some(mins)) = (
                    self.for_hours.cast::<HtmlInputElement>(),
                    self.for_min.cast::<HtmlInputElement>(),
                ) {
                    let hours = hours.value_as_number();
                    let mins = mins.value_as_number();

                    let expiration =
                        Timestamp::now().as_u64() + (hours as u64 * 3600) + (mins as u64 * 60);
                    ctx.props().delegate_cb.emit((expiration, vec![1]));
                }

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|_| Msg::Delegate);
        html! {
            <>
            <p class="tracking-tight text-gray-500 md:text-lg dark:text-gray-400">{ "Create a delegation token, valid for: " }</p>
              <div class="flex">
                  <div class="relative z-0 p-4 mb-6 group">
                      <input type="number" name="floating_hours" id="floating_hours" class="block py-2.5 px-0 w-full text-sm text-gray-900 bg-transparent border-0 border-b-2 border-gray-300 appearance-none dark:text-white dark:border-gray-600 dark:focus:border-blue-500 focus:outline-none focus:ring-0 focus:border-blue-600 peer" placeholder=" " ref={self.for_hours.clone()}/>
                      <label for="floating_hours" class="peer-focus:font-medium absolute text-sm text-gray-500 dark:text-gray-400 duration-300 transform -translate-y-6 scale-75 top-3 -z-10 origin-[0] peer-focus:left-0 peer-focus:text-blue-600 peer-focus:dark:text-blue-500 peer-placeholder-shown:scale-100 peer-placeholder-shown:translate-y-0 peer-focus:scale-75 peer-focus:-translate-y-6">{"Hours"}</label>
                  </div>
                   <div class="relative z-0 p-4 mb-6 group">
                      <input type="number" name="floating_mins" id="floating_mins" class="block py-2.5 px-0 w-full text-sm text-gray-900 bg-transparent border-0 border-b-2 border-gray-300 appearance-none dark:text-white dark:border-gray-600 dark:focus:border-blue-500 focus:outline-none focus:ring-0 focus:border-blue-600 peer" placeholder=" " ref={self.for_min.clone()}/>
                      <label for="floating_mins" class="peer-focus:font-medium absolute text-sm text-gray-500 dark:text-gray-400 duration-300 transform -translate-y-6 scale-75 top-3 -z-10 origin-[0] peer-focus:left-0 peer-focus:text-blue-600 peer-focus:dark:text-blue-500 peer-placeholder-shown:scale-100 peer-placeholder-shown:translate-y-0 peer-focus:scale-75 peer-focus:-translate-y-6">{"Minutes"}</label>
                  </div>
                   <div class="relative z-0 p-4 mb-6 group">
                        <button type="button" class="focus:outline-none text-white bg-purple-700 hover:bg-purple-800 focus:ring-4 focus:ring-purple-300 font-medium rounded-lg text-sm px-5 py-2.5 mb-2 dark:bg-purple-600 dark:hover:bg-purple-700 dark:focus:ring-purple-900" {onclick}>{ "Delegate" } </button>
                    </div>
                </div>
            </>
        }
    }
}
