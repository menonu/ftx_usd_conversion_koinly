use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io;

// deposit csv
#[allow(non_snake_case, dead_code)]
#[derive(Debug, Deserialize)]
struct Record {
    id: String,
    time: String,
    coin: String,
    size: String,
    status: String,
    additionalInfo: String,
    txid: String,
    _delete: String,
}

// withdraw csv
#[allow(non_snake_case, dead_code)]
#[derive(Debug, Deserialize)]
struct Withdraw {
    id: String,
    time: String,
    coin: String,
    size: String,
    status: String,
    address: String,
    txid: String,
    fee: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Default)]
struct KoinlyCsv {
    #[serde(rename = "Koinly Date")]
    Date: String,
    Pair: String,
    Side: String,
    Amount: String,
    Total: String,
    #[serde(rename = "Fee Amount")]
    Fee: String,
    #[serde(rename = "Fee Currency")]
    fee_currency: String,
    #[serde(rename = "Order ID")]
    order_id: String,
    #[serde(rename = "Trade ID")]
    trade_id: String,
}

const DOLLARS: [&str; 5] = ["USDC", "TUSD", "USDP", "BUSD", "HUSD"];
fn is_dollar(denom: &str) -> bool {
    DOLLARS.contains(&denom)
}

fn convert_deposit<T: std::io::Write>(writer: &mut csv::Writer<T>, r: Record) -> Result<()> {
    let dollar_record = match &r {
        Record { coin, txid, .. } if is_dollar(coin) && !txid.find("Transfer from").is_some() => {
            Some(r)
        }
        _ => None,
    };

    if let Some(r) = dollar_record {
        let line = KoinlyCsv {
            Date: r.time,
            Pair: r.coin + "/USD",
            Side: "Sell".to_string(),
            Amount: r.size.clone(),
            Total: r.size.clone(),
            Fee: "".to_string(),
            fee_currency: "USD".to_string(),
            order_id: "".to_string(),
            trade_id: r.txid,
        };

        writer.serialize(line)?;
    }
    Ok(())
}

fn convert_withdraw<T: std::io::Write>(writer: &mut csv::Writer<T>, r: Withdraw) -> Result<()> {
    let dollar_record = match &r {
        Withdraw { coin, txid, .. } if is_dollar(coin) && !txid.find("Transfer from").is_some() => {
            Some(r)
        }
        _ => None,
    };

    if let Some(r) = dollar_record {
        let line = KoinlyCsv {
            Date: r.time,
            Pair: r.coin.clone() + "/USD",
            Side: "Buy".to_string(),
            Amount: r.size.clone(),
            Total: r.size.clone(),
            Fee: r.fee,
            fee_currency: r.coin,
            order_id: "".to_string(),
            trade_id: r.txid,
        };

        writer.serialize(line)?;
    }
    Ok(())
}


fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("args must be 1. e.g) ./main deposit.csv");
    }

    let mut writer = csv::Writer::from_writer(io::stdout());

    let f = File::open(&args[1])?;
    let mut rdr = csv::Reader::from_reader(f);
    let mut iter = rdr.deserialize().peekable();
    let is_deposit = iter.peek().ok_or(anyhow!("a"))?.is_ok();

    if is_deposit {
        for r in iter {
            convert_deposit(&mut writer, r?)?;
        }
    } else {
        // rewind
        let iter2 = rdr.deserialize();
        for r in iter2 {
            convert_withdraw(&mut writer, r?)?;
        }
    }


    Ok(())
}
