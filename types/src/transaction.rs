use crate::address::Address;
use crate::b256::B256;
use crate::execution_error::ExecutionError;
use std::cmp::min;

#[derive(Clone, Debug)]
pub struct AccessListItem {
    pub address: Address,
    pub storage_keys: Vec<B256>,
}

// nonce
// gas-limit
// to
// value
// data 
#[derive(Clone, Debug)]
pub enum Transaction {
    Legacy {
        nonce: u64,
        gas_price: u128,
        gas_limit: u64,
        to: Option<Address>,
        value: u128,
        data: Vec<u8>
    },
    Eip1559 {
        nonce: u64,
        max_priority_fee_per_gas: u128,
        max_fee_per_gas: u128,
        gas_limit: u64,
        to: Option<Address>,
        value: u128,
        data: Vec<u8>,
        access_list: Vec<AccessListItem>
    },
    Eip4844 {
        nonce: u64,
        max_priority_fee_per_gas: u128,
        max_fee_per_gas: u128,
        max_fee_per_blob_gas: u128,
        gas_limit: u64,
        to: Option<Address>,
        value: u128,
        data: Vec<u8>,
        access_list: Vec<AccessListItem>,
        blob_versioned_hashes: Vec<B256>
    }
}

impl Transaction {
    pub fn tx_type(&self) -> Result<u8, ExecutionError> {
        match self {
            Transaction::Legacy { .. } => Ok(0),
            Transaction::Eip1559 { .. } => Ok(2),
            Transaction::Eip4844 { .. } => Ok(3),
        }
    }

    pub fn is_create(&self) -> Result<bool, ExecutionError> {
        match self {
            Transaction::Legacy { to, .. }
             | Transaction::Eip1559 { to, .. }
             | Transaction::Eip4844 { to, .. } => Ok(to.is_none())
        }
    }

    pub fn effective_gas_price(&self, base_fee: u128) -> Result<u128, ExecutionError> {
        match self {
            Transaction::Legacy {gas_price, ..} => Ok(*gas_price),
            Transaction::Eip1559 {max_fee_per_gas, max_priority_fee_per_gas, ..} 
            | Transaction::Eip4844 {max_fee_per_gas, max_priority_fee_per_gas, ..} => { 
                if base_fee > *max_fee_per_gas {
                    return Err(ExecutionError::InsufficientMaxFee { base_fee, max_fee: *max_fee_per_gas });
                }
                let addition = base_fee.checked_add(*max_priority_fee_per_gas).ok_or(ExecutionError::Overflow)?;
                let effective_gas_price = min(*max_fee_per_gas, addition);
                Ok(effective_gas_price)
            },
        }
    }

    pub fn max_cost(&self) -> Result<u128, ExecutionError> {
        match self {
            Transaction::Legacy { gas_price, gas_limit, value, ..} => {
                let multiplication = gas_price.checked_mul((*gas_limit).into()).ok_or(ExecutionError::Overflow)?;
                let max_cost = multiplication.checked_add(*value).ok_or(ExecutionError::Overflow)?;
                Ok(max_cost)
            }
            Transaction::Eip1559 { gas_limit, value, max_fee_per_gas, ..} 
            | Transaction::Eip4844 { gas_limit, value, max_fee_per_gas, ..} => {
                let multiplication = max_fee_per_gas.checked_mul((*gas_limit).into()).ok_or(ExecutionError::Overflow)?;
                let max_cost = multiplication.checked_add(*value).ok_or(ExecutionError::Overflow)?;
                Ok(max_cost)
            }
        }
    }
}

pub struct TransactionSummary{
    pub total_value: u128,
    pub total_gas_limit: u64,
    pub create_count: usize,
    pub tx_count: usize
}

pub fn summarise_transactions(txs: &[Transaction]) -> Result<TransactionSummary, ExecutionError> {
    let mut total_value = 0;
    let mut total_gas_limit = 0;
    let mut create_count = 0;
    let tx_count = txs.len();

    for tx in txs {
        let (value, gas_limit) = match tx {
            Transaction::Legacy{value, gas_limit, ..}
            | Transaction::Eip1559{value, gas_limit, ..}
            | Transaction::Eip4844{value, gas_limit, ..} => (value, gas_limit),
        };

        total_value = value.checked_add(total_value).ok_or(ExecutionError::Overflow)?;
        total_gas_limit = gas_limit.checked_add(total_gas_limit).ok_or(ExecutionError::Overflow)?;
        if tx.is_create()? {
            create_count += 1;
        }
    }

    Ok(TransactionSummary { total_value, total_gas_limit, create_count, tx_count })
}