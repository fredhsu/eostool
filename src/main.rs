// #![deny(warnings)]
// use std::collections::HashMap;
use std::env;
extern crate reqwest;
// use reqwest::Error;
extern crate serde;
use serde::{Deserialize, Serialize};
extern crate serde_json;
// use serde_json::json;

#[macro_use]
extern crate clap;
use clap::App;

#[derive(Debug, Serialize, Deserialize)]
struct JsonRPCResponse {
    jsonrpc: String,
    result: Vec<serde_json::Value>,
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRPCRequest {
    jsonrpc: String,
    method: String,
    params: JsonParams,
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonParams {
    format: String,
    timestamps: bool,
    #[serde(rename = "autoComplete")]
    auto_complete: bool,
    #[serde(rename = "expandAliases")]
    expand_aliases: bool,
    cmds: Vec<String>,
    version: i32,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();
    let command = matches.value_of("command").unwrap();
    let device = matches.value_of("DEVICE").unwrap();
    //let username = matches.value_of("username");
    let username;
    //let password = matches.value_of("password").unwrap_or("admin");
    let password;
    let format = matches.value_of("output").unwrap_or("json");

    let mut commands = vec![command.to_string()];
    match matches.value_of("username") {
        None => username = env::var("EAPI_USERNAME").unwrap(),
        Some(match_username) => username = match_username.to_string(),
    }
    match matches.value_of("password") {
        None => password = env::var("EAPI_PASSWORD").unwrap(),
        Some(match_password) => password = match_password.to_string(),
    }

    if matches.is_present("enable") {
        commands.insert(0, "enable".to_string());
    }
    let url;

    let params = JsonParams {
        format: format.to_string(),
        timestamps: matches.is_present("timestamps"),
        auto_complete: true,
        expand_aliases: true,
        cmds: commands,
        version: 1,
    };
    let jsonrpc = JsonRPCRequest {
        jsonrpc: "2.0".to_string(),
        method: "runCmds".to_string(),
        params,
        id: "eostool".to_string(),
    };

    let body = serde_json::json!(jsonrpc);
    let client;

    if matches.is_present("ssl") {
        url = format!("https://{}/command-api", device);
        client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;
    } else {
        url = format!("http://{}/command-api", device);
        client = reqwest::Client::new();
    }
    let res = client
        .post(url.as_str())
        .json(&body)
        .basic_auth(username, Some(password))
        .send()
        .await?
        .json::<JsonRPCResponse>()
        .await?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&res.result).unwrap());
    } else {
        //let text_output: String = serde_json::from_value(res.result[1]["output"]).unwrap();
        let result = &res.result[1];
        //let text_output: String = serde_json::from_value(result).unwrap();
        //println!("{}", text_output);
        println!(
            "{}",
            serde_json::to_string_pretty(&result["output"]).unwrap()
        );
        //println!("{:?}", serde_json::to_string(&result).unwrap());
    }

    Ok(())
}
