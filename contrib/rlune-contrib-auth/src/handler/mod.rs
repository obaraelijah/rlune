use rlune_core::re_exports::axum::extract::Query;
use rlune_core::re_exports::axum::Json;
use rlune_core::session::Session;
use rlune_core::stuff::api_error::ApiResult;
use rlune_core::Module;
use rlune_macros::get;
use rlune_macros::post;
use rorm::crud::query::QueryBuilder;
use rorm::internal::field::foreign_model::FieldEq_ForeignModelByField_Borrowed;
use rorm::FieldAccess;
use serde::Deserialize;
use serde::Serialize;
use webauthn_rs::prelude::AttestedPasskeyAuthentication;
use webauthn_rs::prelude::RequestChallengeResponse;

use crate::handler::schema::GetLoginFlowsRequest;
use crate::handler::schema::GetLoginFlowsResponse;
use crate::handler::schema::LocalLoginFlow;
use crate::handler::schema::LoginLocalPasswordRequest;
use crate::handler::schema::LoginLocalWebauthnRequest;
use crate::handler::schema::OidcLoginFlow;
use crate::handler::schema::PublicKeyCredential;
use crate::models::AuthModels;
use crate::module::AuthModule;
use crate::MaybeAttestedPasskey;

#[cfg(feature = "oidc")]
mod oidc;
#[cfg(feature = "oidc")]
pub use self::oidc::*;

mod local;
pub use self::local::*;
mod schema;

#[get("/login", core_crate = "::rlune_core")]
pub async fn get_login_flow<M: AuthModels>(
    Query(request): Query<GetLoginFlowsRequest>,
) -> ApiResult<Json<Option<GetLoginFlowsResponse>>> {
    let mut tx = AuthModule::<M>::global().db.start_transaction().await?;

    let Some((user_pk,)) = QueryBuilder::new(&mut tx, (M::account_pk(),))
        .condition(M::account_id().equals(request.identifier.as_str()))
        .optional()
        .await?
    else {
        return Ok(Json(None));
    };

    let oidc = QueryBuilder::new(&mut tx, (M::oidc_account_pk(),))
        .condition(M::oidc_account_fm().equals::<_, FieldEq_ForeignModelByField_Borrowed>(&user_pk))
        .optional()
        .await?;

    let local = QueryBuilder::new(
        &mut tx,
        (M::local_account_pk(), M::local_account_password()),
    )
    .condition(M::local_account_fm().equals::<_, FieldEq_ForeignModelByField_Borrowed>(&user_pk))
    .optional()
    .await?;

    let response = match (oidc, local) {
        (Some(_), None) => GetLoginFlowsResponse::Oidc(OidcLoginFlow {}),
        (None, Some((local_pk, password))) => {
            let webauthn = QueryBuilder::new(&mut tx, (M::webauthn_key_key(),))
                .condition(
                    M::webauthn_key_fm()
                        .equals::<_, FieldEq_ForeignModelByField_Borrowed>(&local_pk),
                )
                .all()
                .await?
                .into_iter()
                .any(|(key,)| matches!(key.0, MaybeAttestedPasskey::Attested(_)));

            GetLoginFlowsResponse::Local(LocalLoginFlow {
                password: password.is_some(),
                webauthn,
            })
        }
        _ => return Err("Invalid account".into()),
    };

    tx.commit().await?;
    Ok(Json(Some(response)))
}

#[post("/login/local/start-webauthn", core_crate = "::rlune_core")]
pub async fn login_local_webauthn<M: AuthModels>(
    session: Session,
    Json(request): Json<LoginLocalWebauthnRequest>,
) -> ApiResult<Json<RequestChallengeResponse>> {
    let mut tx = AuthModule::<M>::global().db.start_transaction().await?;

    let (account_pk,) = QueryBuilder::new(&mut tx, (M::account_pk(),))
        .condition(M::account_id().equals(&request.identifier))
        .optional()
        .await?
        .ok_or("Account not found")?;

    let (local_account_pk,) = QueryBuilder::new(&mut tx, (M::local_account_pk(),))
        .condition(
            M::local_account_fm().equals::<_, FieldEq_ForeignModelByField_Borrowed>(&account_pk),
        )
        .optional()
        .await?
        .ok_or("Not a local account")?;

    let keys = QueryBuilder::new(&mut tx, (M::webauthn_key_key(),))
        .condition(
            M::webauthn_key_fm()
                .equals::<_, FieldEq_ForeignModelByField_Borrowed>(&local_account_pk),
        )
        .all()
        .await?;
    let keys = keys
        .into_iter()
        .filter_map(|(json,)| match json.0 {
            MaybeAttestedPasskey::NotAttested(_) => None,
            MaybeAttestedPasskey::Attested(key) => Some(key),
        })
        .collect::<Vec<_>>();

    let (challenge, state) = AuthModule::<M>::global()
        .webauthn
        .start_attested_passkey_authentication(&keys)?;

    tx.commit().await?;

    session
        .insert(
            "login_local_webauthn",
            LoginLocalWebauthnSessionData {
                identifier: request.identifier,
                state,
            },
        )
        .await?;

    Ok(Json(challenge))
}

#[derive(Serialize, Deserialize)]
struct LoginLocalWebauthnSessionData {
    identifier: String,
    state: AttestedPasskeyAuthentication,
}

#[post("/login/local/finish-webauthn", core_crate = "::rlune_core")]
pub async fn finish_login_local_webauthn<M: AuthModels>(
    session: Session,
    Json(request): Json<PublicKeyCredential>,
) -> ApiResult<()> {
    let LoginLocalWebauthnSessionData { identifier, state } = session
        .remove("login_local_webauthn")
        .await?
        .ok_or("Bad Request")?;

    let authentication_result = AuthModule::<M>::global()
        .webauthn
        .finish_attested_passkey_authentication(&request.0, &state)?;

    let mut tx = AuthModule::<M>::global().db.start_transaction().await?;

    let (account_pk,) = QueryBuilder::new(&mut tx, (M::account_pk(),))
        .condition(M::account_id().equals(&identifier))
        .optional()
        .await?
        .ok_or("Account not found")?;

    let (local_account_pk,) = QueryBuilder::new(&mut tx, (M::local_account_pk(),))
        .condition(
            M::local_account_fm().equals::<_, FieldEq_ForeignModelByField_Borrowed>(&account_pk),
        )
        .optional()
        .await?
        .ok_or("Not a local account")?;

    let keys = QueryBuilder::new(&mut tx, (M::webauthn_key_key(),))
        .condition(
            M::webauthn_key_fm()
                .equals::<_, FieldEq_ForeignModelByField_Borrowed>(&local_account_pk),
        )
        .all()
        .await?;
    let _used_key = keys
        .into_iter()
        .find_map(|(json,)| match json.0 {
            MaybeAttestedPasskey::NotAttested(_) => None,
            MaybeAttestedPasskey::Attested(key) => {
                (key.cred_id() == authentication_result.cred_id()).then_some(key)
            }
        })
        .ok_or("Used unknown key")?;

    tx.commit().await?;

    session.insert("account", account_pk).await?;

    Ok(())
}

#[post("/login/local/password", core_crate = "::rlune_core")]
pub async fn login_local_password<M: AuthModels>(
    session: Session,
    Json(request): Json<LoginLocalPasswordRequest>,
) -> ApiResult<()> {
    let mut tx = AuthModule::<M>::global().db.start_transaction().await?;

    let (account_pk,) = QueryBuilder::new(&mut tx, (M::account_pk(),))
        .condition(M::account_id().equals(&request.identifier))
        .optional()
        .await?
        .ok_or("Account not found")?;

    let (local_account_password,) = QueryBuilder::new(&mut tx, (M::local_account_password(),))
        .condition(
            M::local_account_fm().equals::<_, FieldEq_ForeignModelByField_Borrowed>(&account_pk),
        )
        .optional()
        .await?
        .ok_or("Not a local account")?;

    let local_account_password = local_account_password.ok_or("Account has no password")?;
    // TODO: hashing
    if local_account_password != request.password {
        return Err("Passwords do not match".into());
    }

    // TODO: 2nd factor

    tx.commit().await?;

    session.insert("account", account_pk).await?;

    Ok(())
}

#[post("/logout", core_crate = "::rlune_core")]
pub async fn logout(session: Session) -> ApiResult<()> {
    session.remove::<serde::de::IgnoredAny>("account").await?;
    Ok(())
}
