# k8s_wasi

The `k8s_wasi` crate provides helper functions to easily implement a WASM module according to specification described [here](https://github.com/dvob/k8s-wasm/tree/main/spec).

The quickest way to get started is to copy one of the examples in the `./examples` directory and change it according to your needs.

## Create a module

Create a new crate:
```
cargo new --lib my-k8s-module
```

Add the required dependencies to your module in `Cargo.toml`:
```toml
[dependencies]
k8s_wasi = { git = "https://github.com/dvob/k8s-wasi-rs" }
k8s-openapi = { version = "0.15.0", features = ["v1_24"] }
serde = { version = "1.0.139", features = ["derive"] }
serde_json = "1.0.82"
```

Set the `crate-type` in `Cargo.toml`:
```toml
[lib]
crate-type = ["cdylib"]
```

Implement your module in `src/lib.rs`.
Define a struct which represents your component and one which represent the settings if you expect settings.
We must be able to deserialize the settings hence we use `#[derive(Deserialize)]` on the settings.
```rust
#[derive(Deserialize)]
struct Settings {
    my_setting: String
}

struct MyAuthenticator {}
```

Then implement the appropriate trait for your component:
* Authentication: [`k8s_wasi::Authenticator<S>`](https://github.com/dvob/k8s-wasi-rs/blob/main/k8s_wasi/src/lib.rs#L84)
* Authorization: [`k8s_wasi::Authorizer<S>`](https://github.com/dvob/k8s-wasi-rs/blob/main/k8s_wasi/src/lib.rs#L95)
* Admission: [`k8s_wasi::Admiter<S>`](https://github.com/dvob/k8s-wasi-rs/blob/main/k8s_wasi/src/lib.rs#L109)

The traits are generic over the settings (`S`):
```rust
impl Authenticator<Settings> for MyAuthenticator {
    fn authenticate(tr: TokenReview, settings: Settings) -> Result<TokenReview, Box<dyn std::error::Error>> {
        todo!()
    }
}
```

Then implement the logic according to your needs.

Finally you have to provide the appropriate function (`authn`, `authz`, `validate`) which is required by the specification.
For this you can either implement the function yourself or use the appropriate register macro:
* Authentication: `k8s_wasi::register_authenticator!`
* Authorization: `k8s_wasi::register_authenticator!`
* Admission: `k8s_wasi::register_authenticator!`

Macro:
```rust
k8s_wasi::register_authenticator!(MyAuthenticator);
```

Implement yourself:
```rust
#[no_mangle]
fn authn() {
  MyAuthenticator::runner().run_with_stdin().unwrap();
}
```

To build the module you have to install the `wasm32-wasi` target.
You can install with [rustup](https://rustup.rs/) like this:
```
rustup target add wasm32-wasi
```

Then you can build the module:
```
cargo build --target wasm32-wasi
```

For the final build it is recommended that you use `--release` flag for build since this produces a much smaller module:
```
cargo build --release --target wasm32-wasi
```

Then you can find the module in the target folder under:
```
target/wasm32-wasi/release/my_k8s_authenticator.wasm
```
