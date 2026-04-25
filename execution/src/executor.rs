use types::{Address, Bloom, Transaction};

use crate::{
    error::ExecutionError,
    in_memory::InMemoryProvider,
    primitives::{AccountInfo, Block, Receipt},
    providers::StateProvider,
};

#[derive(Debug)]
pub struct ExecutionOutput {
    pub gas_used: u64,
    pub receipts: Vec<Receipt>,
    pub logs_bloom: Bloom,
}

pub struct BlockWithSenders {
    pub block: Block,
    pub senders: Vec<Address>
}

pub trait BlockExecutor {
    type Output;
    fn execute(
        &self,
        block_with_senders: &BlockWithSenders,
        state: &mut InMemoryProvider,
    ) -> Result<Self::Output, ExecutionError>;
}

pub struct ValueTransferExecutor;

impl BlockExecutor for ValueTransferExecutor {
    type Output = ExecutionOutput;
    fn execute(
        &self,
        block_with_senders: &BlockWithSenders,
        state: &mut InMemoryProvider,
    ) -> Result<Self::Output, ExecutionError> {
        let mut cumulative_gas_used: u64 = 0;
        let mut receipts = vec![];
        let mut logs_bloom: Bloom = Bloom::zero();
        for (i, (signed_tx,sender) ) in block_with_senders.block.transactions.iter().zip(block_with_senders.senders.iter()).enumerate() {
            let mut account = state.get_account(*sender)?;
            let sender_nonce = account.nonce;
            match &signed_tx.transaction {
                Transaction::Legacy {
                    nonce,
                    gas_limit,
                    value,
                    to,
                    ..
                }
                | Transaction::Eip1559 {
                    nonce,
                    gas_limit,
                    value,
                    to,
                    ..
                }
                | Transaction::Eip4844 {
                    nonce,
                    gas_limit,
                    value,
                    to,
                    ..
                } => {
                    if *nonce != sender_nonce {
                        return Err(ExecutionError::InvalidNonce {
                            address: *sender,
                            expected: sender_nonce,
                            actual: *nonce,
                        });
                    }

                    let max_cost = signed_tx.transaction.max_cost()?;
                    if max_cost > account.balance {
                        return Err(ExecutionError::InsufficientBalance {
                            address: *sender,
                            available: account.balance,
                            required: max_cost,
                        });
                    }
                    let effective_gas_price = signed_tx
                        .transaction
                        .effective_gas_price(block_with_senders.block.header.base_fee_per_gas)?;

                    account.balance = account
                        .balance
                        .checked_sub(
                            effective_gas_price
                                .checked_mul(*gas_limit as u128)
                                .ok_or(ExecutionError::Overflow)?
                                .checked_add(*value)
                                .ok_or(ExecutionError::Overflow)?,
                        )
                        .ok_or(ExecutionError::Underflow)?;

                    account.nonce = account
                        .nonce
                        .checked_add(1)
                        .ok_or(ExecutionError::Overflow)?;
                    state.set_account(*sender, account);

                    if let Some(recipient_address) = to {
                        let mut recipient =
                            state.get_account(*recipient_address).or_else(|e| match e {
                                ExecutionError::AccountNotFound { .. } => {
                                    Ok(AccountInfo::default())
                                }
                                other => Err(other),
                            })?;
                        recipient.balance = recipient
                            .balance
                            .checked_add(*value)
                            .ok_or(ExecutionError::Overflow)?;
                        state.set_account(*recipient_address, recipient);
                    } else {
                        todo!()
                    }
                    let gas_used = *gas_limit;
                    cumulative_gas_used = cumulative_gas_used
                        .checked_add(gas_used)
                        .ok_or(ExecutionError::Overflow)?;
                    let receipt = Receipt {
                        transaction_hash: signed_tx.hash()?,
                        transaction_index: i as u64,
                        block_hash: block_with_senders.block.header.hash,
                        block_number: block_with_senders.block.header.block_number,
                        from: *sender,
                        to: *to,
                        contract_address: None,
                        cumulative_gas_used,
                        effective_gas_price,
                        gas_used,
                        status: true,
                        logs: vec![],
                        logs_bloom: Bloom::zero(),
                    };

                    logs_bloom |= &receipt.logs_bloom;
                    receipts.push(receipt);
                }
            }
        }

        Ok(ExecutionOutput {
            gas_used: cumulative_gas_used,
            receipts,
            logs_bloom,
        })
    }
}
