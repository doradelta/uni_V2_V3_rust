<!-- PROJECT LOGO -->
<br />
<div align="center">

  <h3 align="center">Uniswap V2 & V3 event listener</h3> :unicorn:

  <p align="center"> <br />
    Implementation: It listens to the blockchain filtering events (Sync event uniswap V2 and Swap event Uniswap V3). Specifically, there are two data in which symbol (e.g. WETH) and decimals (e.g. 8) need to be queried, this means that there may be a certain overhead of queries, so I have created a HashMap to store this information, and there is no need to ask the blockchain again. The rest of the information can be obtained directly from the events.
    <br /> <br />
    Possible improvements or changes: Filter (apart from events) with contracts (e.g. UniswapV2Contract). When a query is made, if there is an error, it is handled with an expect("..."), that is, stops and panic! the runtime execution (it can be changed with a try!). A more advanced implementation would be to get the optimal price of a pair (e.g. WETH/USDC), through routes with other pairs, or by looking in the mempool.
    <br />
    <br />
    <br />
    <a href="https://docs.uniswap.org/sdk/v2/guides/pricing"><strong>How price is calculated in UniswapV2: docs »</strong></a>
    <br />
    <a href="https://docs.uniswap.org/sdk/v3/guides/fetching-prices"><strong>How price is calculated in UniswapV3: docs »</strong></a>
    <br />
    <a href="https://www.youtube.com/watch?v=hKhdQl126Ys">Youtube: calculate uniswap V2 & V3 price</a>
    <br />
    <br />
    <br />
  <a href="https://github.com/banegil/uni_V2_V3_rust">
    <img src="image.png" alt="Test">
  </a>
  </p>
</div>
