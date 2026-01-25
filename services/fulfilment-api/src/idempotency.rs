use axum::http::{HeaderMap, StatusCode};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Transaction};
use std::future::Future;
use std::pin::Pin;

use shipyard_web::{ApiError, RequestId};

const IDEMPOTENCY_HEADER: &str = "Idempotency-Key";

#[derive(sqlx::FromRow)]
struct IdempotencyRow {
    request_hash: String,
    status: String,
    response_status: Option<i32>,
    response_body: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    InProgress,
    Completed,
}

impl Status {
    fn from_db(s: &str) -> Self {
        match s {
            "COMPLETED" => Status::Completed,
            _ => Status::InProgress,
        }
    }
}

pub async fn with_idempotency<TReq, TResp, F>(
    db: &PgPool,
    headers: &HeaderMap,
    req_id: &RequestId,
    endpoint: &'static str,
    req: &TReq,
    op: F,
) -> Result<(StatusCode, TResp), ApiError>
where
    TReq: Serialize + ?Sized,
    TResp: Serialize + DeserializeOwned,
    F: for<'t> FnOnce(
        &'t mut Transaction<'_, Postgres>,
    ) -> Pin<
        Box<dyn Future<Output = Result<(StatusCode, TResp), ApiError>> + Send + 't>,
    >,
{
    let key = match extract_key(headers) {
        None => {
            // No idempotency: run normally (thin default).
            let mut tx = db.begin().await.map_err(|_| ApiError::internal(req_id))?;
            let (status, resp) = op(&mut tx).await?;
            tx.commit().await.map_err(|_| ApiError::internal(req_id))?;
            return Ok((status, resp));
        }
        Some(k) => k,
    };

    let request_hash = hash_request(req).map_err(|_| ApiError::internal(req_id))?;
    let mut tx = db.begin().await.map_err(|_| ApiError::internal(req_id))?;

    // Try to claim the key.
    let inserted = sqlx::query(
        r#"
        INSERT INTO idempotency_keys (endpoint, idempotency_key, request_hash, status)
        VALUES ($1, $2, $3, 'IN_PROGRESS')
        ON CONFLICT (endpoint, idempotency_key) DO NOTHING
        "#,
    )
    .bind(endpoint)
    .bind(key)
    .bind(&request_hash)
    .execute(&mut *tx)
    .await
    // .map_err(|_| ApiError::internal(req_id))?
    .map_err(|e| {
        tracing::error!(error = %e, "idempotency insert failed");
        ApiError::internal(req_id)
    })?
    .rows_affected()
        == 1;

    if inserted {
        // We own the key: execute operation and store result.
        let (status, resp) = op(&mut tx).await?;

        let body = serde_json::to_value(&resp).map_err(|_| ApiError::internal(req_id))?;
        let status_i32 = status.as_u16() as i32;

        sqlx::query(
            r#"
            UPDATE idempotency_keys
            SET status = 'COMPLETED',
                response_status = $3,
                response_body = $4,
                updated_at = now()
            WHERE endpoint = $1 AND idempotency_key = $2
            "#,
        )
        .bind(endpoint)
        .bind(key)
        .bind(status_i32)
        .bind(body)
        .execute(&mut *tx)
        .await
        // .map_err(|_| ApiError::internal(req_id))?;
        .map_err(|e| {
            tracing::error!(error = %e, "idempotency insert failed");
            ApiError::internal(req_id)
        })?;

        tx.commit().await.map_err(|_| ApiError::internal(req_id))?;
        return Ok((status, resp));
    }

    // Key exists: load record.
    let rec: IdempotencyRow = sqlx::query_as(
        r#"
        SELECT request_hash, status, response_status, response_body
        FROM idempotency_keys
        WHERE endpoint = $1 AND idempotency_key = $2
        "#,
    )
    .bind(endpoint)
    .bind(key)
    .fetch_one(&mut *tx)
    .await
    // .map_err(|_| ApiError::internal(req_id))?;
    .map_err(|e| {
        tracing::error!(error = %e, "idempotency insert failed");
        ApiError::internal(req_id)
    })?;

    if rec.request_hash != request_hash {
        tx.rollback().await.ok();
        return Err(ApiError::conflict(
            req_id,
            "Idempotency-Key reuse with different request payload",
        ));
    }

    match Status::from_db(&rec.status) {
        Status::InProgress => {
            tx.rollback().await.ok();
            Err(ApiError::conflict(
                req_id,
                "Request in progress for this Idempotency-Key; retry shortly",
            ))
        }
        Status::Completed => {
            let code = rec.response_status.unwrap_or(200);
            let body = rec.response_body.unwrap_or(Value::Null);

            let resp: TResp =
                serde_json::from_value(body).map_err(|_| ApiError::internal(req_id))?;

            let status_code = StatusCode::from_u16(code as u16).unwrap_or(StatusCode::OK);

            tx.rollback().await.ok();
            Ok((status_code, resp))
        }
    }
}

fn extract_key(headers: &HeaderMap) -> Option<&str> {
    let raw = headers.get(IDEMPOTENCY_HEADER)?;
    let s = raw.to_str().ok()?.trim();
    if s.is_empty() { None } else { Some(s) }
}

fn hash_request<T: Serialize + ?Sized>(req: &T) -> Result<String, serde_json::Error> {
    let bytes = serde_json::to_vec(req)?;
    let mut h = Sha256::new();
    h.update(bytes);
    Ok(hex::encode(h.finalize()))
}
