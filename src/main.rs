use serde::Deserialize; // serde Deserialize 를 사용해서 JSON 응답을 Rust 구조체로 쉽게 변환할 수 있도록 준비.
use reqwest::Client; // reqwest::Client를 직접 만들어서 timeout을 5초로 설정 => 네트워크 응답이 5초 이상 지연될 경우 자동으로 에러 반환
use std::time::Duration;
use std::io::Write; // 파일에 쓰기 위해 추가
use chrono::{DateTime, Local};
use tokio::{time, signal}; // signal 모듈 추가
use std::fs::{File, OpenOptions}; // 파일 처리를 위해 추가
use std::collections::HashMap; // 동적인 키 ("bitcoin", "ehternum")를 처리하기 위해 HashMap 사용

// 설정 파일(config.toml) 구조에 맞는 구조체 정의
#[derive(Deserialize, Debug)]
struct Settings {
    crypto_ids: Vec<String>, // 여러 ID를 받기 위해 Vce<String>으로 변경
    vs_currency: String,
    interval_seconds: u64,
    log_file_prefix: String
}

// API 응답 JSON 구조에 맞춰 Rust 구조체 정의
#[derive(Deserialize, Debug)]
struct PriceData {
    // 필드가 동적으로 변할 수 있으므로 Option으로 처리
    eur: Option<f64>, 
    usd: Option<f64>,
    jpy: Option<f64>
}

type ApiResponse = HashMap<String, PriceData>;

// Quick Sort 알고리즘

fn quick_sort(arr: &mut [f64]) {
    if arr.len() > 1 {
        let pivot_index = partition(arr);
        
        let (left, right) = arr.split_at_mut(pivot_index);
        quick_sort(left);
        // 피벗 자체는 이미 정렬된 위치에 있으므로, 그 다음부터 정렬.
        if !right.is_empty() {
            quick_sort(&mut right[1..]);
        }
    }
}

// 배열의 중간에 있는 값을 피벗으로 선택
// 수정 : 세 값의 중앙값 방식
fn partition(arr: &mut [f64]) -> usize {
    let len = arr.len();
    let mid = len / 2;
    let last = len - 1;

    // 배열의 첫 값, 중간 값, 마지막 값 세 후보를 정렬.
    if arr[0] > arr[mid] {
        arr.swap(0, mid);
    }
    if arr[0] > arr[last] {
        arr.swap(0, last);
    }
    if arr[mid] > arr[last] {
        arr.swap(mid, last);
    }
    // 이 시점에서  arr[mid]는 세 값 중 중앙값.
    

    // 찾은 중앙값(피벗)을 배열의 끝으로 보냄
    arr.swap(mid, last);

    // 피벗을 기준으로 분할 작업을 수행 (기존 로직과 동일)
    let mut i = 0;
    for j in 0..arr.len() - 1 {
        if arr[j] <= arr[arr.len() - 1] {
            arr.swap(i, j);
            i += 1;
        }
    }
    arr.swap(i, arr.len() - 1);
    i
}

// 데이터 분석
fn analyze_and_print_stats(prefix: &str, crypto_ids: &[String]) {
    println!("Start analyzing existing data...");
    for id in crypto_ids {
        let file_path: String = format!("{}_{}.csv", prefix, id);
        println!("\n -----[{}]Data analyzing--------", id.to_uppercase());

        let file = match File::open(&file_path) {
            Ok(f) => f,
            Err(_) => {
                println!("     -There is no Data file to analyze.('{}')", file_path);
                continue;
            }
        };

        let mut rdr = csv::Reader::from_reader(file);
        let mut prices: Vec<f64> = rdr.records()
            .filter_map(|result| result.ok())
            .filter_map(|record| record.get(1).and_then(|price_str| price_str.parse::<f64>().ok()))
            .collect();

        if prices.is_empty() {
            println!("       -  There is no data to analyze in the file.");
            continue;
        }
        
        let count = prices.len();
        println!("        - A total of past data were loaded.('{}')", count);

        quick_sort(&mut prices);

        let min_price = prices[0];
        let max_price = prices[count - 1];
        let median_price = if count % 2 == 0 {
            (prices[count / 2 - 1] + prices[count / 2]) / 2.0
        } else {
            prices[count / 2]
        };

        println!("        min : ${:.2}", min_price);
        println!("        max : ${:.2}", max_price);
        println!("        median : ${:.2}", median_price);
    }
}


// 핵심 로직 함수 수정 : 가격 정보를 반환하도록 변경
// 핵심 로직 함수 수정 : 설정 값을 인자로 받도록 수정
// API를 호출하여 가격 정보를 Result 타입으로 반환하는 함수
async fn fetch_prices(client: &Client, crypto_ids: &[String], vs_currency: &str) -> Result<ApiResponse, Box<dyn std::error::Error>>{
    // ID 목록을 콤마로 연결하여 API URL 생성
    let ids_str = crypto_ids.join(",");
    // 설정 값으로 API URL을 동적으로 생성
    let api_url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies={}",
        ids_str, vs_currency
    );

    let response: reqwest::Response = client.get(api_url).send().await?;

    if !response.status().is_success() {
        return Err(format!("API SERVER ERROR (STATUS CODE: {})", response.status()).into());
    }

    // API 응답 전체를 반환
    Ok(response.json::<ApiResponse>().await?)
}


// 파일 저장 함수 추가
// 가져온 가격을 CSV 파일에 기록하는 함수
// 파일 경로를 인자로 받도록 수정
fn log_price_to_csv(prefix: &str, crypto_ids: &str, timestamp: &DateTime<Local>, price: f64) -> Result<(), std::io::Error> {

    // 파일을 추가 모드(append)로 열고, 파일이 없으면 새로 생성(create) 합니다.
    let file_path = format!("{}_{}.csv", prefix, crypto_ids);
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
    println!("----------------------------------------------------");

    analyze_and_print_stats(&settings.log_file_prefix, &settings.crypto_ids);
    println!("----------------------------------------------------");

    // 비기능 요구사항 : 5초 타임아웃 설정
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    let mut interval = time::interval(Duration::from_secs(settings.interval_seconds));

   
    println!(" Data collector has started. (Run every {} seconds)", settings.interval_seconds);
    println!("   - collection target: {}", settings.crypto_ids.join(", ").to_uppercase());
    println!("   - Press Ctrl+C to exit.");
    println!("----------------------------------------------------");

    //  우아한 종료 로직 추카

    loop {
        // tokio::select! 는 여러 비동기 작업 중 하나가 완료될 때 까지 기다림.
        tokio::select! {
            // 주기적인 작업 : interval 시간이 되면 이 블록이 실행됨
            _ = interval.tick() => {
                let now = Local::now();
                 // 설정 값을 fetch_prices 함수에 전달
                match fetch_prices(&client, &settings.crypto_ids, &settings.vs_currency).await {
                Ok(prices_map) => {
                    println!("[Time: {}] Data collection success", now.format("%Y-%m-%d %H:%M:%S"));
                    for (id, price_data) in prices_map {
                        let price = match settings.vs_currency.as_str() {
                                "usd" => price_data.usd,
                                "eur" => price_data.eur,
                                "jpy" => price_data.jpy,
                                _ => None,
                            };
                             if let Some(p) = price {
                                println!("   - {}/{}: {}", id.to_uppercase(), settings.vs_currency.to_uppercase(), p);
                                if let Err(e) = log_price_to_csv(&settings.log_file_prefix, &id, &now, p) {
                                    eprintln!(" ERROR '{}' An error occurred while saving the file: {}", id, e);
                                }
                            }
                        }
                    }   
                    Err(e) => {
                        eprintln!(" Data Collecting ERROR : {}", e);
                    }
                }
            }

        // 종료 신호 감지 : Ctrl + C 가 눌리면 이 블록이 실행됨.
            _ = signal::ctrl_c() => {
                println!("\n !! CTRL + C => Program safety exit.");
                break; // loop를 빠져나감.
            } 
        }
    }

    println!("Data Colleter is successful exit.");
    Ok(())
}

