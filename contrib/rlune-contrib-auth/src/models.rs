use rorm::fields::proxy;
use rorm::fields::proxy::FieldProxy;
use rorm::fields::traits::{Array, FieldType};
use rorm::fields::types::Json;
use rorm::internal::field::Field;
use rorm::model::GetField;
use rorm::prelude::ForeignModelByField;
use rorm::{Model, Patch};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::{AttestedPasskey, Passkey};

pub trait AuthModels: Send + Sync + 'static {
    type Account: Model<
            Primary: Field<
                Type: FieldType<Decoder: Send, Columns = Array<1>>
                        + Serialize
                        + DeserializeOwned
                        + Send
                        + Sync,
            >,
        > + GetField<<Self::Account as Model>::Primary>
        + Send
        + Sync;
    /// The account's identifier field
    ///
    /// The identifier is a string used by users to identify their accounts.
    /// It must be unique and suitable to be known to and remembered by a user.
    fn account_id() -> FieldProxy<(
        impl Field<Type = String, Model = Self::Account>,
        Self::Account,
    )>;
    /// The account's primary key field
    ///
    /// The primary key MAY be the same as the identifier.
    /// An application SHOULD use a different field.
    fn account_pk() -> FieldProxy<(<Self::Account as Model>::Primary, Self::Account)> {
        proxy::new()
    }
    fn insertable_account(id: String) -> impl Patch<Model = Self::Account> + Send + Sync;

    type OidcAccount: Model<Primary: Field<Type: FieldType<Decoder: Send, Columns = Array<1>> + Send + Sync>>
        + Send
        + Sync;
    fn oidc_account_pk() -> FieldProxy<(<Self::OidcAccount as Model>::Primary, Self::OidcAccount)> {
        proxy::new()
    }
    /// The foreign model field of `OidcAccount` pointing to `Account`
    fn oidc_account_fm() -> FieldProxy<(
        impl Field<
            Type = ForeignModelByField<<Self::Account as Model>::Primary>,
            Model = Self::OidcAccount,
        >,
        Self::OidcAccount,
    )>;
    fn oidc_account_id() -> FieldProxy<(
        impl Field<Type = String, Model = Self::OidcAccount>,
        Self::OidcAccount,
    )>;
    fn insertable_oidc_account(
        id: String,
        account_pk: &<<Self::Account as Model>::Primary as Field>::Type,
    ) -> impl Patch<Model = Self::OidcAccount> + Send + Sync;

    type LocalAccount: Model<Primary: Field<Type: FieldType<Decoder: Send, Columns = Array<1>> + Send + Sync>>
        + GetField<<Self::LocalAccount as Model>::Primary>
        + Send
        + Sync;
    fn local_account_pk() -> FieldProxy<(<Self::LocalAccount as Model>::Primary, Self::LocalAccount)>
    {
        proxy::new()
    }
    /// The foreign model field of `LocalAccount` pointing to `Account`
    fn local_account_fm() -> FieldProxy<(
        impl Field<
            Type = ForeignModelByField<<Self::Account as Model>::Primary>,
            Model = Self::LocalAccount,
        >,
        Self::LocalAccount,
    )>;
    fn local_account_password() -> FieldProxy<(
        impl Field<Type = Option<String>, Model = Self::LocalAccount>,
        Self::LocalAccount,
    )>;

    type TotpKey: Model;
    fn totp_key_pk() -> FieldProxy<(<Self::TotpKey as Model>::Primary, Self::TotpKey)> {
        proxy::new()
    }
    fn totp_key_fm() -> FieldProxy<(
        impl Field<
            Type = ForeignModelByField<<Self::LocalAccount as Model>::Primary>,
            Model = Self::TotpKey,
        >,
        Self::TotpKey,
    )>;

    type WebauthnKey: Model;
    fn webauthn_key_pk() -> FieldProxy<(<Self::WebauthnKey as Model>::Primary, Self::WebauthnKey)> {
        proxy::new()
    }
    fn webauthn_key_fm() -> FieldProxy<(
        impl Field<
            Type = ForeignModelByField<<Self::LocalAccount as Model>::Primary>,
            Model = Self::WebauthnKey,
        >,
        Self::WebauthnKey,
    )>;
    fn webauthn_key_key() -> FieldProxy<(
        impl Field<Type = Json<MaybeAttestedPasskey>, Model = Self::WebauthnKey>,
        Self::WebauthnKey,
    )>;
}

#[derive(Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum MaybeAttestedPasskey {
    NotAttested(Passkey),
    Attested(AttestedPasskey),
}