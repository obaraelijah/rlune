use rlune_core::re_exports::axum::Json;
use rlune_core::session::Session;
use rlune_core::stuff::api_error::ApiError;
use rlune_core::stuff::api_error::ApiResult;
use rlune_core::Module;
use rlune_macros::delete;
use rlune_macros::put;

use crate::models::LocalAccount;
use crate::models::WebAuthnKey;
use crate::AuthModule;
use crate::MaybeAttestedPasskey;

type SetLocalPasswordRequest = String;

#[put("/local/password", core_crate = "::rlune_core")]
pub async fn set_local_password(
    session: Session,
    Json(request): Json<SetLocalPasswordRequest>,
) -> ApiResult<()> {
    let account_pk: i64 = session
        .get("account")
        .await?
        .ok_or(ApiError::bad_request("Not logged-in"))?;

    let mut tx = AuthModule::global().db.start_transaction().await?;

    let _local_pk = rorm::query(&mut tx, LocalAccount.pk)
        .condition(LocalAccount.account.equals(&account_pk))
        .optional()
        .await?
        .ok_or(ApiError::bad_request("User is not a local one"))?;

    // TODO: hashing

    rorm::update(&mut tx, LocalAccount)
        .set(LocalAccount.password, Some(request))
        .condition(LocalAccount.account.equals(&account_pk))
        .await?;

    tx.commit().await?;

    Ok(())
}

#[delete("/local/password", core_crate = "::rlune_core")]
pub async fn delete_local_password(session: Session) -> ApiResult<()> {
    let account_pk: i64 = session
        .get("account")
        .await?
        .ok_or(ApiError::bad_request("Not logged-in"))?;

    let mut tx = AuthModule::global().db.start_transaction().await?;

    let local_pk = rorm::query(&mut tx, LocalAccount.pk)
        .condition(LocalAccount.account.equals(&account_pk))
        .optional()
        .await?
        .ok_or(ApiError::bad_request("User is not a local one"))?;

    let has_webauthn = rorm::query(&mut tx, WebAuthnKey.key)
        .condition(WebAuthnKey.local_account.equals(&local_pk))
        .all()
        .await?
        .into_iter()
        .any(|key| matches!(key.0, MaybeAttestedPasskey::Attested(_)));
    if !has_webauthn {
        return Err(ApiError::bad_request("User has no other login method"));
    }

    rorm::update(&mut tx, LocalAccount)
        .set(LocalAccount.password, None)
        .condition(LocalAccount.account.equals(&account_pk))
        .await?;

    tx.commit().await?;
    Ok(())
}
