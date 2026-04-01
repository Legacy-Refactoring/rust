-- seeder.sql
-- intentionally awful bulk seeder for legacy payment lab

-- recommended:
-- run after init.sql
-- psql -U legacy_user -d legacy_payments -f seeder.sql

-- optional cleanup
-- TRUNCATE TABLE payment_logs;
-- TRUNCATE TABLE fraud_reviews;
-- TRUNCATE TABLE chargebacks;
-- TRUNCATE TABLE refunds;
-- TRUNCATE TABLE payments;
-- TRUNCATE TABLE payment_methods;
-- TRUNCATE TABLE customers;

-- =========================
-- volume configuration
-- =========================
-- change these numbers as needed
-- be careful with payment_logs because it grows very fast

-- customers:         1,000,000
-- payment_methods:   1,000,000
-- payments:          3,000,000
-- refunds:             300,000
-- chargebacks:         120,000
-- fraud_reviews:       250,000
-- payment_logs:      4,000,000

-- =========================
-- customers
-- =========================
INSERT INTO customers (
    id,
    email,
    username,
    phone,
    country,
    city,
    address_line_1,
    address_line_2,
    postal_code,
    full_name,
    first_name,
    last_name,
    password,
    password_hint,
    secret_answer,
    security_question,
    plain_backup_code,
    reset_token,
    reset_token_expires_at,
    email_verification_token,
    session_token,
    api_key,
    api_secret,
    internal_notes,
    public_notes,
    risk_score,
    blocked_flag,
    is_admin,
    role_name,
    failed_login_count,
    last_login_ip,
    register_ip,
    user_agent,
    marketing_source,
    referral_code,
    total_paid,
    total_refunded,
    preferred_currency,
    created_at,
    updated_at,
    deleted_at
)
SELECT
    gs::text,
    'user' || gs || '@example.com',
    'legacy_user_' || gs,
    '+3816' || lpad((gs % 10000000)::text, 7, '0'),
    (ARRAY['US','RS','DE','GB','FR','CA','AU','BR','IN'])[1 + (random() * 7)::int],
    (ARRAY['Belgrade','Novi Sad','Berlin','Paris','London','Toronto','Mumbai','Chicago'])[1 + (random() * 7)::int],
    'Street ' || gs || ' Apt ' || (gs % 9999),
    'Floor ' || (gs % 30) || ' <script>alert("xss-address")</script>',
    lpad((10000 + (gs % 89999))::text, 5, '0'),
    'User ' || gs || ' O''Connor',
    'Name' || gs,
    'Last' || gs,
    CASE
        WHEN gs % 50 = 0 THEN '123456'
        WHEN gs % 75 = 0 THEN 'password'
        WHEN gs % 120 = 0 THEN 'admin'
        ELSE 'plain-pass-' || gs
    END,
    'pet',
    CASE
        WHEN gs % 90 = 0 THEN ''' OR ''1''=''1'
        WHEN gs % 130 = 0 THEN '<script>alert("secret")</script>'
        ELSE 'blue'
    END,
    'What is your pet name?',
    'backup-' || gs || '-0000',
    'reset-token-' || gs,
    now()::text,
    'verify-token-' || gs,
    'session-token-' || gs,
    'api-key-' || gs,
    'api-secret-' || gs || '-SUPER-PLAINTEXT',
    CASE
        WHEN gs % 100 = 0 THEN 'VIP customer; do not refund; card=4111111111111111 cvv=123'
        WHEN gs % 333 = 0 THEN 'customer said: <img src=x onerror=alert(1)>'
        ELSE 'internal note ' || gs
    END,
    CASE
        WHEN gs % 200 = 0 THEN '<script>alert("public-note")</script>'
        ELSE 'public note ' || gs
    END,
    (random() * 100)::int::text,
    CASE WHEN gs % 40 = 0 THEN 'true' ELSE 'false' END,
    CASE WHEN gs % 5000 = 0 THEN 'true' ELSE 'false' END,
    CASE
        WHEN gs % 5000 = 0 THEN 'superadmin'
        WHEN gs % 2000 = 0 THEN 'admin'
        ELSE 'customer'
    END,
    (random() * 12)::int::text,
    '192.168.' || (gs % 255) || '.' || ((gs / 255) % 255),
    '10.0.' || (gs % 255) || '.' || ((gs / 255) % 255),
    CASE
        WHEN gs % 111 = 0 THEN 'Mozilla/5.0 <script>alert("ua")</script>'
        ELSE 'Mozilla/5.0 LegacyBrowser ' || gs
    END,
    (ARRAY['facebook','google','affiliate','newsletter','popup','unknown','<script>x</script>'])[1 + (random() * 6)::int],
    'REF-' || gs,
    (round((random() * 50000)::numeric, 2))::text,
    (round((random() * 7000)::numeric, 2))::text,
    (ARRAY['USD','EUR','RSD','GBP'])[1 + (random() * 3)::int],
    now()::text,
    now()::text,
    CASE WHEN gs % 10000 = 0 THEN now()::text ELSE NULL END
FROM generate_series(1, 1000000) AS gs;

-- =========================
-- payment methods
-- one method per customer
-- =========================
INSERT INTO payment_methods (
    id,
    customer_id,
    type,
    provider,
    provider_account_id,
    token,
    reusable_token,
    masked_ref,
    card_holder_name,
    card_number,
    card_expiry_month,
    card_expiry_year,
    card_cvv,
    card_pin,
    card_blob,
    iban,
    bank_account_number,
    routing_number,
    swift_code,
    billing_full_name,
    billing_email,
    billing_phone,
    billing_country,
    billing_city,
    billing_address,
    billing_zip,
    billing_data,
    auth_pin,
    three_ds_password,
    active_flag,
    verification_status,
    internal_comment,
    created_at,
    updated_at,
    deleted_at
)
SELECT
    ('pm-' || gs),
    gs::text,
    CASE
        WHEN gs % 10 = 0 THEN 'bank'
        WHEN gs % 3 = 0 THEN 'wallet'
        ELSE 'card'
    END,
    (ARRAY['legacy-gateway','broken-pay','bad-processor','old-bank-xml'])[1 + (random() * 3)::int],
    'acct-' || gs,
    'tok-' || gs || '-plaintext',
    'reusable-' || gs,
    '****' || lpad((1000 + (gs % 9000))::text, 4, '0'),
    'User ' || gs,
    CASE
        WHEN gs % 2 = 0 THEN '4111111111111111'
        WHEN gs % 3 = 0 THEN '5555444433331111'
        ELSE '400000000000' || lpad((gs % 10000)::text, 4, '0')
    END,
    lpad(((1 + (gs % 12)))::text, 2, '0'),
    (2026 + (gs % 7))::text,
    lpad((100 + (gs % 899))::text, 3, '0'),
    lpad((1000 + (gs % 8999))::text, 4, '0'),
    '4111111111111111|' || lpad(((1 + (gs % 12)))::text, 2, '0') || '/' || (2026 + (gs % 7))::text || '|' || lpad((100 + (gs % 899))::text, 3, '0'),
    'RS35260005601001611379',
    'BA-' || gs,
    'ROUT-' || gs,
    'SWFTCODE' || (gs % 1000),
    'Billing User ' || gs,
    'billing' || gs || '@example.com',
    '+3816' || lpad((gs % 10000000)::text, 7, '0'),
    (ARRAY['US','RS','DE','GB'])[1 + (random() * 3)::int],
    (ARRAY['Belgrade','Berlin','London','Chicago'])[1 + (random() * 3)::int],
    'Billing Address ' || gs || ' <script>alert("billing")</script>',
    lpad((10000 + (gs % 89999))::text, 5, '0'),
    '{"zip":"11000","rawCard":"4111111111111111","cvv":"123"}',
    '0000',
    '3ds-password-' || gs,
    CASE WHEN gs % 20 = 0 THEN 'false' ELSE 'true' END,
    CASE WHEN gs % 15 = 0 THEN 'failed' ELSE 'verified' END,
    CASE
        WHEN gs % 222 = 0 THEN 'manual override; token='' OR ''1''=''1'
        ELSE 'comment ' || gs
    END,
    now()::text,
    now()::text,
    CASE WHEN gs % 25000 = 0 THEN now()::text ELSE NULL END
FROM generate_series(1, 1000000) AS gs;

-- =========================
-- payments
-- three per customer on average
-- =========================
INSERT INTO payments (
    id,
    customer_id,
    payment_method_id,
    external_order_id,
    external_cart_id,
    merchant_account_id,
    amount,
    fee_amount,
    tax_amount,
    discount_amount,
    net_amount,
    refund_limit_amount,
    currency,
    base_currency,
    fx_rate,
    status,
    sub_status,
    provider_ref,
    provider_transaction_id,
    gateway_response_code,
    gateway_response_message,
    fraud_flag,
    manual_review_flag,
    captured_flag,
    settled_flag,
    installment_count,
    error_message,
    raw_provider_payload,
    raw_client_payload,
    metadata,
    browser_info,
    ip_address,
    device_id,
    geo_country,
    geo_city,
    idempotency_key,
    duplicate_group_key,
    retry_count,
    previous_payment_id,
    settlement_bucket,
    created_by,
    approved_by,
    created_at,
    updated_at,
    paid_at,
    settled_at,
    expired_at
)
SELECT
    ('pay-' || gs),
    (((gs - 1) % 1000000) + 1)::text,
    ('pm-' || (((gs - 1) % 1000000) + 1)),
    'order-' || gs,
    'cart-' || gs,
    'merchant-' || ((gs % 250) + 1),
    (round((random() * 10000 + 1)::numeric, 2))::text,
    (round((random() * 100)::numeric, 2))::text,
    (round((random() * 50)::numeric, 2))::text,
    (round((random() * 30)::numeric, 2))::text,
    (round((random() * 9000)::numeric, 2))::text,
    (round((random() * 5000)::numeric, 2))::text,
    (ARRAY['USD','EUR','GBP','RSD'])[1 + (random() * 3)::int],
    'USD',
    (round((0.8 + random() * 0.6)::numeric, 4))::text,
    CASE
        WHEN gs % 30 = 0 THEN 'failed'
        WHEN gs % 25 = 0 THEN 'chargeback'
        WHEN gs % 18 = 0 THEN 'pending'
        WHEN gs % 12 = 0 THEN 'manual_review'
        WHEN gs % 9 = 0 THEN 'success'
        ELSE 'created'
    END,
    CASE
        WHEN gs % 90 = 0 THEN 'gateway_timeout'
        WHEN gs % 140 = 0 THEN 'needs_manual_review'
        ELSE ''
    END,
    'provider-ref-' || gs,
    'txn-' || gs,
    CASE
        WHEN gs % 30 = 0 THEN '500'
        WHEN gs % 12 = 0 THEN '202'
        ELSE '200'
    END,
    CASE
        WHEN gs % 30 = 0 THEN 'timeout'
        WHEN gs % 44 = 0 THEN '<script>alert("gateway")</script>'
        ELSE 'ok'
    END,
    CASE WHEN gs % 25 = 0 OR gs % 13 = 0 THEN 'true' ELSE 'false' END,
    CASE WHEN gs % 12 = 0 THEN 'true' ELSE 'false' END,
    CASE WHEN gs % 4 = 0 THEN 'true' ELSE 'false' END,
    CASE WHEN gs % 7 = 0 THEN 'true' ELSE 'false' END,
    ((gs % 12) + 1)::text,
    CASE
        WHEN gs % 30 = 0 THEN 'provider timeout'
        WHEN gs % 333 = 0 THEN ''' OR ''1''=''1'
        ELSE ''
    END,
    '{"secret":"provider-secret-' || gs || '","status":"ok"}',
    '{"card":"4111111111111111","cvv":"123","note":"<script>alert(1)</script>"}',
    '{"source":"seed","campaign":"legacy-' || (gs % 100) || '"}',
    CASE
        WHEN gs % 400 = 0 THEN '<img src=x onerror=alert(1)>'
        ELSE 'Mozilla Payment Browser ' || gs
    END,
    '172.16.' || (gs % 255) || '.' || ((gs / 255) % 255),
    'device-' || md5(gs::text),
    (ARRAY['US','RS','DE','GB','FR'])[1 + (random() * 4)::int],
    (ARRAY['Belgrade','Berlin','London','Paris','Austin'])[1 + (random() * 4)::int],
    CASE WHEN gs % 8 = 0 THEN '' ELSE 'idem-' || gs END,
    'dup-' || (((gs - 1) % 50000) + 1),
    (gs % 6)::text,
    CASE WHEN gs > 1 THEN 'pay-' || (gs - 1) ELSE '' END,
    CASE WHEN gs % 2 = 0 THEN 'legacy' ELSE 'nightly' END,
    'system',
    CASE WHEN gs % 500 = 0 THEN 'admin@example.com' ELSE '' END,
    now()::text,
    now()::text,
    now()::text,
    now()::text,
    CASE WHEN gs % 100 = 0 THEN now()::text ELSE NULL END
FROM generate_series(1, 3000000) AS gs;

-- =========================
-- refunds
-- =========================
INSERT INTO refunds (
    id,
    payment_id,
    external_refund_id,
    amount,
    currency,
    status,
    reason,
    provider_ref,
    requested_by,
    approved_by,
    customer_message,
    internal_message,
    raw_provider_payload,
    notes,
    created_at,
    updated_at,
    processed_at
)
SELECT
    ('ref-' || gs),
    'pay-' || ((random() * 2999999 + 1)::bigint),
    'ext-ref-' || gs,
    (round((random() * 500)::numeric, 2))::text,
    (ARRAY['USD','EUR','GBP','RSD'])[1 + (random() * 3)::int],
    CASE
        WHEN gs % 10 = 0 THEN 'failed'
        WHEN gs % 4 = 0 THEN 'processing'
        ELSE 'created'
    END,
    CASE
        WHEN gs % 50 = 0 THEN '<script>alert("refund")</script>'
        WHEN gs % 90 = 0 THEN 'customer said '' OR 1=1 --'
        ELSE 'refund reason ' || gs
    END,
    'refund-provider-' || gs,
    CASE WHEN gs % 7 = 0 THEN 'support@example.com' ELSE 'system' END,
    CASE WHEN gs % 9 = 0 THEN 'manager@example.com' ELSE '' END,
    'please refund payment ' || gs,
    'internal refund note card=4111111111111111 cvv=123',
    '{"raw":"refund-payload-' || gs || '"}',
    'note ' || gs,
    now()::text,
    now()::text,
    now()::text
FROM generate_series(1, 300000) AS gs;

-- =========================
-- chargebacks
-- =========================
INSERT INTO chargebacks (
    id,
    payment_id,
    external_case_id,
    external_reason_code,
    amount,
    currency,
    reason,
    status,
    won_flag,
    evidence_blob,
    customer_statement,
    internal_notes,
    provider_payload,
    created_at,
    updated_at,
    deadline_at,
    closed_at
)
SELECT
    ('cb-' || gs),
    'pay-' || ((random() * 2999999 + 1)::bigint),
    'case-' || gs,
    'RC-' || (gs % 100),
    (round((random() * 1500 + 10)::numeric, 2))::text,
    (ARRAY['USD','EUR','GBP','RSD'])[1 + (random() * 3)::int],
    CASE
        WHEN gs % 30 = 0 THEN 'fraud'
        WHEN gs % 45 = 0 THEN '<script>alert("cb")</script>'
        ELSE 'legacy reason ' || gs
    END,
    CASE
        WHEN gs % 3 = 0 THEN 'open'
        WHEN gs % 5 = 0 THEN 'lost'
        ELSE 'won'
    END,
    CASE WHEN gs % 5 = 0 THEN 'true' ELSE 'false' END,
    'base64-evidence-' || gs || '-with-card-4111111111111111',
    'customer statement ' || gs,
    'internal cb note ' || gs || ' password=admin123',
    '{"provider":"cb-' || gs || '"}',
    now()::text,
    now()::text,
    now()::text,
    CASE WHEN gs % 4 = 0 THEN now()::text ELSE NULL END
FROM generate_series(1, 120000) AS gs;

-- =========================
-- fraud reviews
-- =========================
INSERT INTO fraud_reviews (
    id,
    payment_id,
    customer_id,
    score,
    rules_triggered,
    decision,
    reviewer,
    reviewer_password,
    notes,
    raw_signals,
    ip_address,
    device_id,
    email_match_score,
    card_match_score,
    velocity_window_minutes,
    created_at,
    updated_at
)
SELECT
    ('fr-' || gs),
    'pay-' || ((random() * 2999999 + 1)::bigint),
    ((random() * 999999 + 1)::bigint)::text,
    (random() * 100)::int::text,
    'velocity,duplicate,ip-risk,<script>alert("fraud")</script>',
    CASE
        WHEN gs % 5 = 0 THEN 'hold'
        WHEN gs % 9 = 0 THEN 'reject'
        ELSE 'review'
    END,
    CASE WHEN gs % 20 = 0 THEN 'bot' ELSE 'risk-engine-v0' END,
    'reviewer-pass-' || gs,
    CASE
        WHEN gs % 70 = 0 THEN '''; DROP TABLE payments; --'
        ELSE 'fraud note ' || gs
    END,
    '{"device":"dev-' || gs || '","signal":"high"}',
    '10.10.' || (gs % 255) || '.' || ((gs / 255) % 255),
    'device-' || md5(('fr' || gs)::text),
    (random() * 100)::int::text,
    (random() * 100)::int::text,
    ((random() * 120)::int)::text,
    now()::text,
    now()::text
FROM generate_series(1, 250000) AS gs;

-- =========================
-- payment logs
-- =========================
INSERT INTO payment_logs (
    id,
    payment_id,
    customer_id,
    log_level,
    message,
    payload,
    request_headers,
    request_body,
    response_body,
    stack_trace,
    db_dsn,
    provider_secret,
    internal_key,
    actor_email,
    actor_password,
    source,
    created_at
)
SELECT
    ('log-' || gs),
    'pay-' || ((random() * 2999999 + 1)::bigint),
    ((random() * 999999 + 1)::bigint)::text,
    (ARRAY['INFO','WARN','ERROR','DEBUG'])[1 + (random() * 3)::int],
    CASE
        WHEN gs % 80 = 0 THEN 'XSS payload detected <script>alert("log")</script>'
        WHEN gs % 120 = 0 THEN 'possible SQL injection '' OR 1=1 --'
        ELSE 'legacy log message ' || gs
    END,
    '{"card":"4111111111111111","cvv":"123","token":"tok-' || gs || '"}',
    '{"Authorization":"Bearer plain-token-' || gs || '"}',
    '{"email":"user' || gs || '@example.com","password":"plain-pass-' || gs || '"}',
    '{"status":"ok","provider_secret":"secret-' || gs || '"}',
    'Stack trace line 1 ... line 2 ... payment=' || gs,
    'host=db port=5432 dbname=legacy_payments user=legacy_user password=legacy_password',
    'provider-secret-' || gs,
    'internal-key-' || gs,
    'admin' || gs || '@example.com',
    'admin-pass-' || gs,
    CASE WHEN gs % 2 = 0 THEN 'app' ELSE 'worker' END,
    now()::text
FROM generate_series(1, 4000000) AS gs;

ANALYZE;