use serde::Deserialize; // serde Deserialize 를 사용해서 JSON 응답을 Rust 구조체로 쉽게 변환할 수 있도록 준비.
use reqwest::Client;
use std::time::Duration;
use chrono::Local;

// API 응답 JSON 구조에 맞춰 Rust 구조체 정의
#[derive(Deserialize, Debug)]
struct PrinceInfo {
    usd: f64,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    bitcoin: PrinceInfo,
}

// #[tokio::main]으로 비동기 런타임 설정
// main 함수는 잠재적인 모든 에러를 처리하기 위해 Result 타입을 반환.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";

    // 비기능 요구사항 : 5초 타임아웃 설정
    let client = Client::builder()
    .timeout(Duration::from_secs(5))
    .build()?;

    println!("DATA 가져오는 중...");

    // 기능 요구사항 : API에 GET 요청 보내기
    let response = client.get(api_url).send().await?;

    // 비기능 요구사항 : API 서버 에러 처리
    if !response.status().is_success() {
        // HTTP 상태 코드가 2xx가 아닐 경우 에러를 발생시킴
        let error_message = format!("API SERVER ERROR RESPONSE (STATUS CODE : {})", response.status());
        return Err(error_message.into());
    }

    // 기능 요구사항 : JSON 파싱 및 데이터 추출
    // 비기능 요구사항 : DATA 파싱 ERROR 처리
    let api_data = response.json::<ApiResponse>().await?;
    let btc_price = api_data.bitcoin.usd;

    // 기능 요구사항 : 현재 시간과 함께 결과 출력
    let now = Local::now();
    println!("----------------------------------------");
     println!(
        "{} - BTC/USD: {}",
        now.format("%Y-%m-%d %H:%M:%S"),
        btc_price
    );
    println!("----------------------------------------");

    Ok(())
}