### Nostr Connect (NIP46) WASM

An almost useless [NIP46](https://github.com/nostr-protocol/nips/blob/master/46.md) example using [yew.rs](https://yew.rs/) with delegation



### Building
```
cargo install trunk
rustup target add wasm32-unknown-unknown
```

Install tailwind 
https://tailwindcss.com/docs/installation

## Run
```
trunk serve --open
```


For now the best way to test with delegation is with nostr-tools https://github.com/thesimplekid/nostr-tool/tree/nostr-connect. 