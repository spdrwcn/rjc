use clap::Parser;
use csv::WriterBuilder;
use rayon::prelude::*;
use redis::{Client, Commands};
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use serde_json::to_string_pretty;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = String::from("redis://127.0.0.1:6379/0"), value_parser = redis_ip_address_parser)]
    ip_address: String,
    #[arg(short, long, default_value_t = String::from("mac.csv"))]
    path: String,
}

fn redis_ip_address_parser(s: &str) -> Result<String, String> {
    let ip_pattern = Regex::new(r"^redis://((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?):([1-9]\d{0,4}|[1-5]\d{4}|6[0-4]\d{3}|65[0-4]\d{2}|655[0-2]\d|6553[0-5])/([0-9]|1[0-5])$").unwrap();

    if !ip_pattern.is_match(s) {
        return Err("错误: 请输入正确的Redis地址格式, 例如 'redis://127.0.0.1:6379/0'".to_string());
    }
    Ok(s.to_string())
}

#[derive(Serialize, Deserialize, Debug)]
struct DeviceInfo {
    sn: String,
    bluetooth_mac: String,
    wired_mac: String,
    wireless_mac: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeviceValue {
    bluetooth_mac: String,
    wired_mac: String,
    wireless_mac: String,
}

fn get_client(ip_address: &str) -> Result<Client, Box<dyn std::error::Error>> {
    let client = Client::open(ip_address)?;
    Ok(client)
}

fn get_keys(con: &mut redis::Connection) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let keys: Vec<String> = con.keys("*")?;
    Ok(keys)
}

fn select_output_function(
    path: &str,
    key_value_pairs: HashMap<String, DeviceInfo>,
) -> Result<(), Box<dyn std::error::Error>> {
    match path.rsplit_once('.').unwrap_or(("", "")).1 {
        "csv" => {
            let records: Vec<Vec<String>> = key_value_pairs
                .par_iter()
                .map(|(sn, device_info)| {
                    vec![
                        sn.clone(),
                        device_info.wired_mac.clone(),
                        device_info.wireless_mac.clone(),
                        device_info.bluetooth_mac.clone(),
                    ]
                })
                .collect();
            let headers = vec!["SN", "wired_mac", "wireless_mac", "bluetooth_mac"];
            Ok(write_csv_file(path, headers, records)?)
        }
        "json" => {
            let json_string = to_string_pretty(&key_value_pairs.values().collect::<Vec<_>>())?;
            write_json_file(&json_string, path)
        }
        _ => Err("不支持的文件格式".into()),
    }
}

fn get_device_infos(
    con: &mut redis::Connection,
    keys: &[String],
    as_json: bool,
) -> Result<HashMap<String, DeviceInfo>, Box<dyn std::error::Error>> {
    let mut key_value_pairs: HashMap<String, DeviceInfo> = HashMap::new();
    let mut pipe = redis::pipe();

    for key in keys {
        pipe.get(key);
    }
    let values: Vec<String> = pipe.query(con)?;

    let results: Vec<(String, DeviceInfo)> = keys
        .par_iter()
        .zip(values.par_iter())
        .filter_map(
            |(key, value)| match serde_json::from_str::<DeviceValue>(value) {
                Ok(device_value) => {
                    let device_info = DeviceInfo {
                        sn: key.clone(),
                        bluetooth_mac: device_value.bluetooth_mac,
                        wired_mac: device_value.wired_mac,
                        wireless_mac: device_value.wireless_mac,
                    };
                    Some((key.to_owned(), device_info))
                }
                Err(_) => match as_json {
                    true => {
                        eprintln!("{}: 值类型不匹配，已忽略", key);
                        None
                    }
                    false => match serde_json::from_str::<DeviceInfo>(value) {
                        Ok(device_info) => Some((key.to_owned(), device_info)),
                        Err(_) => {
                            eprintln!("{}: 值类型不匹配，已忽略", key);
                            None
                        }
                    },
                },
            },
        )
        .collect();

    for (key, value) in results {
        key_value_pairs.insert(key, value);
    }

    Ok(key_value_pairs)
}

fn write_csv_file(
    path: &str,
    headers: Vec<&str>,
    records: Vec<Vec<String>>,
) -> Result<(), csv::Error> {
    let file = File::create(path)?;
    let mut writer = WriterBuilder::new().from_writer(file);

    writer.write_record(headers)?;

    for record in records {
        writer.write_record(record)?;
        writer.flush()?;
    }

    Ok(())
}

fn write_json_file(json_string: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    file.write_all(json_string.as_bytes())?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    let cli = Cli::parse();

    let client = get_client(&cli.ip_address)?;
    let mut con = client.get_connection()?;
    let keys = get_keys(&mut con)?;

    let is_json = cli.path.ends_with(".json");
    let key_value_pairs = get_device_infos(&mut con, &keys, is_json)?;

    select_output_function(&cli.path, key_value_pairs)?;

    let elapsed_time = start_time.elapsed();
    println!("获取数据: {}组", keys.len());
    println!("用时: {:?}", elapsed_time);
    Ok(())
}
