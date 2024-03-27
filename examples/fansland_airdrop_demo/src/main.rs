use ethers::{
    contract::abigen,
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, U256},
};
use eyre::Result;
use std::convert::TryFrom;
use std::sync::Arc;
use tracing::{instrument::WithSubscriber, Level};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // let chainid = 56;
    // let rpc_url = "https://bsc-testnet.nodereal.io/v1/83d3d7d77ee04a2e948a5fdb0f9dd98e";

    let provider = Provider::<Http>::try_from(
        "https://bsc-testnet.nodereal.io/v1/83d3d7d77ee04a2e948a5fdb0f9dd98e",
    )?;

    let chain_id = provider.get_chainid().await?;

    let contract_address = "0xd43C9799288311356aF329f3cBeB4Aa015cC5cEd".parse::<Address>()?;

    let from_wallet: LocalWallet =
        "0xa1102aa1ecf406a2633bd227efc4ecd16aa5c642d3b85a606b7b20fad109a50d"
            .parse::<LocalWallet>()?
            .with_chain_id(chain_id.as_u64());

    let signer = Arc::new(SignerMiddleware::new(&provider, from_wallet));

    println!("==================");
    println!("from_wallet: {}", signer.address());
    abigen!(FanslandNFTContract, "FanslandNFT.abi",);

    // let fansland_nft_contract_address = contract_address;
    // let contract_address_h160: Address = contract_address.parse().unwrap();
    let contract = FanslandNFTContract::new(contract_address, signer);

    let type_id: U256 = 0.into();
    let token_id: U256 = 60000.into();
    let recipient: Address = String::from("0x274848a43f6afdDEed6623FB45c8B3e369936B5E")
        .parse()
        .unwrap();

    let tx = contract.redeem_airdrop(type_id, token_id, recipient);
    tracing::info!("pending_tx: {:?}", tx);

    let pending_tx = tx.send().await.unwrap();
    tracing::info!("pending_tx: {:?}", pending_tx);

    let mined_tx = pending_tx.await.unwrap();
    tracing::info!("pending_tx: {:?}", mined_tx);

    Ok(())
}
