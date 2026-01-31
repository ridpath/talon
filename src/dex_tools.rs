use reqwest::blocking::Client;
use serde_json::Value;

/// Calls Uniswap or Curve API and extracts price.
/// Supported sources: "uniswap", "curve"
pub fn dex_price(source: &str, token_a: &str, token_b: &str) -> f64 {
    match source {
        "uniswap" => get_uniswap_price(token_a, token_b),
        "curve" => get_curve_price(token_a, token_b),
        _ => 0.0,
    }
}

fn get_uniswap_price(token_a: &str, token_b: &str) -> f64 {
    let query = format!(
        r#"
        {{
            pair(id: "{}-{}") {{
                token0Price
                token1Price
            }}
        }}
    "#,
        token_a, token_b
    );

    let client = Client::new();
    let res = client
        .post("https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v2")
        .json(&serde_json::json!({ "query": query }))
        .send()
        .expect("Uniswap API failed");

    let json: Value = res.json().expect("JSON error");
    let price = json["data"]["pair"]["token0Price"]
        .as_str()
        .unwrap_or("0.0")
        .parse::<f64>()
        .unwrap_or(0.0);

    price
}

fn get_curve_price(_token_a: &str, _token_b: &str) -> f64 {
    // This can be replaced with an actual eth_call to Curve pools via web3
    // For now, mock price
    0.9912
}
