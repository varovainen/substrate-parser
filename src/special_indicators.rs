//! Special decoding triggers and indicators.
//!
//! Although the metadata has all sufficient information to decode the
//! data for a known type, some types must be treated specially for displaying
//! and/or further data handling.
//!
//! Additionally, some data should better be decoded directly as the custom type
//! mentioned in metadata descriptors, rather than decoded as more generalized
//! type and cast into custom type later on.
use external_memory_tools::{AddressableBuffer, ExternalMemory};
use scale_info::{form::PortableForm, Field, Path, Type, TypeDef, TypeDefPrimitive, Variant};

use crate::std::{borrow::ToOwned, string::String, vec::Vec};

use crate::cards::Info;
use crate::decoding_sci::{husk_type, pick_variant};
use crate::propagated::Checker;
use crate::traits::{AsMetadata, ResolveType, SignedExtensionMetadata};

/// [`Field`] `type_name` set indicating that the value *may* be
/// currency-related.
///
/// If the value is unsigned integer, it will be considered currency.
pub const BALANCE_ID_SET: &[&str] = &[
    "Balance",
    "BalanceOf<T>",
    "BalanceOf<T, I>",
    "DepositBalance",
    "ExtendedBalance",
    "PalletBalanceOf<T>",
    "T::Balance",
];

/// [`Field`] `type_name` set indicating that the value *may* be Ecdsa
/// signature.
///
/// If the value is array `[u8; 65]`, it will be considered Ecdsa signature.
pub const SIGNATURE_ECDSA_ID_SET: &[&str] = &["ecdsa::Signature"];

/// [`Field`] `type_name` set indicating that the value *may* be Ed25519
/// signature.
///
/// If the value is array `[u8; 64]`, it will be considered Ed25519 signature.
pub const SIGNATURE_ED25519_ID_SET: &[&str] = &["ed25519::Signature"];

/// [`Field`] `type_name` set indicating that the value *may* be Sr25519
/// signature.
///
/// If the value is array `[u8; 64]`, it will be considered Sr25519 signature.
pub const SIGNATURE_SR25519_ID_SET: &[&str] = &["sr25519::Signature"];

/// [`Field`] `name` set indicating the value *may* be nonce.
///
/// If the value is unsigned integer, it will be considered nonce.
pub const NONCE_ID_SET: &[&str] = &["nonce"];

/// [`Field`] `name` set indicating the value *may* be metadata `spec_version`.
///
/// If the value is unsigned integer, it will be considered `spec_version`.
pub const SPEC_VERSION_ID_SET: &[&str] = &["spec_version"];

/// [`Field`] `name` set indicating the value *may* be metadata `spec_name`.
///
/// If the value is `str`, it will be considered `spec_name`.
pub const SPEC_NAME_ID_SET: &[&str] = &["spec_name"];

/// [`Field`] `name` set indicating the value *may* be metadata
/// `transaction_version`.
///
/// If the value is unsigned integer, it will be considered `transaction_version`.
pub const TX_VERSION_ID_SET: &[&str] = &["tx_version", "transaction_version"];

/// [`Type`]-associated [`Path`] `ident` for
/// [`sp_core::crypto::AccountId32`](https://docs.rs/sp-core/latest/sp_core/crypto/struct.AccountId32.html).
pub const ACCOUNT_ID32: &str = "AccountId32";

/// [`Type`]-associated [`Path`] `ident` indicating that the data to follow
/// *may* be a call.
pub const CALL: &[&str] = &["Call", "RuntimeCall"];

/// [`Type`]-associated [`Path`] `ident` for
/// [`sp_runtime::generic::Era`](https://docs.rs/sp-runtime/latest/sp_runtime/generic/enum.Era.html).
pub const ERA: &str = "Era";

/// [`Type`]-associated [`Path`] `ident` indicating that the data to follow
/// *may* be an event.
pub const EVENT: &[&str] = &["Event", "RuntimeEvent"];

/// [`Type`]-associated [`Path`] `ident` set for [primitive_types::H160].
pub const H160: &[&str] = &["AccountId20", "H160"];

/// [`Type`]-associated [`Path`] `ident` for [primitive_types::H256].
pub const H256: &str = "H256";

/// [`Type`]-associated [`Path`] `ident` for [primitive_types::H512].
pub const H512: &str = "H512";

/// [`Type`]-associated [`Path`] `ident` for `sp_runtime::MultiSignature`.
pub const MULTI_SIGNATURE: &str = "MultiSignature";

/// [`Type`]-associated [`Path`] `ident` for [sp_arithmetic::Perbill].
pub const PERBILL: &str = "Perbill";

/// [`Type`]-associated [`Path`] `ident` for [sp_arithmetic::Percent].
pub const PERCENT: &str = "Percent";

/// [`Type`]-associated [`Path`] `ident` for [sp_arithmetic::Permill].
pub const PERMILL: &str = "Permill";

/// [`Type`]-associated [`Path`] `ident` for [sp_arithmetic::Perquintill].
pub const PERQUINTILL: &str = "Perquintill";

/// [`Type`]-associated [`Path`] `ident` for [sp_arithmetic::PerU16].
pub const PERU16: &str = "PerU16";

/// [`Type`]-associated [`Path`] `ident` for possible public key.
pub const PUBLIC: &str = "Public";

/// [`Type`]-associated [`Path`] `ident` for possible signature.
pub const SIGNATURE: &str = "Signature";

/// [`Path`] `namespace` for `sp_core::ed25519`.
pub const SP_CORE_ED25519: &[&str] = &["sp_core", "ed25519"];

/// [`Path`] `namespace` for `sp_core::sr25519`.
pub const SP_CORE_SR25519: &[&str] = &["sp_core", "sr25519"];

/// [`Path`] `namespace` for `sp_core::ecdsa`.
pub const SP_CORE_ECDSA: &[&str] = &["sp_core", "ecdsa"];

/// [`Path`] `namespace` for `sp_runtime::generic::UncheckedExtrinsic`.
pub const UNCHECKED_EXTRINSIC_NAMESPACE: &[&str] =
    &["sp_runtime", "generic", "unchecked_extrinsic"];

/// [`Type`]-associated [`Path`] `ident` for
/// `sp_runtime::generic::UncheckedExtrinsic`.
pub const UNCHECKED_EXTRINSIC_IDENT: &str = "UncheckedExtrinsic";

/// Extensions `identifier` from [`SignedExtensionMetadata`] for metadata spec
/// version.
///
/// If underlying value is unsigned integer, it will be considered
/// `spec_version.`
///
/// Apparently established `identifier` across different chains.
pub const CHECK_SPEC_VERSION: &str = "CheckSpecVersion";

/// Extensions `identifier` from [`SignedExtensionMetadata`] for `tx_version`.
///
/// If underlying value is unsigned integer, it will be considered `tx_version`.
///
/// Apparently established `identifier` across different chains.
pub const CHECK_TX_VERSION: &str = "CheckTxVersion";

/// Extensions `identifier` from [`SignedExtensionMetadata`] for chain genesis
/// hash.
///
/// If underlying value is `H256`, it will be considered genesis hash.
///
/// Apparently established `identifier` across different chains.
pub const CHECK_GENESIS: &str = "CheckGenesis";

/// Extensions `identifier` from [`SignedExtensionMetadata`] for block hash.
///
/// If underlying value is `H256`, it will be considered block hash.
///
/// Same identifier accompanies `Era` in extensions, but `Era` gets detected by
/// matching [`Path`] of the corresponding [`Type`] with [`ERA`].
///
/// Apparently established `identifier` across different chains.
pub const CHECK_MORTALITY: &str = "CheckMortality";

/// Extensions `identifier` from [`SignedExtensionMetadata`] for nonce.
///
/// If underlying value is unsigned integer, it will be considered nonce.
///
/// Apparently established `identifier` across different chains.
pub const CHECK_NONCE: &str = "CheckNonce";

/// Extensions `identifier` from [`SignedExtensionMetadata`] for transaction
/// tip.
///
/// If underlying value is unsigned integer, it will be considered tip.
///
/// Note: signable transaction tip always gets carded as balance with chain
/// units and decimals.
///
/// Apparently established `identifier` across different chains.
pub const CHARGE_TRANSACTION_PAYMENT: &str = "ChargeTransactionPayment";

/// Extensions `identifier` from [`SignedExtensionMetadata`] for transaction
/// tip that may or may not be in asset units.
///
/// If underlying value has unsigned integer, it will be considered an asset
/// tip.
///
/// Note: asset tip is always gets carded as raw.
///
/// Apparently established `identifier` across different chains.
pub const CHARGE_ASSET_TX_PAYMENT: &str = "ChargeAssetTxPayment";

/// Encoded length of an enum variant index.
pub const ENUM_INDEX_ENCODED_LEN: usize = 1;

/// Specialty attributed to unsigned integer.
///
/// `SpecialtyUnsignedInteger` is stored in unsigned integer `ParsedData` and
/// determines the card type.
///
/// Is determined by propagating [`Hint`] from [`SignedExtensionMetadata`]
/// identifier or from [`Field`] descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpecialtyUnsignedInteger {
    /// Regular unsigned integer.
    None,

    /// Value is currency-related, displayed with chain decimals and units for
    /// appropriate [pallets](crate::cards::PALLETS_BALANCE_VALID).
    Balance,

    /// Value is transaction tip from signable transaction extensions, always
    /// displayed as currency with chain decimals and units.
    Tip,

    /// Value is transaction tip from signable transaction extensions, possibly
    /// with asset units and decimals. Displayed as raw.
    TipAsset,

    /// Value is nonce.
    Nonce,

    /// Value is metadata `spec_version` from signable transaction extensions.
    SpecVersion,

    /// Value is `tx_version` from signable transaction extensions.
    TxVersion,
}

/// Specialty attributed to `str` data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpecialtyStr {
    None,
    SpecName,
}

/// Specialty attributed to `H256` hashes.
///
/// Is used only when parsing signable transaction extensions.
///
/// Is determined by propagating [`Hint`] from [`SignedExtensionMetadata`].
#[derive(Clone, Copy, Debug)]
pub enum SpecialtyH256 {
    None,
    GenesisHash,
    BlockHash,
}

/// Indicator that the field would be considered a signature. Non-propagating.
#[derive(Clone, Copy, Debug)]
pub enum SignatureIndicator {
    None,
    Ecdsa,
    Ed25519,
    Sr25519,
}

impl SignatureIndicator {
    /// `SignatureIndicator` for a [`Field`]. Uses `type_name`, checks signature
    /// type to be an `u8` array of the known length.
    pub fn from_field<E, M>(
        field: &Field<PortableForm>,
        ext_memory: &mut E,
        registry: &M::TypeRegistry,
    ) -> Self
    where
        E: ExternalMemory,
        M: AsMetadata<E>,
    {
        if let Ok(husked_field_ty) =
            husk_type::<E, M>(&field.ty, registry, ext_memory, Checker::new())
        {
            // check here that the underlying type is really `[u8; LEN]` array
            let array_u8_length = match husked_field_ty.ty.type_def {
                TypeDef::Array(signature_array_ty) => {
                    let element_ty_id = signature_array_ty.type_param.id;
                    if let Ok(element_ty) = registry.resolve_ty(element_ty_id, ext_memory) {
                        if let TypeDef::Primitive(TypeDefPrimitive::U8) = element_ty.type_def {
                            signature_array_ty.len
                        } else {
                            return Self::None;
                        }
                    } else {
                        return Self::None;
                    }
                }
                _ => return Self::None,
            };
            match &field.type_name {
                Some(type_name) => match type_name.as_str() {
                    a if SIGNATURE_ECDSA_ID_SET.contains(&a) => {
                        if array_u8_length == substrate_crypto_light::ecdsa::SIGNATURE_LEN as u32 {
                            Self::Ecdsa
                        } else {
                            Self::None
                        }
                    }
                    a if SIGNATURE_ED25519_ID_SET.contains(&a) => {
                        if array_u8_length == substrate_crypto_light::ed25519::SIGNATURE_LEN as u32
                        {
                            Self::Ed25519
                        } else {
                            Self::None
                        }
                    }
                    a if SIGNATURE_SR25519_ID_SET.contains(&a) => {
                        if array_u8_length == substrate_crypto_light::sr25519::SIGNATURE_LEN as u32
                        {
                            Self::Sr25519
                        } else {
                            Self::None
                        }
                    }
                    _ => Self::None,
                },
                None => Self::None,
            }
        } else {
            Self::None
        }
    }
}

/// Nonbinding, propagating type specialty indicator.
///
/// Propagates during the decoding into compacts and single-field structs and
/// gets used only if suitable type is encountered.
///
/// `Hint` can originate from [`SignedExtensionMetadata`] identifier or from
/// [`Field`] descriptor.
///
/// If non-`None` `Hint` is encountered during decoding, it does not get updated
/// until the extension or the field are decoded through.
#[derive(Clone, Copy, Debug)]
pub enum Hint {
    None,
    CheckSpecVersion,
    CheckTxVersion,
    CheckGenesis,
    CheckMortality,
    CheckNonce,
    ChargeAssetTxPayment,
    ChargeTransactionPayment,
    FieldBalance,
    FieldNonce,
    FieldSpecName,
    FieldSpecVersion,
    FieldTxVersion,
}

impl Hint {
    /// `Hint` for a [`Field`]. Both `name` and `type_name` are used, `name` is
    /// more reliable and gets checked first.
    pub fn from_field(field: &Field<PortableForm>) -> Self {
        let mut out = match &field.name {
            Some(name) => match name.as_str() {
                a if NONCE_ID_SET.contains(&a) => Self::FieldNonce,
                a if SPEC_VERSION_ID_SET.contains(&a) => Self::FieldSpecVersion,
                a if SPEC_NAME_ID_SET.contains(&a) => Self::FieldSpecName,
                a if TX_VERSION_ID_SET.contains(&a) => Self::FieldTxVersion,
                _ => Self::None,
            },
            None => Self::None,
        };
        if let Self::None = out {
            if let Some(type_name) = &field.type_name {
                out = match type_name.as_str() {
                    a if BALANCE_ID_SET.contains(&a) => Self::FieldBalance,
                    _ => Self::None,
                };
            }
        }
        out
    }

    /// `Hint` for signed extensions instance.
    pub fn from_ext_meta(signed_ext_meta: &SignedExtensionMetadata) -> Self {
        match signed_ext_meta.identifier.as_str() {
            CHECK_SPEC_VERSION => Self::CheckSpecVersion,
            CHECK_TX_VERSION => Self::CheckTxVersion,
            CHECK_GENESIS => Self::CheckGenesis,
            CHECK_MORTALITY => Self::CheckMortality,
            CHECK_NONCE => Self::CheckNonce,
            CHARGE_ASSET_TX_PAYMENT => Self::ChargeAssetTxPayment,
            CHARGE_TRANSACTION_PAYMENT => Self::ChargeTransactionPayment,
            _ => Self::None,
        }
    }

    /// `Hint` from [`Path`].
    ///
    /// Can appear only when decoding tuples, as each tuple field has a [`Type`]
    /// with possibly specified `Path`.
    pub fn from_path(path: &Path<PortableForm>) -> Self {
        match path.ident() {
            Some(a) => match a.as_str() {
                CHECK_NONCE => Self::CheckNonce,
                CHARGE_ASSET_TX_PAYMENT => Self::ChargeAssetTxPayment,
                CHARGE_TRANSACTION_PAYMENT => Self::ChargeTransactionPayment,
                _ => Self::None,
            },
            None => Self::None,
        }
    }

    /// Apply [`Hint`] on unsigned integer decoding.
    pub fn unsigned_integer(&self) -> SpecialtyUnsignedInteger {
        match &self {
            Hint::CheckSpecVersion | Hint::FieldSpecVersion => {
                SpecialtyUnsignedInteger::SpecVersion
            }
            Hint::CheckTxVersion | Hint::FieldTxVersion => SpecialtyUnsignedInteger::TxVersion,
            Hint::CheckNonce | Hint::FieldNonce => SpecialtyUnsignedInteger::Nonce,
            Hint::ChargeTransactionPayment => SpecialtyUnsignedInteger::Tip,
            Hint::ChargeAssetTxPayment => SpecialtyUnsignedInteger::TipAsset,
            Hint::FieldBalance => SpecialtyUnsignedInteger::Balance,
            _ => SpecialtyUnsignedInteger::None,
        }
    }

    /// Apply [`Hint`] on `str` decoding.
    pub fn string(&self) -> SpecialtyStr {
        match &self {
            Hint::FieldSpecName => SpecialtyStr::SpecName,
            _ => SpecialtyStr::None,
        }
    }

    /// Apply [`Hint`] on `H256` decoding.
    pub fn hash256(&self) -> SpecialtyH256 {
        match &self {
            Hint::CheckGenesis => SpecialtyH256::GenesisHash,
            Hint::CheckMortality => SpecialtyH256::BlockHash,
            _ => SpecialtyH256::None,
        }
    }
}

/// Unconfirmed specialty for a [`Type`].
///
/// Does not propagate.
///
/// Type internal structure must be additionally confirmed for `Call` and
/// `Event` before transforming into [`SpecialtyTypeChecked`], the type
/// specialty that causes parser action.
pub enum SpecialtyTypeHinted {
    None,
    AccountId32,
    Era,
    H160,
    H256,
    H512,
    MultiSignature,
    PalletSpecific(PalletSpecificItem),
    Perbill,
    Percent,
    Permill,
    Perquintill,
    PerU16,
    PublicEd25519,
    PublicSr25519,
    PublicEcdsa,
    SignatureEd25519,
    SignatureSr25519,
    SignatureEcdsa,
    UncheckedExtrinsic,
}

/// Specialty types that are associated with particular pallet. Currently `Call`
/// and `Enum`.
///
/// Identifier `Call` and `Enum` are encoutered in both hierarchically first and
/// second enums that descibe the pallet and the call/event name
/// correspondingly.
#[derive(Debug, Eq, PartialEq)]
pub enum PalletSpecificItem {
    Call,
    Event,
}

impl SpecialtyTypeHinted {
    /// Get `SpecialtyTypeHinted` from type-associated [`Path`].
    pub fn from_type(ty: &Type<PortableForm>) -> Self {
        match ty.path.ident() {
            Some(a) => match a.as_str() {
                ACCOUNT_ID32 => Self::AccountId32,
                a if CALL.contains(&a) => Self::PalletSpecific(PalletSpecificItem::Call),
                ERA => Self::Era,
                a if EVENT.contains(&a) => Self::PalletSpecific(PalletSpecificItem::Event),
                a if H160.contains(&a) => Self::H160,
                H256 => Self::H256,
                H512 => Self::H512,
                MULTI_SIGNATURE => Self::MultiSignature,
                PERBILL => Self::Perbill,
                PERCENT => Self::Percent,
                PERMILL => Self::Permill,
                PERQUINTILL => Self::Perquintill,
                PERU16 => Self::PerU16,
                PUBLIC => match ty
                    .path
                    .namespace()
                    .iter()
                    .map(|x| x.as_str())
                    .collect::<Vec<&str>>()
                    .as_ref()
                {
                    SP_CORE_ED25519 => Self::PublicEd25519,
                    SP_CORE_SR25519 => Self::PublicSr25519,
                    SP_CORE_ECDSA => Self::PublicEcdsa,
                    _ => Self::None,
                },
                SIGNATURE => match ty
                    .path
                    .namespace()
                    .iter()
                    .map(|x| x.as_str())
                    .collect::<Vec<&str>>()
                    .as_ref()
                {
                    SP_CORE_ED25519 => Self::SignatureEd25519,
                    SP_CORE_SR25519 => Self::SignatureSr25519,
                    SP_CORE_ECDSA => Self::SignatureEcdsa,
                    _ => Self::None,
                },
                UNCHECKED_EXTRINSIC_IDENT => match ty
                    .path
                    .namespace()
                    .iter()
                    .map(|x| x.as_str())
                    .collect::<Vec<&str>>()
                    .as_ref()
                {
                    UNCHECKED_EXTRINSIC_NAMESPACE => Self::UncheckedExtrinsic,
                    _ => Self::None,
                },
                _ => Self::None,
            },
            None => Self::None,
        }
    }
}

/// [`Type`] specialty, based on [`Path`] and type internal structure.
///
/// Does not propagate. If found, causes parser to decode through special route.
/// If decoding through special route fails, it is considered parser error.
pub enum SpecialtyTypeChecked {
    None,
    AccountId32,
    Era,
    H160,
    H256,
    H512,
    PalletSpecific {
        pallet_name: String,
        pallet_info: Info,
        pallet_variant: Variant<PortableForm>,
        item_ty_id: u32,
        variants: Vec<Variant<PortableForm>>,
        item: PalletSpecificItem,
    },
    Perbill,
    Percent,
    Permill,
    Perquintill,
    PerU16,
    PublicEd25519,
    PublicSr25519,
    PublicEcdsa,
    SignatureEd25519,
    SignatureSr25519,
    SignatureEcdsa,
}

impl SpecialtyTypeChecked {
    /// Get `SpecialtyTypeChecked` for a [`Type`].
    ///
    /// Checks type internal structure and uses input data for
    /// [`PalletSpecificItem`].
    pub fn from_type<B, E, M>(
        ty: &Type<PortableForm>,
        data: &B,
        ext_memory: &mut E,
        position: &mut usize,
        registry: &M::TypeRegistry,
    ) -> Self
    where
        B: AddressableBuffer<E>,
        E: ExternalMemory,
        M: AsMetadata<E>,
    {
        match SpecialtyTypeHinted::from_type(ty) {
            SpecialtyTypeHinted::None => Self::None,
            SpecialtyTypeHinted::AccountId32 => Self::AccountId32,
            SpecialtyTypeHinted::Era => Self::Era,
            SpecialtyTypeHinted::H160 => Self::H160,
            SpecialtyTypeHinted::H256 => Self::H256,
            SpecialtyTypeHinted::H512 => Self::H512,
            SpecialtyTypeHinted::MultiSignature => {
                if let TypeDef::Variant(x) = &ty.type_def {
                    match pick_variant::<B, E>(&x.variants, data, ext_memory, *position) {
                        Ok(signature_variant) => {
                            if signature_variant.fields.len() == 1 {
                                match SignatureIndicator::from_field::<E, M>(
                                    &signature_variant.fields[0],
                                    ext_memory,
                                    registry,
                                ) {
                                    SignatureIndicator::None => Self::None,
                                    SignatureIndicator::Ecdsa => {
                                        *position += ENUM_INDEX_ENCODED_LEN;
                                        Self::SignatureEcdsa
                                    }
                                    SignatureIndicator::Ed25519 => {
                                        *position += ENUM_INDEX_ENCODED_LEN;
                                        Self::SignatureEd25519
                                    }
                                    SignatureIndicator::Sr25519 => {
                                        *position += ENUM_INDEX_ENCODED_LEN;
                                        Self::SignatureSr25519
                                    }
                                }
                            } else {
                                Self::None
                            }
                        }
                        Err(_) => Self::None,
                    }
                } else {
                    Self::None
                }
            }
            SpecialtyTypeHinted::PalletSpecific(item) => {
                if let TypeDef::Variant(x) = &ty.type_def {
                    // found specific variant corresponding to pallet,
                    // get pallet name from here
                    match pick_variant::<B, E>(&x.variants, data, ext_memory, *position) {
                        Ok(pallet_variant) => {
                            let pallet_name = pallet_variant.name.to_owned();
                            if pallet_variant.fields.len() == 1 {
                                let item_ty_id = pallet_variant.fields[0].ty.id;
                                match registry.resolve_ty(item_ty_id, ext_memory) {
                                    Ok(variants_ty) => {
                                        if let SpecialtyTypeHinted::PalletSpecific(item_repeated) =
                                            SpecialtyTypeHinted::from_type(&variants_ty)
                                        {
                                            if item != item_repeated {
                                                Self::None
                                            } else if let TypeDef::Variant(ref var) =
                                                variants_ty.type_def
                                            {
                                                let pallet_info = Info::from_ty(&variants_ty);
                                                *position += ENUM_INDEX_ENCODED_LEN;
                                                Self::PalletSpecific {
                                                    pallet_name,
                                                    pallet_info,
                                                    pallet_variant: pallet_variant.to_owned(),
                                                    item_ty_id,
                                                    variants: var.variants.to_vec(),
                                                    item,
                                                }
                                            } else {
                                                Self::None
                                            }
                                        } else {
                                            Self::None
                                        }
                                    }
                                    Err(_) => Self::None,
                                }
                            } else {
                                Self::None
                            }
                        }
                        Err(_) => Self::None,
                    }
                } else {
                    Self::None
                }
            }
            SpecialtyTypeHinted::Perbill => Self::Perbill,
            SpecialtyTypeHinted::Percent => Self::Percent,
            SpecialtyTypeHinted::Permill => Self::Permill,
            SpecialtyTypeHinted::Perquintill => Self::Perquintill,
            SpecialtyTypeHinted::PerU16 => Self::PerU16,
            SpecialtyTypeHinted::PublicEd25519 => Self::PublicEd25519,
            SpecialtyTypeHinted::PublicSr25519 => Self::PublicSr25519,
            SpecialtyTypeHinted::PublicEcdsa => Self::PublicEcdsa,
            SpecialtyTypeHinted::SignatureEd25519 => Self::SignatureEd25519,
            SpecialtyTypeHinted::SignatureSr25519 => Self::SignatureSr25519,
            SpecialtyTypeHinted::SignatureEcdsa => Self::SignatureEcdsa,
            SpecialtyTypeHinted::UncheckedExtrinsic => Self::None,
        }
    }
}
