use serde::Deserialize; // serde Deserialize 를 사용해서 JSON 응답을 Rust 구조체로 쉽게 변환할 수 있도록 준비.
use reqwest::Client; // reqwest::Client를 직접 만들어서 timeout을 5초로 설정 => 네트워크 응답이 5초 이상 지연될 경우 자동으로 에러 반환
use std::time::Duration;
use std::fs::OpenOptions; // 파일 처리를 위해 추가
use std::io::Write; // 파일에 쓰기 위해 추가
use chrono::{DateTime, Local};
use tokio::time;

// API 응답 JSON 구조에 맞춰 Rust 구조체 정의
#[derive(Deserialize, Debug)]
struct PrinceInfo {
    usd: f64,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    bitcoin: PrinceInfo,
}

// 핵심 로직 함수 수정 : 가격 정보를 반환하도록 변경
// API를 호출하여 가격 정보를 Result 타입으로 반환하는 함수
async fn fetch_price(client: &Client) -> Result<f64,Box<dyn std::error::Error>> {
    let api_url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";

    let response = client.get(api_url).send().await?;

    if !response.status().is_success() {
        let error_message = format!("API SERVER ERROR RESPONSE (STATUS CODE : {})", response.status());
         return Err(error_message.into());
    }

    let api_data = response.json::<ApiResponse>().await?;
    Ok(api_data.bitcoin.usd)
}

// 파일 저장 함수 추가
// 가져온 가격을 CSV 파일에 기록하는 함수
fn log_price_to_csv(timestamp: &DateTime<Local>, price: f64) -> Result<(), std::io::Error> {
    let file_path = "price_log_csv";

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
    writeln!(file, "{},{}" timestamp.to_rfc3339(), price)?;

    Ok(())
}

// #[tokio::main]으로 비동기 런타임 설정
// main 함수는 잠재적인 모든 에러를 처리하기 위해 Result 타입을 반환.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Result<...> : main 함수의 반환 타입을 Result<(), Box<dyns std::error::Error>>로 설정하고 ? 연산자를 사용해서 코드 어디에서든 에러가 발생하면 해당 에러가 발생하고 프로그램을 안전하게 종료가능.
    // Rust의 견고한 에러처리.

    // 비기능 요구사항 : 5초 타임아웃 설정
    let client = Client::builder()
    .timeout(Duration::from_secs(5))
    .build()?;

    let mut interval = time::interval(Duration::from_secs(60));

    println!("Data collector started (runs every 60 seconds)");
    println!("Collected data is saved in the price_log.csv file.");
    println!("----------------------------------------------------");

    loop {
        interval.tick().await;
        let now = Local::now();

        match fetch_price(&client).await {
            Ok(price) => {
                // 콘솔에 결과 출력
                 println!("{} - BTC/USD: {}",now.format("%Y-%m-%d %H:%M:%S"),price);
            }
            
        }
    }

    
    let api_url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";

    
    let client = Client::builder()
    .timeout(Duration::from_secs(5))
    .build()?;

    println!("DATA 가져오는 중...");

    // 기능 요구사항 : API에 GET 요청 보내기
    let response = client.get(api_url).send().await?;

    // 비기능 요구사항 : API 서버 에러 처리
    if !response.status().is_success() {
        // response.status().is_suecces()를 통해 HTTP 상태 코드가 2xx가 아닐 경우 에러를 발생시킴
        let error_message = format!("API SERVER ERROR RESPONSE (STATUS CODE : {})", response.status());
        return Err(error_message.into());
    }

    // 기능 요구사항 : JSON 파싱 및 데이터 추출
    // 비기능 요구사항 : DATA 파싱 ERROR 처리
    let api_data = response.json::<ApiResponse>().await?;
    // 응답 본문을 임의로 정의한 ApiResponse 구조체로 변환합니다. JSON 형식이 맞지 않으면 이부분에서 에러가 발생하여 프로그램이 안전하게 종료됨.
    let btc_price = api_data.bitcoin.usd;

    // 기능 요구사항 : 현재 시간과 함께 결과 출력
    let now = Local::now();
    println!("----------------------------------------");

    println!("{} - BTC/USD: {}",now.format("%Y-%m-%d %H:%M:%S"),btc_price);
    
    println!("----------------------------------------");

    Ok(())
}