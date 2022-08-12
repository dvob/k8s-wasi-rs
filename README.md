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
use k8s_openapi::api::authentication::v1::TokenReview;
use k8s_wasi::Authenticator;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize)]
struct Settings {
    my_setting: String,
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
    fn authenticate(tr: TokenReview, settings: Settings) -> Result<TokenReview, Box<dyn Error>> {
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

## Authentication
The module `k8s_wasi::token_review` contains helper functions for the authentication:
```rust
use k8s_wasi::token_review::*;
```

### Input
Read the token from the token review:
```rust
let token = get_token(tr)?;
```

### Output
Authenticate with UID `0`, user `magic-user` and group `magic-group`:
```rust
response_from_status(
	authenticate(
		"0".to_string(),
                "magic-user".to_string(),
                vec![
			"magic-group".to_string()
		],
	)
)
```

Do not authenticate:
```rust
response_from_status(reject())
```

## Authorization
The module `k8s_wasi::subject_access_review` contains helper functions to construct a `SubjectAccessReview` easily.
```rust
use k8s_wasi::subject_access_review::*;
```

### Output
Authorize:
```rust
response_from_status(allow())
```

Do not authorize:
```rust
response_from_status(reject())
```

## Admission
The module `k8s_wasi::admission` contains types and functions to construct a `AdmissionReview` easily.
```rust
use k8s_wasi::admission::*;
```

### Input
Read the request:
```rust
let mut request = ar.get_request()?;
```

Read certain object from AdmissionReviewRequest:
```rust
use k8s_openapi::api::core::v1::ConfigMap;

let config_map: ConfigMap = request.get_object()?;
```

### Output
Accept request:
```rust
AdmissionReview::accept(request.uid)
```

Accept request and mutate object:
```rust
AdmissionReview::mutate(request.uid, config_map)
```

Reject request:
```rust
AdmissionReview::reject_with_message(
	request.uid,
	format!("reason for rejection")
)
```
