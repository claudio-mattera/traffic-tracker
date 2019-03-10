mod error;
use crate::error::*;

mod configuration;
use crate::configuration::*;

use std::fmt;

use log::*;

fn main() -> Result<(), TrafficError> {
    let configuration = load_configuration()?;

    let base_url = reqwest::Url::parse(&configuration.base_url)?;
    let username = configuration.username;
    let password = configuration.password;

    let client = reqwest::Client::new();

    let session_id = login(&base_url, &client, &username, &password)?;
    debug!("Session ID: {}", session_id);
    let total_traffic = get_overview(&base_url, &client, session_id)?;
    logout(&base_url, &client, session_id)?;

    info!("Total traffic: {}", total_traffic);

    Ok(())
}

fn login(
    base_url: &reqwest::Url,
    client: &reqwest::Client,
    username: &str,
    password: &str,
) -> Result<u64, TrafficError> {

    debug!("Logging in...");
    let params = [("Username", username), ("Password", password)];
    let url = base_url.join("/index/login.cgi")?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::COOKIE, "Language=en_us.".parse().unwrap());

    let request = client.post(url)
        .form(&params)
        .headers(headers)
        .build()?;

    let response = process_request(&client, request)?;

    if let Some(cookie) = response.headers().get(reqwest::header::SET_COOKIE) {
        let mut cookie = cookie.to_str()?.to_string();
        let index = cookie.find(';').unwrap();
        cookie.truncate(index);
        if cookie.find("SessionID_R3=").is_some() {
            let session_id = cookie.split_off("SessionID_R3=".len());
            let session_id: u64 = session_id.parse()?;
            return Ok(session_id);
        }
        return Err(TrafficError::new(
            "Did not receive a new session id".to_string()
        ));
    }

    Err(TrafficError::new(
        "Did not receive a new cookie".to_string()
    ))
}

fn logout(
    base_url: &reqwest::Url,
    client: &reqwest::Client,
    session_id: u64,
) -> Result<(), TrafficError> {

    debug!("Logging out...");

    let cookie = format!("Language=en_us; SessionID_R3={}", session_id);
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::COOKIE, cookie.parse().unwrap());

    let url = base_url.join("/index/logout.cgi")?;
    let request = client.post(url)
        .headers(headers)
        .build()?;
    let _response = process_request(&client, request)?;

    Ok(())
}

fn get_overview(
    base_url: &reqwest::Url,
    client: &reqwest::Client,
    session_id: u64,
) -> Result<Bytes, TrafficError> {

    debug!("Getting overview...");

    let cookie = format!("Language=en_us; SessionID_R3={}", session_id);
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::COOKIE, cookie.parse().unwrap());
    headers.insert(
        reqwest::header::REFERER,
        "http://192.168.1.1/index/login.cgi".parse().unwrap()
    );

    let url = base_url.join("/html/status/overview.asp")?;
    let request = client.get(url)
        .headers(headers)
        .build()?;

    let mut response = process_request(&client, request)?;

    let mut text = response.text()?;

    if let Some(index) = text.find("WanStatistics = {") {
        let mut text = text.split_off(index + "WanStatistics = ".len());
        if let Some(index) = text.find('}') {
            text.truncate(index + 1);
            // { uprate' : '0' , 'downrate' : '0' , 'upvolume' : '0' , 'downvolume' : '0' , 'liveTime' : '0' }
            let text = text.replace("'", "\"");
            let dict: serde_json::Value = serde_json::from_str(&text)?;
            let upvolume: u64 = dict.get("upvolume").unwrap()
                .as_str().unwrap()
                .parse().unwrap();
            let downvolume: u64 = dict.get("downvolume").unwrap()
                .as_str().unwrap()
                .parse().unwrap();
            let livetime: u64 = dict.get("liveTime").unwrap()
                .as_str().unwrap()
                .parse().unwrap();
            let livetime = Duration::from_secs(livetime);
            let total_traffic = upvolume + downvolume;
            debug!("Total traffic: {}", total_traffic);
            debug!("Livetime: {}", livetime);
            return Ok(Bytes::new(total_traffic));
        } else {
            Err(TrafficError::new("No closing brace".to_string()))
        }
    } else {
        Err(TrafficError::new("No WanStatistics structure".to_string()))
    }
}

fn process_request(
    client: &reqwest::Client,
    request: reqwest::Request,
) -> Result<reqwest::Response, TrafficError> {

    let url = request.url().clone();
    debug!("T {} -> {}", "this", url);
    debug!("{} {} HTTP/1.1.", request.method(), url);
    for (key, value) in request.headers().iter() {
        debug!("{:?}: {:?}.", key, value);
    }
    if let Some(body) = request.body() {
        debug!("");
        debug!("");
        debug!("{:#?}", body);
    }
    debug!("");
    debug!("");

    let response = client.execute(request)?;
    debug!("T {} -> {}", url, "this");
    debug!("HTTP/1.1 {} {}.", response.status().as_u16(), response.status().as_str());
    for (key, value) in response.headers().iter() {
        debug!("{:?}: {:?}.", key, value);
    }

    Ok(response)
}

struct Bytes(u64);

impl Bytes {
    fn new(n: u64) -> Self {
        Self(n)
    }
}

impl fmt::Display for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 < 1024 {
            write!(f, "{} B", self.0)
        } else if self.0 < 1024 * 1024 {
            write!(f, "{:.2} kB", self.0 as f64 / 1024.0)
        } else if self.0 < 1024 * 1024 * 1024 {
            write!(f, "{:.2} MB", self.0 as f64 / 1024.0 / 1024.0)
        } else {
            write!(f, "{:.2} GB", self.0 as f64 / 1024.0 / 1024.0 / 1024.0)
        }
    }
}


struct Duration(u64);

impl Duration {
    fn from_secs(n: u64) -> Self {
        Self(n)
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "P")?;

        let mut n: u64 = self.0;

        if n >= 3600 * 24 {
            write!(f, "D{}", n / (3600 * 24))?;
            n %= 3600 * 24;
        }

        if n > 0 {
            write!(f, "T")?;
        }

        if n >= 3600 {
            write!(f, "H{:02}", n / 3600)?;
        }
        n %= 3600;
        if n >= 60 {
            write!(f, "M{:02}", n / 60)?;
        }
        n %= 60;
        if n > 0 {
            write!(f, "S{:02}", n)?;
        }

        Ok(())
    }
}
