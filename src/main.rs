use serde::Deserialize; // serde Deserialize 를 사용해서 JSON 응답을 Rust 구조체로 쉽게 변환할 수 있도록 준비.
use reqwest::Client; // reqwest::Client를 직접 만들어서 timeout을 5초로 설정 => 네트워크 응답이 5초 이상 지연될 경우 자동으로 에러 반환
use std::time::Duration;
use std::fs::OpenOptions; // 파일 처리를 위해 추가
use std::io::Write; // 파일에 쓰기 위해 추가
use chrono::{DateTime, Local};
use tokio::time;

// 설정 파일(config.toml) 구조에 맞는 구조체 정의
#[derive(Deserialize, Debug)]
struct Settings {
    crypto_id: String,
    vs_currency: String,
    interval_seconds: u64,
    log_file: String,
}

// API 응답 JSON 구조에 맞춰 Rust 구조체 정의
#[derive(Deserialize, Debug)]
struct PrinceData {
    eur: Option<f64>, // 필드가 동적으로 변할 수 있으므로 Option으로 처리
    usd: Option<f64>,
    jpy: Option<f64>,
}

// 동적인 키 ("bitcoin", "ehternum")를 처리하기 위해 HashMap 사용
use std::collections::HashMap;
type ApiResponse<PriceData> = HashMap<String, PriceData>;


// 핵심 로직 함수 수정 : 가격 정보를 반환하도록 변경
// 핵심 로직 함수 수정 : 설정 값을 인자로 받도록 수정
// API를 호출하여 가격 정보를 Result 타입으로 반환하는 함수
async fn fetch_price(client: &Client, crypto_id: &str, vs_currency: &str) -> Result<f64,Box<dyn std::error::Error>> {
    // 설정 값으로 API URL을 동적으로 생성
    let api_url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies={}",
        crypto_id, vs_currency
    );
    // let api_url: &'static str = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";

    let response: reqwest::Response = client.get(api_url).send().await?;

    if !response.status().is_success() {
        return Err(format!("API SERVER ERROR (STATUS CODE: {})", response.status()).into());
    }

    let mut api_data = response.json::<ApiResponse<PrinceData>>().await?;

    // 응답에서 가격 데이터 추출
    let price_info: PrinceData = api_data.remove(crypto_id)
    .ok_or_else(|| format!("Response '{}' None Data", crypto_id))?;

    let price = match vs_currency {
        "usd" => price_info.usd,
        "eur" => price_info.eur,
        "jpy" => price_info.jpy, _ => None,
    }.ok_or_else(|| format!("Response '{}'None Currnecy Data", vs_currency))?;

    Ok(price)
}

// 파일 저장 함수 추가
// 가져온 가격을 CSV 파일에 기록하는 함수
// 파일 경로를 인자로 받도록 수정

fn log_price_to_csv(file_path: &str, timestamp: &DateTime<Local>, price: f64) -> Result<(), std::io::Error> {

    // 파일을 추가 모드(append)로 열고, 파일이 없으면 새로 생성(create) 합니다.
    let mut file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(file_path)?;

    // 파일이 비어있는지 (새로 생성되었는지) 확인.
    if file.metadata()?.len() == 0 {
        // 파일이 비어있으면 헤더를 추가
        writeln!(file, "timestamp,price")?;
    }

    // "타임스탬프, 가격" 형식으로 새로운 데이터를 파일에 씀
    writeln!(file, "{},{}", timestamp.to_rfc3339(), price)?;


    Ok(())
}

 //   let file_path: &'static str = "price_log_csv";

  

// #[tokio::main]으로 비동기 런타임 설정
// main 함수는 잠재적인 모든 에러를 처리하기 위해 Result 타입을 반환.
// Result<...> : main 함수의 반환 타입을 Result<(), Box<dyns std::error::Error>>로 설정하고 ? 연산자를 사용해서 코드 어디에서든 에러가 발생하면 해당 에러가 발생하고 프로그램을 안전하게 종료가능.
// Rust의 견고한 에러처리.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let settings= config::Config::builder()
    .add_source(config::File::with_name("config")) // config.toml 파일을 읽음.
    .build()?
    .try_deserialize::<Settings>()?; // 읽은 값을 Settings 구조체로 변환 
    
    println!("Settings File Load Success : {:?}", settings);

    // 비기능 요구사항 : 5초 타임아웃 설정
    let client = Client::builder()
    .timeout(Duration::from_secs(5))
    .build()?;

    let mut interval = time::interval(Duration::from_secs(settings.interval_seconds));

    println!("Data collector started (runs every {} seconds)", settings.interval_seconds);
    println!("Collected data is saved in the price_log.csv file : {}/{}", settings.crypto_id.to_uppercase(), settings.vs_currency.to_uppercase());
    println!("Saved File : {}", settings.log_file);
    println!("----------------------------------------------------");

    loop {
        interval.tick().await;
        let now: DateTime<Local> = Local::now();

        // 설정 값을 fetch_price 함수에 전달
        match fetch_price(&client, &settings.crypto_id, &settings.vs_currency).await {
            Ok(price) => {
                 println!(
                    "[시간: {}] {}/{}: {}",
                 now.format("%Y-%m-%d %H:%M:%S"),
                 settings.crypto_id.to_uppercase(),
                 settings.vs_currency.to_uppercase(),
                 price
                );
                 // 설정 파일에서 읽은 파일 경로를 전달.
                 if let Err(e) =  log_price_to_csv(&settings.log_file, &now, price) {
                     eprintln!("File Saving ERROR : {}", e);
                 }
            }
            Err(e) => {
                eprintln!("Data Collecting ERROR : {}", e);
            }
        }
    }
}