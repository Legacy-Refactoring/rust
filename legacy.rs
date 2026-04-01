// legacy.rs
// Extremely insecure legacy payment system in Rust
// Educational bad code example - full of SQL injection, plain text secrets, massive code duplication

use postgres::{Client, NoTls, Row};
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

const DB_HOST: &str = "localhost";
const DB_PORT: &str = "5432";
const DB_NAME: &str = "payment_legacy_db";
const DB_USER: &str = "postgres";
const DB_PASS: &str = "SuperSecret123!";
const SITE_SECRET: &str = "myglobalsecret123";

static mut GLOBAL_CLIENT: Option<Client> = None;

fn get_client() -> &'static mut Client {
    unsafe {
        if GLOBAL_CLIENT.is_none() {
            let conn_str = format!(
                "host={} port={} user={} password={} dbname={} sslmode=disable",
                DB_HOST, DB_PORT, DB_USER, DB_PASS, DB_NAME
            );
            let client = Client::connect(&conn_str, NoTls).expect("CRITICAL DATABASE FAILURE");
            let _ = client.execute("SET client_encoding = 'UTF8';", &[]);
            GLOBAL_CLIENT = Some(client);
        }
        GLOBAL_CLIENT.as_mut().unwrap()
    }
}

fn append_to_log(msg: &str) {
    if let Ok(mut file) = OpenOptions::new().append(true).create(true).open("legacy_errors.log") {
        let _ = writeln!(file, "{} | {}", chrono::Utc::now().to_rfc3339(), msg);
    }
}

fn register_customer(username: &str, email: &str, password: &str, full_name: &str, phone: &str, country: &str, city: &str, address: &str) -> String {
    let client = get_client();
    let id = format!("cust_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
    let sql = format!(
        "INSERT INTO customers (
            id, username, email, password, full_name, phone, country, city, address_line_1,
            created_at, updated_at, register_ip, user_agent, is_admin, role_name
        ) VALUES (
            '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}',
            NOW()::text, NOW()::text, '127.0.0.1', 'RUST-LEGACY', 'false', 'customer'
        ) RETURNING id;",
        id, username, email, password, full_name, phone, country, city, address
    );
    match client.query_one(&sql, &[]) {
        Ok(row) => {
            let new_id: String = row.get(0);
            println!("Customer registered ID: {}", new_id);
            new_id
        }
        Err(e) => {
            println!("[ERROR] {}", e);
            append_to_log(&format!("{} | SQL: {}", e, sql));
            String::new()
        }
    }
}

fn login_customer(username: &str, password: &str) -> String {
    let client = get_client();
    let sql = format!("SELECT * FROM customers WHERE username = '{}' AND password = '{}' LIMIT 1;", username, password);
    match client.query_opt(&sql, &[]) {
        Ok(Some(row)) => {
            let id: String = row.get("id");
            let session_token = format!("{:x}", md5::compute(format!("{}{}{}", id, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(), SITE_SECRET)));
            let update = format!("UPDATE customers SET session_token = '{}', last_login_ip = '127.0.0.1', failed_login_count = '0', updated_at = NOW()::text WHERE id = '{}';", session_token, id);
            let _ = client.execute(&update, &[]);
            println!("LOGIN SUCCESS Session: {}", session_token);
            session_token
        }
        _ => {
            let fail_sql = format!("UPDATE customers SET failed_login_count = (failed_login_count::int + 1)::text WHERE username = '{}';", username);
            let _ = client.execute(&fail_sql, &[]);
            println!("LOGIN FAILED");
            String::new()
        }
    }
}

fn get_customer(customer_id: &str) -> Option<Row> {
    let client = get_client();
    let sql = format!("SELECT * FROM customers WHERE id = '{}' LIMIT 1;", customer_id);
    match client.query_opt(&sql, &[]) {
        Ok(row) => row,
        Err(e) => {
            println!("[ERROR] {}", e);
            append_to_log(&format!("{} | SQL: {}", e, sql));
            None
        }
    }
}

fn update_customer_profile(customer_id: &str, new_email: &str, new_phone: &str, new_address: &str) {
    let client = get_client();
    let sql = format!("UPDATE customers SET email = '{}', phone = '{}', address_line_1 = '{}', updated_at = NOW()::text WHERE id = '{}';", new_email, new_phone, new_address, customer_id);
    if let Err(e) = client.execute(&sql, &[]) {
        println!("[ERROR] {}", e);
        append_to_log(&format!("{} | SQL: {}", e, sql));
    } else {
        println!("Customer profile updated");
    }
}

fn reset_password(email: &str, new_password: &str) {
    let client = get_client();
    let sql = format!("UPDATE customers SET password = '{}', reset_token = 'reset_' || md5(NOW()::text), reset_token_expires_at = (NOW() + INTERVAL '1 day')::text WHERE email = '{}';", new_password, email);
    if let Err(e) = client.execute(&sql, &[]) {
        println!("[ERROR] {}", e);
        append_to_log(&format!("{} | SQL: {}", e, sql));
    } else {
        println!("Password reset token generated for {}", email);
    }
}

fn verify_email(token: &str) {
    let client = get_client();
    let sql = format!("UPDATE customers SET email_verification_token = NULL WHERE email_verification_token = '{}';", token);
    if let Err(e) = client.execute(&sql, &[]) {
        println!("[ERROR] {}", e);
        append_to_log(&format!("{} | SQL: {}", e, sql));
    } else {
        println!("Email verified with token {}", token);
    }
}

fn add_payment_method(customer_id: &str, type_: &str, card_number: &str, expiry_month: &str, expiry_year: &str, cvv: &str, holder_name: &str, iban: &str) -> String {
    let client = get_client();
    let id = format!("pm_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
    let sql = format!(
        "INSERT INTO payment_methods (
            id, customer_id, type, provider, card_number, card_expiry_month, card_expiry_year, 
            card_cvv, card_holder_name, iban, active_flag, created_at, updated_at
        ) VALUES (
            '{}', '{}', '{}', 'legacy_bank_gateway', '{}', '{}', '{}', '{}', '{}', '{}', 'true', NOW()::text, NOW()::text
        ) RETURNING id;",
        id, customer_id, type_, card_number, expiry_month, expiry_year, cvv, holder_name, iban
    );
    match client.query_one(&sql, &[]) {
        Ok(row) => {
            let new_id: String = row.get(0);
            println!("Payment method added ID: {}", new_id);
            new_id
        }
        Err(e) => {
            println!("[ERROR] {}", e);
            append_to_log(&format!("{} | SQL: {}", e, sql));
            String::new()
        }
    }
}

fn list_payment_methods(customer_id: &str) -> Vec<Row> {
    let client = get_client();
    let sql = format!("SELECT * FROM payment_methods WHERE customer_id = '{}' AND deleted_at IS NULL;", customer_id);
    match client.query(&sql, &[]) {
        Ok(rows) => rows,
        Err(e) => {
            println!("[ERROR] {}", e);
            append_to_log(&format!("{} | SQL: {}", e, sql));
            vec![]
        }
    }
}

fn delete_payment_method(pm_id: &str) {
    let client = get_client();
    let sql = format!("UPDATE payment_methods SET deleted_at = NOW()::text WHERE id = '{}';", pm_id);
    if let Err(e) = client.execute(&sql, &[]) {
        println!("[ERROR] {}", e);
        append_to_log(&format!("{} | SQL: {}", e, sql));
    } else {
        println!("Payment method deleted");
    }
}

fn process_payment(customer_id: &str, payment_method_id: &str, amount: &str, currency: &str, external_order_id: &str, ip: &str) -> String {
    let client = get_client();
    let id = format!("pay_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
    let real_ip = if ip.is_empty() { "127.0.0.1" } else { ip };
    let ext_order = if external_order_id.is_empty() { format!("ord_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()) } else { external_order_id.to_string() };
    let raw_payload = r#"{"card_number":"****4242","provider_secret":"sk_live_9876543210abcdef","cvv_used":"123","3ds_password":"customer123"}"#;

    let sql = format!(
        "INSERT INTO payments (
            id, customer_id, payment_method_id, external_order_id, amount, currency, status,
            provider_ref, ip_address, raw_provider_payload, created_at, paid_at, captured_flag
        ) VALUES (
            '{}', '{}', '{}', '{}', '{}', '{}', 'captured',
            'prov_{}', '{}', '{}', NOW()::text, NOW()::text, 'true'
        ) RETURNING id;",
        id, customer_id, payment_method_id, ext_order, amount, currency,
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(), real_ip, raw_payload
    );

    match client.query_one(&sql, &[]) {
        Ok(row) => {
            let pay_id: String = row.get(0);
            let update = format!("UPDATE customers SET total_paid = (COALESCE(total_paid::numeric, 0) + {})::text WHERE id = '{}';", amount, customer_id);
            let _ = client.execute(&update, &[]);

            let log_sql = format!(
                "INSERT INTO payment_logs (id, payment_id, customer_id, log_level, message, payload, created_at, actor_email, source)
                 VALUES ('log_' || nextval('payment_logs_id_seq'::regclass), '{}', '{}', 'INFO', 'Payment captured successfully', '{}', NOW()::text, 'system@legacy.com', 'legacy_core');",
                pay_id, customer_id, raw_payload.replace("'", "''")
            );
            let _ = client.execute(&log_sql, &[]);

            println!("PAYMENT PROCESSED ID: {} Amount: {} {}", pay_id, amount, currency);
            pay_id
        }
        Err(e) => {
            println!("[ERROR] {}", e);
            append_to_log(&format!("{} | SQL: {}", e, sql));
            String::new()
        }
    }
}

fn list_payments(customer_id: &str) -> Vec<Row> {
    let client = get_client();
    let sql = format!("SELECT * FROM payments WHERE customer_id = '{}' ORDER BY created_at DESC;", customer_id);
    match client.query(&sql, &[]) {
        Ok(rows) => rows,
        Err(e) => {
            println!("[ERROR] {}", e);
            append_to_log(&format!("{} | SQL: {}", e, sql));
            vec![]
        }
    }
}

fn get_payment_details(payment_id: &str) -> Option<Row> {
    let client = get_client();
    let sql = format!("SELECT * FROM payments WHERE id = '{}' LIMIT 1;", payment_id);
    match client.query_opt(&sql, &[]) {
        Ok(row) => row,
        Err(e) => {
            println!("[ERROR] {}", e);
            append_to_log(&format!("{} | SQL: {}", e, sql));
            None
        }
    }
}

fn create_refund(payment_id: &str, amount: &str, reason: &str) {
    let client = get_client();
    let id = format!("ref_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
    let sql = format!(
        "INSERT INTO refunds (id, payment_id, amount, currency, status, reason, created_at)
         VALUES ('{}', '{}', '{}', 'EUR', 'pending', '{}', NOW()::text);",
        id, payment_id, amount, reason
    );
    if let Err(e) = client.execute(&sql, &[]) {
        println!("[ERROR] {}", e);
        append_to_log(&format!("{} | SQL: {}", e, sql));
    } else {
        println!("Refund created for payment {}", payment_id);
    }
}

fn process_refund(refund_id: &str) {
    let client = get_client();
    let sql = format!("UPDATE refunds SET status = 'processed', processed_at = NOW()::text WHERE id = '{}';", refund_id);
    if let Err(e) = client.execute(&sql, &[]) {
        println!("[ERROR] {}", e);
        append_to_log(&format!("{} | SQL: {}", e, sql));
    } else {
        println!("Refund processed ID: {}", refund_id);
    }
}

fn simulate_chargeback(payment_id: &str, amount: &str, reason: &str) {
    let client = get_client();
    let id = format!("cb_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
    let sql = format!(
        "INSERT INTO chargebacks (id, payment_id, amount, currency, reason, status, created_at, deadline_at)
         VALUES ('{}', '{}', '{}', 'EUR', '{}', 'open', NOW()::text, (NOW() + INTERVAL '7 days')::text);",
        id, payment_id, amount, reason
    );
    if let Err(e) = client.execute(&sql, &[]) {
        println!("[ERROR] {}", e);
        append_to_log(&format!("{} | SQL: {}", e, sql));
    } else {
        println!("Chargeback created for payment {}", payment_id);
    }
}

fn resolve_chargeback(chargeback_id: &str, won: &str) {
    let client = get_client();
    let sql = format!("UPDATE chargebacks SET status = 'closed', won_flag = '{}', closed_at = NOW()::text WHERE id = '{}';", won, chargeback_id);
    if let Err(e) = client.execute(&sql, &[]) {
        println!("[ERROR] {}", e);
        append_to_log(&format!("{} | SQL: {}", e, sql));
    } else {
        println!("Chargeback resolved ID: {}", chargeback_id);
    }
}

fn create_fraud_review(payment_id: &str, customer_id: &str, score: &str) {
    let client = get_client();
    let id = format!("fraud_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
    let sql = format!(
        "INSERT INTO fraud_reviews (id, payment_id, customer_id, score, decision, created_at)
         VALUES ('{}', '{}', '{}', '{}', 'pending', NOW()::text);",
        id, payment_id, customer_id, score
    );
    if let Err(e) = client.execute(&sql, &[]) {
        println!("[ERROR] {}", e);
        append_to_log(&format!("{} | SQL: {}", e, sql));
    } else {
        println!("Fraud review created for payment {}", payment_id);
    }
}

fn decide_fraud_review(review_id: &str, decision: &str, reviewer_email: &str, reviewer_password: &str) {
    let client = get_client();
    let check = format!("SELECT * FROM customers WHERE email = '{}' AND password = '{}' AND is_admin = 'true';", reviewer_email, reviewer_password);
    match client.query_opt(&check, &[]) {
        Ok(Some(_)) => {
            let sql = format!("UPDATE fraud_reviews SET decision = '{}', reviewer = '{}', updated_at = NOW()::text WHERE id = '{}';", decision, reviewer_email, review_id);
            if let Err(e) = client.execute(&sql, &[]) {
                println!("[ERROR] {}", e);
                append_to_log(&format!("{} | SQL: {}", e, sql));
            } else {
                println!("Fraud review decided as {}", decision);
            }
        }
        _ => {
            println!("Fraud review access denied");
        }
    }
}

fn admin_export_all_data() {
    let client = get_client();
    let sql = format!(
        "COPY (
            SELECT * FROM customers 
            UNION ALL SELECT * FROM payments 
            UNION ALL SELECT * FROM payment_methods 
            UNION ALL SELECT * FROM refunds 
            UNION ALL SELECT * FROM chargebacks 
            UNION ALL SELECT * FROM fraud_reviews
        ) TO '/tmp/legacy_full_export_{}.csv' WITH CSV HEADER;",
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    );
    if let Err(e) = client.execute(&sql, &[]) {
        println!("[ERROR] {}", e);
        append_to_log(&format!("{} | SQL: {}", e, sql));
    } else {
        println!("Full data export completed to /tmp/legacy_full_export_*.csv");
    }
}

fn search_payments(search_term: &str) -> Vec<Row> {
    let client = get_client();
    let sql = format!("SELECT * FROM payments WHERE raw_provider_payload LIKE '%{}%' OR error_message LIKE '%{}%';", search_term, search_term);
    match client.query(&sql, &[]) {
        Ok(rows) => rows,
        Err(e) => {
            println!("[ERROR] {}", e);
            append_to_log(&format!("{} | SQL: {}", e, sql));
            vec![]
        }
    }
}

fn process_recurring_billing() {
    let client = get_client();
    let sql = "SELECT * FROM payments WHERE status = 'captured' AND installment_count > '0';";
    match client.query(sql, &[]) {
        Ok(rows) => {
            for row in rows {
                let id: String = row.get("id");
                let customer_id: String = row.get("customer_id");
                let payment_method_id: String = row.get("payment_method_id");
                let amount: String = row.get("amount");
                let currency: String = row.get("currency");
                println!("Recurring payment processed for {}", id);
                process_payment(&customer_id, &payment_method_id, &amount, &currency, "", "");
            }
        }
        Err(e) => {
            println!("[ERROR] {}", e);
            append_to_log(&format!("{} | SQL: {}", e, sql));
        }
    }
}

fn handle_webhook(payload: &str) {
    let client = get_client();
    let raw: serde_json::Value = serde_json::from_str(payload).unwrap_or_default();
    if let Some(payment_id) = raw["payment_id"].as_str() {
        let sql = format!("UPDATE payments SET status = 'settled', settled_at = NOW()::text WHERE id = '{}';", payment_id);
        let _ = client.execute(&sql, &[]);
        let log_sql = format!(
            "INSERT INTO payment_logs (id, payment_id, customer_id, log_level, message, payload, created_at, actor_email, source)
             VALUES ('log_' || nextval('payment_logs_id_seq'::regclass), '{}', '{}', 'INFO', 'Webhook received', '{}', NOW()::text, 'system@legacy.com', 'legacy_core');",
            payment_id, raw["customer_id"].as_str().unwrap_or(""), payload.replace("'", "''")
        );
        let _ = client.execute(&log_sql, &[]);
        println!("Webhook processed");
    }
}

fn ban_customer(customer_id: &str) {
    let client = get_client();
    let sql = format!("UPDATE customers SET blocked_flag = 'true' WHERE id = '{}';", customer_id);
    if let Err(e) = client.execute(&sql, &[]) {
        println!("[ERROR] {}", e);
        append_to_log(&format!("{} | SQL: {}", e, sql));
    } else {
        println!("Customer banned");
    }
}

fn generate_api_key(customer_id: &str) {
    let client = get_client();
    let key = format!("key_{:x}", md5::compute(format!("{}{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(), SITE_SECRET)));
    let secret = format!("secret_{:x}", md5::compute(format!("{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis())));
    let sql = format!("UPDATE customers SET api_key = '{}', api_secret = '{}' WHERE id = '{}';", key, secret, customer_id);
    if let Err(e) = client.execute(&sql, &[]) {
        println!("[ERROR] {}", e);
        append_to_log(&format!("{} | SQL: {}", e, sql));
    } else {
        println!("API key generated: {}", key);
    }
}

fn main() {
    println!("LEGACY PAYMENT SYSTEM STARTED (Rust version)");

    let cust1 = register_customer("testuser1", "test1@example.com", "PlainPass123", "Test User One", "381601234567", "RS", "Belgrade", "Novi Beograd 1");
    let cust2 = register_customer("testuser2", "test2@example.com", "AnotherPass456", "Test User Two", "381609876543", "RS", "Novi Sad", "Address 2");

    login_customer("testuser1", "PlainPass123");
    login_customer("testuser2", "AnotherPass456");

    let pm1 = add_payment_method(&cust1, "card", "4242424242424242", "12", "2028", "123", "Test User One", "");
    let pm2 = add_payment_method(&cust2, "iban", "", "", "", "", "Test User Two", "RS12345678901234567890");

    let pay1 = process_payment(&cust1, &pm1, "149.99", "EUR", "ORDER-1001", "");
    let pay2 = process_payment(&cust2, &pm2, "299.50", "USD", "ORDER-1002", "");

    create_refund(&pay1, "49.99", "partial return");
    process_refund(&format!("ref_{}", &pay1[4..]));

    simulate_chargeback(&pay2, "299.50", "dispute");
    resolve_chargeback(&format!("cb_{}", &pay2[4..]), "false");

    create_fraud_review(&pay1, &cust1, "78");
    decide_fraud_review(&format!("fraud_{}", &pay1[4..]), "approve", "admin@legacy.com", "AdminPass123");

    reset_password("test1@example.com", "NewPlainPass789");
    verify_email("email_verify_token_demo");

    admin_export_all_data();

    process_recurring_billing();

    let webhook_payload = r#"{"payment_id":"PAY123","customer_id":"CUST123","status":"settled"}"#;
    handle_webhook(webhook_payload);

    generate_api_key(&cust1);
    ban_customer(&cust2);

    println!("LEGACY PAYMENT SYSTEM WORKFLOW COMPLETE");
}