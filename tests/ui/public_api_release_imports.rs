//! Recommended imports: SDK entry point, errors, API roots, primitives, NOM models, and
//! wallet/key types importable directly from `znn_sdk_rust`.

#![allow(unused_imports, dead_code)]

use znn_sdk_rust::{
    AccountBlockTemplate, Address, BlockType, ClientTransport, EmbeddedApi, Error, Hash, HashHeight,
    HttpClient, KeyPair, KeyStore, LedgerApi, NativePowProvider, PageQuery, PowFuture, PowProvider,
    StatsApi, SubscribeApi, TokenStandard, Zenon,
};

fn main() {
    let _ = std::any::type_name::<Zenon>();
    let _ = std::any::type_name::<Error>();
    let _ = std::any::type_name::<Address>();
    let _ = std::any::type_name::<Hash>();
    let _ = std::any::type_name::<HashHeight>();
    let _ = std::any::type_name::<TokenStandard>();
    let _ = std::any::type_name::<AccountBlockTemplate>();
    let _ = std::any::type_name::<BlockType>();
    let _ = std::any::type_name::<KeyPair>();
    let _ = std::any::type_name::<KeyStore>();
    let _ = std::any::type_name::<LedgerApi>();
    let _ = std::any::type_name::<StatsApi>();
    let _ = std::any::type_name::<SubscribeApi>();
    let _ = std::any::type_name::<EmbeddedApi>();
    let _ = std::any::type_name::<PageQuery>();
    let _ = std::any::type_name::<PowFuture<'static>>();
    let _ = std::any::type_name::<&dyn PowProvider>();
    let _ = std::any::type_name::<NativePowProvider>();
    let _ = std::any::type_name::<HttpClient>();
    let _ = std::any::type_name::<ClientTransport>();
    // `new_client` is a function re-exported at the crate root; the `use`
    // import above proves it is importable, and this function-pointer type
    // pins its signature.
    let _ =
        std::any::type_name::<fn(&str) -> Result<ClientTransport, znn_sdk_rust::client::exceptions::ClientError>>();
}
