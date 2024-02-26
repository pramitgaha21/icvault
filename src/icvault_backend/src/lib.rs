use std::{cell::RefCell, collections::HashMap};

use candid::{CandidType, Nat, Principal};
use ic_cdk::{query, update};
use icrc_ledger_types::icrc1::{account::{Account, Subaccount}, transfer::{TransferArg, TransferError}};

const ICP_LEDGER: Principal = Principal::from_slice(b"ryjl3-tyaaa-aaaaa-aaaba-cai");
const FEE: u128 = 10_000;

#[derive(Clone, CandidType)]
pub struct BalanceDetail{
    // generated for user, with combination of canister's principal and subaccount generated from user's principal
    pub deposit_address: Account,
    pub amount: Nat,
}

pub struct State {
    pub balance_state: HashMap<Principal, BalanceDetail>,
}

thread_local! {
    pub static STATE: RefCell<State> = RefCell::new(State{
        balance_state: HashMap::default()
    })
}

fn principal_to_subaccount(principal: &Principal) -> Subaccount{
    let mut subaccount = [0; 32];
    let sliced = principal.as_slice();
    subaccount[0..sliced.len()].copy_from_slice(&sliced);
    subaccount
}

#[update]
pub fn register() -> bool{
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous(){
        ic_cdk::trap("Anonymous Caller")
    }
    let account = Account{
        owner: caller.clone(),
        subaccount: Some(principal_to_subaccount(&caller))
    };
    STATE.with_borrow_mut(|s| {
        if s.balance_state.contains_key(&caller){
            ic_cdk::trap("User Already Registered")
        }
        s.balance_state.insert(caller, BalanceDetail{
            deposit_address: account,
            amount: Nat::from(0_u128)
        });
        true
    })
}

#[update]
pub async fn deposit() -> Nat{
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous(){
        ic_cdk::trap("Anonymous Caller")
    }
    let mut user_detail = match STATE.with_borrow(|s| s.balance_state.get(&caller).cloned()){
        None => ic_cdk::trap("User Not Registered"),
        Some(detail) => detail
    };
    let balance: (Nat,) = ic_cdk::call(ICP_LEDGER, "icrc1_balance_of", (user_detail.deposit_address,)).await.unwrap();
    user_detail.amount += balance.0.clone();
    STATE.with_borrow_mut(|s| s.balance_state.insert(caller, user_detail));
    balance.0 // returning how much balance was deposited
}

#[update]
pub async fn withdraw(amount: Nat, to: Account) -> bool{
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous(){
        ic_cdk::trap("Anonymous Caller")
    }
    let user_vault_address = STATE.with_borrow_mut(|s| {
        match s.balance_state.get_mut(&caller){
            None => ic_cdk::trap("User Not Registered"),
            Some(detail) => {
                if detail.amount < amount{
                    ic_cdk::trap("Not Enough Balance")
                }
                detail.amount -= amount.clone();
                detail.deposit_address.clone()
            }
        }
    });
    let fee = Nat::from(FEE);
    let transfer_arg = TransferArg{
        from_subaccount: user_vault_address.subaccount.clone(),
        to,
        amount: amount.clone() - fee.clone(),
        fee: Some(fee),
        created_at_time: None,
        memo: None
    };
    let result: Result<Nat, TransferError> = match ic_cdk::call(ICP_LEDGER, "icrc1_transfer", (transfer_arg,)).await{
        Ok((res,)) => res,
        // checking for error for cross canister call
        Err((code, msg)) => {
            STATE.with_borrow_mut(|s|{
                s.balance_state.get_mut(&caller).unwrap().amount += amount;
            });
            let error_message = format!("Failed to withdraw: Error code: {:?}, Error Message: {:?}", code, msg);
            ic_cdk::trap(&error_message);
        }
    };
    // checking for error returned from transfer
    if let Err(e) = result{
        STATE.with_borrow_mut(|s|{
            s.balance_state.get_mut(&caller).unwrap().amount += amount;
        });
        let error_message = format!("Failed to withdraw: Error Message: {:?}", e);
        ic_cdk::trap(&error_message);
    }
    true
}

#[query]
pub fn query_detail() -> Option<BalanceDetail>{
    let caller = ic_cdk::caller();
    STATE.with_borrow(|s| s.balance_state.get(&caller).cloned())
}

ic_cdk::export_candid!();