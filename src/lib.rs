#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol, Vec};

#[contracttype]
#[derive(Clone)]
pub struct Stream {
    pub sender: Address,
    pub recipient: Address,
    pub token: Address,
    pub amount: i128,
    pub start_time: u64,
    pub end_time: u64,
    pub withdrawn: i128,
}

#[contract]
pub struct StreamContract;

#[contractimpl]
impl StreamContract {
    pub fn create_stream(
        e: Env,
        sender: Address,
        recipient: Address,
        token: Address,
        amount: i128,
        duration: u64,
    ) -> u32 {
        sender.require_auth();

        let start_time = e.ledger().timestamp();
        let end_time = start_time + duration;
        
        let stream = Stream {
            sender: sender.clone(),
            recipient,
            token: token.clone(),
            amount,
            start_time,
            end_time,
            withdrawn: 0,
        };

        // Transfer tokens to the contract
        // (Assuming a standard token interface is used)
        // token::Client::new(&e, &token).transfer(&sender, &e.current_contract_address(), &amount);

        let stream_id = e.storage().instance().get(&Symbol::new(&e, "next_id")).unwrap_or(0);
        e.storage().persistent().set(&stream_id, &stream);
        e.storage().instance().set(&Symbol::new(&e, "next_id"), &(stream_id + 1));

        stream_id
    }

    pub fn withdraw(e: Env, stream_id: u32) {
        let mut stream: Stream = e.storage().persistent().get(&stream_id).unwrap();
        stream.recipient.require_auth();

        let now = e.ledger().timestamp();
        let total_duration = stream.end_time - stream.start_time;
        let elapsed = if now >= stream.end_time {
            total_duration
        } else if now <= stream.start_time {
            0
        } else {
            now - stream.start_time
        };

        let total_claimable = (stream.amount * elapsed as i128) / total_duration as i128;
        let withdraw_amount = total_claimable - stream.withdrawn;

        if withdraw_amount > 0 {
            stream.withdrawn += withdraw_amount;
            e.storage().persistent().set(&stream_id, &stream);

            // Transfer tokens to the recipient
            // token::Client::new(&e, &stream.token).transfer(&e.current_contract_address(), &stream.recipient, &withdraw_amount);
        }
    }

    pub fn get_stream(e: Env, stream_id: u32) -> Stream {
        e.storage().persistent().get(&stream_id).unwrap()
    }
}
