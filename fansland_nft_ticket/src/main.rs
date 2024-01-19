use ethers::{
    contract::abigen,
    core::types::{Address, Filter, H160, H256, U256},
    providers::{Http, Middleware, Provider},
};
use std::sync::Arc;

const HTTP_URL: &str = "https://rpc.flashbots.net";
const V3FACTORY_ADDRESS: &str = "0x1F98431c8aD98523631AE4a59f267346ea31F984";
const DAI_ADDRESS: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
const USDC_ADDRESS: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
const USDT_ADDRESS: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";

// This example demonstrates filtering and parsing event logs by fetching all Uniswap V3 pools
// where both tokens are in the set [USDC, USDT, DAI].
//
// V3 factory reference: https://github.com/Uniswap/v3-core/blob/main/contracts/interfaces/IUniswapV3Factory.sol
// #[tokio::main]
// async fn main() -> eyre::Result<()> {
//     let provider = Provider::<Http>::try_from(HTTP_URL)?;
//     let client = Arc::new(provider);
//     let token_topics = [
//         H256::from(USDC_ADDRESS.parse::<H160>()?),
//         H256::from(USDT_ADDRESS.parse::<H160>()?),
//         H256::from(DAI_ADDRESS.parse::<H160>()?),
//     ];
//     let filter = Filter::new()
//         .address(V3FACTORY_ADDRESS.parse::<Address>()?)
//         .event("PoolCreated(address,address,uint24,int24,address)")
//         .topic1(token_topics.to_vec())
//         .topic2(token_topics.to_vec())
//         .from_block(0);
//     let logs = client.get_logs(&filter).await?;
//     println!("{} pools found!", logs.iter().len());
//     for log in logs.iter() {
//         let token0 = Address::from(log.topics[1]);
//         let token1 = Address::from(log.topics[2]);
//         let fee_tier = U256::from_big_endian(&log.topics[3].as_bytes()[29..32]);
//         let tick_spacing = U256::from_big_endian(&log.data[29..32]);
//         let pool = Address::from(&log.data[44..64].try_into()?);
//         println!(
//             "pool = {pool}, token0 = {token0}, token1 = {token1}, fee = {fee_tier}, spacing = {tick_spacing}"
//         );
//     }
//     Ok(())
// }

// use ethers::prelude::*;
// use ethers::types::Address;
// use std::sync::Arc;

const RPC_URL: &str = "https://polygon-mumbai.g.alchemy.com/v2/NJsreaaTReGW3P0jzESHWw9TIeBKZ3Ly";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from(RPC_URL)?;

    abigen!(SimpleContract, "FanslandNFT.abi",);

    const CONTRACT_ADDRESS: &str = "0x1ae803334c2Bd896ea1d80bb5fF2f3500A239E75";
    let contract_address: Address = CONTRACT_ADDRESS.parse()?;
    let client = Arc::new(provider);
    let contract = SimpleContract::new(contract_address, client);

    // println!(
    //     "合约当前mint的数量: {:?}",
    //     contract.total_supply().call().await?
    // );

    println!(
        "查询地址的余额: {:?}",
        contract
            .balance_of("0x82C3CA8426d4541E48939D9a0F22Fe4359aD1627".parse()?)
            .call()
            .await?
    );

    println!(
        "合约当前mint的数量: {:?}",
        contract
            .tokens_of_owner("0x82C3CA8426d4541E48939D9a0F22Fe4359aD1627".parse()?)
            .call()
            .await?
    );
    // 0xD10295d911E8aD2cDE9126C42b4eC3D974758530
    println!(
        "合约当前mint的数量: {:?}",
        contract
            .tokens_of_owner("0xD10295d911E8aD2cDE9126C42b4eC3D974758530".parse()?)
            .call()
            .await?
    );

    Ok(())
}
