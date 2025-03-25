use autoliquid_be::services::dex::{BluefinDex, DexInterface, OpenPositionOptions};
use sui_types::{base_types::SuiAddress, crypto::SuiKeyPair};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let url = "https://fullnode.mainnet.sui.io:443";
    let package_id = "0x3492c874c1e3b3e2984e8c41b589e642d4d0a5d6459e5a9cfc2d52fd7c89c267";
    let global_config = "0x03db251ba509a8d5d8777b6338836082335d93eecbdd09a11e190a1cff51c352";
    let bluefin_dex =
        BluefinDex::new(url.to_string(), package_id.to_string(), global_config.to_string()).await;

    let kp = SuiKeyPair::decode("<private-key>").unwrap();
    let addr = SuiAddress::from(&kp.public());
    tracing::info!("Address: {}", addr.to_string());

    bluefin_dex
        .open_position(
            &kp,
            OpenPositionOptions {
                pool_id: "0x3b585786b13af1d8ea067ab37101b6513a05d2f90cfe60e8b1d9e1b46a63c4fa"
                    .to_string(),
                coin_a: "0x2::sui::SUI".to_string(),
                coin_b:
                    "0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC"
                        .to_string(),
                lower_tick_bits: 4294905976,
                upper_tick_bits: 4294907976,
            },
        )
        .await
        .unwrap();
}
