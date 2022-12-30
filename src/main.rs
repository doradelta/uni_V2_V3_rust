
extern crate web3;
extern crate ethabi;

use std::collections::HashMap;
use chrono::Local;
use futures::TryStreamExt;
use hex_literal::hex;
use ethereum_types::U256;
use std::ops::Div;
use bignumber::{BigNumber};
use web3::{
    contract::{Options, Contract},
    types::{FilterBuilder,Log, Address},
};

#[derive(Clone, Default)] 
// Adress_pool_info
struct AddressInfo {
    token0_symbol: String,
    token1_symbol: String,
    token0_decimals: u8,
    token1_decimals: u8
}


pub const WEBSOCKET_URL: &str = "wss://localhost:8545";

/**
    Prints uniswap pool type (V2) and price token1/token0 (e.g. WETH/USDC: 0.00065) 
    UniswapV2 source how price is calculated: https://docs.uniswap.org/sdk/v2/guides/pricing
**/
fn print_uniswap_v2_price(log: &Log, token0_decimals: u8, token1_decimals: u8) {
    println!("Pool type: UniswapV2");

    // Data : [ reserve0 (uint112), reserve1 (uint112) ]
    let data = &log.data.0;
    let (data0, data1) = data.split_at(data.len() / 2);

    let reserve0 = U256::from_big_endian(data0);
    let reserve1 = U256::from_big_endian(data1);

    // Decimals of token0 and token1 (e.g. WETH: 18 decimals and USDC 6 decimals)
    let exp_token0 = BigNumber::from(10).powi(token0_decimals as i32);
    let exp_token1 = BigNumber::from(10).powi(token1_decimals as i32);

    // reserve1 / reserve0
    let price = BigNumber::from(reserve1).div(exp_token1) / BigNumber::from(reserve0).div(exp_token0);
    
    println!("Price: {}", price.to_precision(5));
}

/*
    Prints uniswap pool type (V3) and price token1/token0 (e.g. WETH/USDC: 0.00065) 
    UniswapV3 source how price is calculated: https://docs.uniswap.org/sdk/v3/guides/fetching-prices
*/
fn print_uniswap_v3_price(log: &Log, token0_decimals: u8, token1_decimals: u8) {
    println!("Pool type: UniswapV3");

    // Data : [ _, _, sqrtPriceX96 (uint160), _, _]
    // we are just interested in third element of Data vec<u8>
    let data = &log.data.0;
    let chunks: Vec<&[u8]> = data.chunks(data.len() / 5).collect();

    // price = (sqrtPriceX96 ** 2) / ((2 ** 96) ** 2)
    let sqrt_ratio_x96 = U256::from_big_endian(chunks[2]);
    let q96 = BigNumber::from(2).powi(96);
    let p = BigNumber::from(sqrt_ratio_x96);

    // not important, just improves performance calculation
    let exp = token1_decimals as i32 - token0_decimals as i32;

    let price = BigNumber::from(1) / (BigNumber::from(10).powi(exp) / (p.div(q96)).powi(2));
    
    println!("Price: {}", price.to_precision(5));
}

#[tokio::main]
async fn main() -> web3::contract::Result<()> {
    // Insert you WEBSOCKET_URL here
    let web3 = web3::Web3::new(web3::transports::WebSocket::new(WEBSOCKET_URL).await?);

    // Set up the filter to listen for Sync(V2) & Swap(V3) events emitted by any contract that support them
    let filter = FilterBuilder::default()
        .topics(
            Some(vec![hex!("1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1").into(),   // Sync event Uniswap V2
                      hex!("c42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67").into()]), // Swap event Uniswap V3
            None,
            None,
            None,
        ).build();

    // Subscribe
    let sub = web3.eth_subscribe().subscribe_logs(filter).await?;
    let mut map : HashMap<Address, AddressInfo> = HashMap::new();
    
    // Iterate over rreceived events
    let _ = sub.try_fold(&mut map, |map, log_result| async {
        let log = log_result;
        let mut pool_address_info : AddressInfo = Default::default();

        /** 
            If the pool has already been seen by this program at runtime, 
            we don't ask again for static data (such as the decimals or symbol of a token),
            because we have stored this information in a map
        **/
        
        if map.contains_key(&log.address) {
            // Pool_address already stored
            pool_address_info = map[&log.address].clone();
        }
        else{ // New pool_address, we need to query data (symbols and decimals)
            let uniswap_pair_contract =
                Contract::from_json(web3.eth(), log.address, include_bytes!("../abi/UniswapPair_abi.json")).unwrap();
            
            // Query token0 address
            let token0 : Address = uniswap_pair_contract
                .query("token0", (), None, Options::default(), None)
                .await
                .expect("Error reading token address from UniswapPair_contract contract");
        
            // Query token1 address
            let token1 : Address = uniswap_pair_contract
                .query("token1", (), None, Options::default(), None)
                .await
                .expect("Error reading token address from UniswapPair_contract contract");
        
            // Get tokens contract objects
            let token0_contract = 
                Contract::from_json(web3.eth(), token0, include_bytes!("../abi/ERC20_abi.json")).unwrap();
            let token1_contract = 
                Contract::from_json(web3.eth(), token1, include_bytes!("../abi/ERC20_abi.json")).unwrap();

            // Query token0 decimals
            pool_address_info.token0_decimals = token0_contract
                .query("decimals", (), None, Options::default(), None)
                .await
                .expect("Error reading decimals from ERC20 contract");

            // Query token1 decimals
            pool_address_info.token1_decimals = token1_contract
                .query("decimals", (), None, Options::default(), None)
                .await
                .expect("Error reading decimals from ERC20 contract");

            // Query token0 symbol (e.g. WETH)
            pool_address_info.token0_symbol = token0_contract
                .query("symbol", (), None, Options::default(), None)
                .await
                .expect("Error reading symbol from ERC20 contract");
        
            // Query token1 symbol (e.g. USDC)
            pool_address_info.token1_symbol = token1_contract
                .query("symbol", (), None, Options::default(), None)
                .await
                .expect("Error reading symbol from ERC20 contract");

            // Store the pool_addres and its info, to avoid overhead of querys
            map.insert(log.address, pool_address_info.clone());
        }

        // Print info
        println!("Timestamp: {}", Local::now());
        println!("Block number: {}", log.block_number.unwrap());
        println!("Transaction hash: 0x{:x}", log.transaction_hash.unwrap());
        println!("Pool contract: 0x{:x}", log.address);
        println!("Token's pair: {}/{}", pool_address_info.token0_symbol, pool_address_info.token1_symbol);

        // We know Sync_topics.len() == 1 and Swap_topics.len() == 3
        match log.topics.len() {
            1 => print_uniswap_v2_price(&log, pool_address_info.token0_decimals, pool_address_info.token1_decimals),
            3 => print_uniswap_v3_price(&log, pool_address_info.token0_decimals, pool_address_info.token1_decimals),
            _ => println!("Error, unknow log.topic"),
        }

        println!("");
        Ok(map)
    })
    .await;
    
    Ok(())
}
