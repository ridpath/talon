# ⛓️ Web3 + Ethereum scan example
# - Decodes transaction input
# - Sends eth_call to check EVM behavior
# Useful for contract auditing, MEV detection, or tx tracing

include "blockchain/rpc.my"
include "blockchain/abi.my"
include "blockchain/txdecode.my"

define function scan()
    # Extracts function signature and resolves it
    decode_tx_input("a9059cbb000000000000000000000000deadbeef00000000000000000000000000000064")

    # Simulate an eth_call to Infura or local node
    eth_call("https://mainnet.infura.io/v3/YOURKEY", "0xa9059cbb...")
end

# 🚀 Run Web3 scan + decode
scan()
