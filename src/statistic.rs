use chrono::prelude::*;
use std::sync::Mutex;

lazy_static!{
    // server received requests until now.
    static ref REQ_NUM: Mutex<u64> = Mutex::new(0);
    // server sent responses number until now.
    static ref RESP_NUM: Mutex<u64> = Mutex::new(0);
    // 200 response number
    static ref RESP_200_NUM: Mutex<u64> = Mutex::new(0);
    // 301 response number
    static ref RESP_301_NUM: Mutex<u64> = Mutex::new(0);
    // 302 response number
    static ref RESP_302_NUM: Mutex<u64> = Mutex::new(0);
    // 400 response number
    static ref RESP_400_NUM: Mutex<u64> = Mutex::new(0);
    // 403 response number
    static ref RESP_403_NUM: Mutex<u64> = Mutex::new(0);
    // 404 response number
    static ref RESP_404_NUM: Mutex<u64> = Mutex::new(0);
    // 500 response number
    static ref RESP_500_NUM: Mutex<u64> = Mutex::new(0);
    // 501 response number
    static ref RESP_501_NUM: Mutex<u64> = Mutex::new(0);
    // 502 response number
    static ref RESP_502_NUM: Mutex<u64> = Mutex::new(0);
    // 503 response number
    static ref RESP_503_NUM: Mutex<u64> = Mutex::new(0);
}

// inc request number
pub fn inc_req_num() {
    *REQ_NUM.lock().unwrap() += 1;
}

// inc response number
pub fn inc_resp_num() {
    *RESP_NUM.lock().unwrap() += 1;
}

// inc 200 response number
pub fn inc_200_resp_num() {
    *RESP_200_NUM.lock().unwrap() += 1;
}

// inc 301 response number
pub fn inc_301_resp_num() {
    *RESP_301_NUM.lock().unwrap() += 1;
}

// inc 302 response number
pub fn inc_302_resp_num() {
    *RESP_302_NUM.lock().unwrap() += 1;
}

// inc 400 response number
pub fn inc_400_resp_num() {
    *RESP_400_NUM.lock().unwrap() += 1;
}

// inc 403 response number
pub fn inc_403_resp_num() {
    *RESP_403_NUM.lock().unwrap() += 1;
}

// inc 404 response number
pub fn inc_404_resp_num() {
    *RESP_404_NUM.lock().unwrap() += 1;
}

// inc 500 response number
pub fn inc_500_resp_num() {
    *RESP_500_NUM.lock().unwrap() += 1;
}

// inc 501 response number
pub fn inc_501_resp_num() {
    *RESP_501_NUM.lock().unwrap() += 1;
}

// inc 502 response number
pub fn inc_502_resp_num() {
    *RESP_502_NUM.lock().unwrap() += 1;
}

// inc 503 response number
pub fn inc_503_resp_num() {
    *RESP_503_NUM.lock().unwrap() += 1;
}


// show statistics
pub fn show_statistics() {
    let local: DateTime<Local> = Local::now();
    let current_time = local.format("%Y/%m/%d %H:%M:%S").to_string();
    println!("################################################################################");
    println!("Current time: {}", current_time);
    println!(" __ Connections ____________________________");
    println!("|                                           |");
    println!("|   Connecting        : {}", 1000);
    println!("|___________________________________________|");
    println!(" __ Request ________________________________");
    println!("|                                           |");
    println!("|   Total Request     : {}", *REQ_NUM.lock().unwrap());
    println!("|   Total Response    : {}", *RESP_NUM.lock().unwrap());
    println!("|   200 Response      : {}", *RESP_200_NUM.lock().unwrap());
    println!("|   301 Response      : {}", *RESP_301_NUM.lock().unwrap());
    println!("|   302 Response      : {}", *RESP_302_NUM.lock().unwrap());
    println!("|   400 Response      : {}", *RESP_400_NUM.lock().unwrap());
    println!("|   403 Response      : {}", *RESP_403_NUM.lock().unwrap());
    println!("|   404 Response      : {}", *RESP_404_NUM.lock().unwrap());
    println!("|   500 Response      : {}", *RESP_500_NUM.lock().unwrap());
    println!("|   501 Response      : {}", *RESP_500_NUM.lock().unwrap());
    println!("|   502 Response      : {}", *RESP_502_NUM.lock().unwrap());
    println!("|   503 Response      : {}", *RESP_503_NUM.lock().unwrap());
    println!("|___________________________________________|");
    println!("################################################################################");
}