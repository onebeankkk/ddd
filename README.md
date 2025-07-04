
1. 특정 시간마다 자동으로 비트코인 시세알려주는 코드
tokio의 interval 기능을 사용해 핵심 로직을 별도의 함수로 분라, main 함수에서는 이 함수를 주기적으로 호출

main.rs 수정된 코드

===== 주요 변경사항 설명 =====


fetch_and_print_price 함수 분리: API를 호출하고 결과를 출력하는 핵심 로직을 별도의 비동기 함수로 만들고 이렇게 하면 main 함수의 코드가 더 깔끔해지고 로직을 재사용하기 쉬워짐.
tokio::time::interval 사용: main 함수에서 time::interval(Duration::from_secs(60))를 사용하여 60초 간격으로 신호를 보내는 타이머
무한 루프 (loop): loop 블록 안에서 계속해서 작업을 반복
interval.tick().await;: 타이머의 다음 신호가 올 때까지 여기서 실행을 멈추고 대기
에러 처리: fetch_and_print_price 함수가 실패하여 Err를 반환하더라도, if let을 통해 에러 메시지만 출력하고 루프는 계속되고, 한 번의 네트워크 오류로 인해 전체 프로그램이 중단되는 것을 막을 수 있따.



2. 데이터를 파일에 저장하도록 만드는 기능


main.rs 수정된 코드

===== 주요 변경사항 설명 =====


fetch_price 함수 변경: 이제 이 함수는 화면에 출력하는 대신, 가져온 가격(f64)을 Result로 감싸서 반환.
-> 가져온 데이터를 다른 함수에서 활용 용이.

log_price_to_csv 함수 추가: 이 함수는 데이터를 파일에 쓰는 역할만 전담합니다.
OpenOptions를 사용하여 파일을 열 때, 파일이 없으면(create(true)) 새로 만들고, 항상 파일 끝에 내용을 추가(append(true))하도록 설정
file.metadata()?.len() == 0 코드로 파일 크기를 확인해서, 0이면 (즉, 방금 새로 만들어졌으면) CSV 헤더(timestamp,price)를 먼저 사용

main 함수 로직 수정:
fetch_price를 호출하여 성공적으로 가격을 가져오면(Ok(price)), 새로 만든 log_price_to_csv 함수를 호출하여 파일에 저장



3. 설정 파일에서 실행 값 읽어오기


main.rs 수정된 코드

===== 주요 변경사항 설명 =====


Settings 구조체: config.toml 파일의 구조와 정확히 일치하는 Rust 구조체를 정의했습니다. serde가 이 구조체를 보고 TOML 파일 내용을 자동으로 파싱

동적 API 응답 처리:
API 응답의 키("bitcoin")와 필드("usd")가 설정에 따라 변하므로, HashMap과 Option을 사용하여 어떤 암호화폐/통화 조합이든 처리할 수 있도록 응답 구조체(ApiResponse, PriceData)를 더 유연하게 변경했습니다.

설정 파일 로드:
main 함수 시작 부분에서 config::Config::builder()를 사용하여 config.toml 파일을 읽고, 그 내용을 Settings 구조체 인스턴스로 변환합니다.

설정 값 사용:
API URL 생성, interval 설정, 로그 파일 경로 지정 등 코드에 박혀있던 모든 값들을 settings 변수에서 가져와 사용하도록 수정
