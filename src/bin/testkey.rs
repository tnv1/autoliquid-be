use sui_types::{
    base_types::SuiAddress,
    crypto::{EncodeDecodeBase64, SuiKeyPair, get_key_pair},
};

fn main() {
    // Generate a new Ed25519 keypair
    let keypair = SuiKeyPair::Ed25519(get_key_pair().1);

    // Get the public key encoded as a base64 string
    let public_key_base64 = keypair.public().encode_base64();
    println!("Public Key (Base64): {}", public_key_base64);

    // Optional: Export private key as bytes (careful with this!)
    let private_bytes = keypair.encode().unwrap();
    println!("Private Key (bytes): {:?}", private_bytes);

    let kp = SuiKeyPair::decode("<private-key>").unwrap();
    println!("Public Key (Base64): {}", kp.public().encode_base64());

    let addr = SuiAddress::from(&kp.public());
    println!("Address: {}", addr.to_string());
}
