use anyhow::{anyhow, Context, Result};
use js_sys::futures::JsFuture;
use js_sys::{Array, Object, Reflect};
use wasm_bindgen::JsCast;
use web_sys::{window, CryptoKey};

use crate::types::KeyPair;

pub async fn generate_keypair() -> Result<KeyPair> {
    let subtle = window()
        .context("window is not available")?
        .crypto()
        .map_err(|e| anyhow!("Failed to access Web Crypto API: {e:?}"))?
        .subtle();

    let algorithm = Object::new();

    Reflect::set(&algorithm, &"name".into(), &"RSA-OAEP".into())
        .map_err(|e| anyhow!("Failed to set algorithm name: {e:?}"))?;

    Reflect::set(&algorithm, &"modulusLength".into(), &2048.into())
        .map_err(|e| anyhow!("Failed to set modulus length: {e:?}"))?;

    Reflect::set(
        &algorithm,
        &"publicExponent".into(),
        &js_sys::Uint8Array::from([1, 0, 1].as_slice()),
    )
    .map_err(|e| anyhow!("Failed to set public exponent: {e:?}"))?;

    Reflect::set(&algorithm, &"hash".into(), &"SHA-256".into())
        .map_err(|e| anyhow!("Failed to set hash algorithm: {e:?}"))?;

    let usages = Array::new();
    usages.push(&"encrypt".into());
    usages.push(&"decrypt".into());

    let pair = JsFuture::from(
        subtle
            .generate_key_with_object(&algorithm, true, &usages)
            .map_err(|e| anyhow!("Failed to generate key pair: {e:?}"))?,
    )
    .await
    .map_err(|e| anyhow!("Key generation promise rejected: {e:?}"))?;

    let public: CryptoKey = Reflect::get(&pair, &"publicKey".into())
        .map_err(|e| anyhow!("Missing public key: {e:?}"))?
        .dyn_into()
        .map_err(|_| anyhow!("Failed to cast public key"))?;

    let private: CryptoKey = Reflect::get(&pair, &"privateKey".into())
        .map_err(|e| anyhow!("Missing private key: {e:?}"))?
        .dyn_into()
        .map_err(|_| anyhow!("Failed to cast private key"))?;

    let public = JsFuture::from(
        subtle
            .export_key("spki", &public)
            .map_err(|e| anyhow!("Failed to export public key: {e:?}"))?,
    )
    .await
    .map_err(|e| anyhow!("Failed to export public key: {e:?}"))?;

    let private = JsFuture::from(
        subtle
            .export_key("pkcs8", &private)
            .map_err(|e| anyhow!("Failed to export private key: {e:?}"))?,
    )
    .await
    .map_err(|e| anyhow!("Failed to export private key: {e:?}"))?;

    Ok(KeyPair {
        public: js_sys::Uint8Array::new(&public).to_vec(),
        private: js_sys::Uint8Array::new(&private).to_vec(),
    })
}
