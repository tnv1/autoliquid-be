use std::str::FromStr;

use async_trait::async_trait;
use fastcrypto::hash::HashFunction;
use shared_crypto::intent::{Intent, IntentMessage};
use sui_sdk::{
    SuiClient, SuiClientBuilder,
    rpc_types::{SuiObjectDataOptions, SuiObjectResponse, SuiTransactionBlockResponseOptions},
};
use sui_types::{
    Identifier, SUI_CLOCK_OBJECT_ID, SUI_CLOCK_OBJECT_SHARED_VERSION, TypeTag,
    base_types::{ObjectID, ObjectRef, SuiAddress},
    crypto::{EncodeDecodeBase64, Signer, SuiKeyPair, SuiSignature},
    object::OBJECT_START_VERSION,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    quorum_driver_types::ExecuteTransactionRequestType,
    signature::GenericSignature,
    transaction::{Command, ObjectArg, Transaction, TransactionData},
};

#[derive(Debug)]
pub struct AddLiquidityOptions {
    pub pool_id: String,
    pub position_id: String,
    pub coin_a: String,
    pub coin_b: String,
    pub amount: u64,
    pub coin_a_max: u64,
    pub coin_b_max: u64,
    pub is_fixed_a: bool,
}

#[derive(Debug)]
pub struct RemoveLiquidityOptions {
    pub pool_id: String,
    pub position_id: String,
    pub coin_a_amount: u64,
    pub coin_b_amount: u64,
    pub tick_lower: i32,
    pub tick_upper: i32,
}

#[derive(Debug)]
pub struct ClosePositionOptions {
    pub pool_id: String,
    pub position_id: String,
}

#[derive(Debug)]
pub struct OpenPositionOptions {
    pub pool_id: String,
    pub coin_a: String,
    pub coin_b: String,
    pub lower_tick_bits: u32,
    pub upper_tick_bits: u32,
}

#[derive(Debug)]
pub struct RepositionOptions {
    pub pool_id: String,
    pub position_id: String,
}

// Define the interface for the dex
#[async_trait]
pub trait DexInterface: Sync + Send {
    // Add liquidity to the pool
    async fn provide_liquidity(
        &self,
        signer: &SuiKeyPair,
        options: AddLiquidityOptions,
    ) -> anyhow::Result<()>;

    // Remove liquidity from the pool
    async fn remove_liquidity(
        &self,
        signer: &SuiKeyPair,
        options: RemoveLiquidityOptions,
    ) -> anyhow::Result<()>;

    // Close position
    async fn close_position(
        &self,
        signer: &SuiKeyPair,
        options: ClosePositionOptions,
    ) -> anyhow::Result<()>;

    // Open position
    async fn open_position(
        &self,
        signer: &SuiKeyPair,
        options: OpenPositionOptions,
    ) -> anyhow::Result<()>;

    async fn reposition(
        &self,
        signer: &SuiKeyPair,
        options: RepositionOptions,
    ) -> anyhow::Result<()> {
        tracing::info!(
            "Repositioning position: {:?}, signer: {:?}",
            options,
            signer.public().encode_base64()
        );

        Ok(())
    }
}

pub struct BluefinDex {
    pub package_id: String,
    pub global_config: String,
    pub sui_client: SuiClient,
}

impl BluefinDex {
    pub async fn new(rpc_url: String, package_id: String, global_config: String) -> Self {
        let sui_client = SuiClientBuilder::default().build(rpc_url).await.unwrap();
        Self { sui_client, package_id, global_config }
    }
}

#[async_trait]
impl DexInterface for BluefinDex {
    async fn provide_liquidity(
        &self,
        keypair: &SuiKeyPair,
        options: AddLiquidityOptions,
    ) -> anyhow::Result<()> {
        tracing::info!(
            "Adding liquidity to pool: {:?}, signer: {:?}",
            options,
            keypair.public().encode_base64()
        );
        let sender = SuiAddress::from(&keypair.public());
        let package =
            ObjectID::from_hex_literal(&self.package_id).map_err(|e| anyhow::anyhow!(e))?;
        let module = Identifier::new("gateway").map_err(|e| anyhow::anyhow!(e))?;
        let function = Identifier::new("provide_liquidity").map_err(|e| anyhow::anyhow!(e))?;

        let type_args = vec![
            TypeTag::from_str(options.coin_a.as_ref())?,
            TypeTag::from_str(options.coin_b.as_ref())?,
        ];

        let mut ptb = ProgrammableTransactionBuilder::new();

        let clock_arg = ptb.obj(clock_obj())?;
        let config_arg = ptb.obj(shared_obj_mut(&self.sui_client, &self.global_config).await?)?;
        let pool_arg = ptb.obj(shared_obj_mut(&self.sui_client, &options.pool_id).await?)?;
        let position_arg =
            ptb.obj(shared_obj_mut(&self.sui_client, &options.position_id).await?)?;

        let coin_a_arg = ptb.obj(owned_obj(&self.sui_client, &options.coin_a).await?)?;
        let coin_b_arg = ptb.obj(owned_obj(&self.sui_client, &options.coin_a).await?)?;

        let amount_arg = ptb.pure(options.amount)?;
        let coin_a_max_arg = ptb.pure(options.coin_a_max)?;
        let coin_b_max_arg = ptb.pure(options.coin_b_max)?;
        let is_fixed_a_arg = ptb.pure(options.is_fixed_a)?;

        // public entry fun provide_liquidity_with_fixed_amount<CoinTypeA, CoinTypeB>(
        //     clock: &Clock,
        //     protocol_config: &GlobalConfig,
        //     pool: &mut Pool<CoinTypeA, CoinTypeB>,
        //     position: &mut Position,
        //     coin_a: Coin<CoinTypeA>,
        //     coin_b: Coin<CoinTypeB>,
        //     amount: u64,
        //     coin_a_max: u64,
        //     coin_b_max: u64,
        //     is_fixed_a: bool,
        //     ctx: &mut TxContext) {
        //     abort 0
        // }

        let args = vec![
            clock_arg,
            config_arg,
            pool_arg,
            position_arg,
            coin_a_arg,
            coin_b_arg,
            amount_arg,
            coin_a_max_arg,
            coin_b_max_arg,
            is_fixed_a_arg,
        ];

        ptb.command(Command::move_call(package, module, function, type_args, args));

        let builder = ptb.finish();

        let gas_budget = 10_000_000;
        let gas_price = self.sui_client.read_api().get_reference_gas_price().await?;

        let gas_coin = self
            .sui_client
            .coin_read_api()
            .get_coins(sender, None, None, None)
            .await?
            .data
            .into_iter()
            .next()
            .ok_or(anyhow::anyhow!("No coins found for sender"))?;

        let tx_data = TransactionData::new_programmable(
            sender,
            vec![gas_coin.object_ref()],
            builder,
            gas_budget,
            gas_price,
        );

        let signature = keypair.sign(tx_data.digest().as_ref());

        let transaction_response = self
            .sui_client
            .quorum_driver_api()
            .execute_transaction_block(
                Transaction::from_data(tx_data, vec![signature]),
                SuiTransactionBlockResponseOptions::full_content(),
                Some(ExecuteTransactionRequestType::WaitForLocalExecution),
            )
            .await?;

        tracing::info!("Transaction response: {:?}", transaction_response);

        Ok(())
    }

    async fn remove_liquidity(
        &self,
        keypair: &SuiKeyPair,
        options: RemoveLiquidityOptions,
    ) -> anyhow::Result<()> {
        tracing::info!(
            "Removing liquidity to pool: {:?}, signer: {:?}",
            options,
            keypair.public().encode_base64()
        );
        Ok(())
    }

    async fn close_position(
        &self,
        keypair: &SuiKeyPair,
        options: ClosePositionOptions,
    ) -> anyhow::Result<()> {
        tracing::info!(
            "Closing position: {:?}, signer: {:?}",
            options,
            keypair.public().encode_base64()
        );

        Ok(())
    }

    async fn open_position(
        &self,
        keypair: &SuiKeyPair,
        options: OpenPositionOptions,
    ) -> anyhow::Result<()> {
        tracing::info!(
            "Opening position: {:?}, signer: {:?}",
            options,
            keypair.public().encode_base64()
        );

        let sender = SuiAddress::from(&keypair.public());
        let package =
            ObjectID::from_hex_literal(&self.package_id).map_err(|e| anyhow::anyhow!(e))?;
        let module = Identifier::new("pool").map_err(|e| anyhow::anyhow!(e))?;
        let function = Identifier::new("open_position").map_err(|e| anyhow::anyhow!(e))?;

        let type_args = vec![
            TypeTag::from_str(options.coin_a.as_ref())?,
            TypeTag::from_str(options.coin_b.as_ref())?,
        ];

        let mut ptb = ProgrammableTransactionBuilder::new();

        let config_arg = ptb.obj(shared_obj(&self.sui_client, &self.global_config).await?)?;
        let pool_arg = ptb.obj(shared_obj_mut(&self.sui_client, &options.pool_id).await?)?;

        let lower_tick_bits_arg = ptb.pure(options.lower_tick_bits)?;
        let upper_tick_bits_arg = ptb.pure(options.upper_tick_bits)?;

        // public fun open_position<CoinTypeA, CoinTypeB>(
        //     protocol_config: &GlobalConfig,
        //     pool: &mut Pool<CoinTypeA, CoinTypeB>,
        //     lower_tick_bits: u32,
        //     upper_tick_bits: u32,
        //     ctx: &mut TxContext): Position {
        //     abort 0
        // }

        let args = vec![config_arg, pool_arg, lower_tick_bits_arg, upper_tick_bits_arg];

        tracing::info!("Calling move_call {:?}", args);

        ptb.command(Command::move_call(package, module, function, type_args, args));

        let builder = ptb.finish();

        let gas_budget = 10_000_000;
        let gas_price = self.sui_client.read_api().get_reference_gas_price().await?;

        let gas_coin = self
            .sui_client
            .coin_read_api()
            .get_coins(sender, None, None, None)
            .await?
            .data
            .into_iter()
            .next()
            .ok_or(anyhow::anyhow!("No coins found for sender"))?;

        tracing::info!("Gas coin: {:?}", gas_coin);

        let tx_data = TransactionData::new_programmable(
            sender,
            vec![gas_coin.object_ref()],
            builder,
            gas_budget,
            gas_price,
        );

        let intent_msg = IntentMessage::new(Intent::sui_transaction(), tx_data);
        let raw_tx = bcs::to_bytes(&intent_msg).expect("bcs should not fail");
        let mut hasher = sui_types::crypto::DefaultHash::default();
        hasher.update(raw_tx.clone());
        let digest = hasher.finalize().digest;

        let sui_sig = keypair.sign(&digest);
        sui_sig.verify_secure(&intent_msg, sender, sui_types::crypto::SignatureScheme::ED25519)?;
        tracing::info!("Signature: {:?}", sui_sig.encode_base64());

        // execute the transaction.
        let transaction_response = self
            .sui_client
            .quorum_driver_api()
            .execute_transaction_block(
                sui_types::transaction::Transaction::from_generic_sig_data(
                    intent_msg.value,
                    vec![GenericSignature::Signature(sui_sig)],
                ),
                SuiTransactionBlockResponseOptions::default(),
                None,
            )
            .await?;

        tracing::info!("Transaction response: {:?}", transaction_response);

        Ok(())
    }
}

pub async fn object_ref(client: &SuiClient, object_str: &str) -> anyhow::Result<ObjectRef> {
    let object_id = ObjectID::from_str(object_str)?;

    let object: SuiObjectResponse = client
        .read_api()
        .get_object_with_options(object_id, SuiObjectDataOptions::default())
        .await?;

    if let Some(error) = object.error {
        return Err(anyhow::anyhow!(error));
    }

    if let Some(data) = object.data {
        Ok(data.object_ref())
    } else {
        Err(anyhow::anyhow!("No data found for object {:?}", object_id))
    }
}

pub async fn owned_obj(client: &SuiClient, object_id: &str) -> anyhow::Result<ObjectArg> {
    let object_ref = object_ref(client, object_id).await?;
    Ok(ObjectArg::ImmOrOwnedObject(object_ref))
}

pub async fn shared_obj_mut(client: &SuiClient, object_id: &str) -> anyhow::Result<ObjectArg> {
    let object_ref = object_ref(client, object_id).await?;
    Ok(ObjectArg::SharedObject {
        id: object_ref.0,
        initial_shared_version: OBJECT_START_VERSION,
        mutable: true,
    })
}

pub async fn shared_obj(client: &SuiClient, object_id: &str) -> anyhow::Result<ObjectArg> {
    let object_ref = object_ref(client, object_id).await?;
    Ok(ObjectArg::SharedObject {
        id: object_ref.0,
        initial_shared_version: OBJECT_START_VERSION,
        mutable: false,
    })
}

pub fn clock_obj() -> ObjectArg {
    let object_arg = ObjectArg::SharedObject {
        id: SUI_CLOCK_OBJECT_ID,
        initial_shared_version: SUI_CLOCK_OBJECT_SHARED_VERSION,
        mutable: false,
    };
    object_arg
}
