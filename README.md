# icvault

```bash
dfx start --clean --background

dfx identity new minter
dfx identity use minter
export MINTER_ACCOUNT_ID=$(dfx ledger account-id)

dfx identity use default
export DEFAULT_ACCOUNT_ID=$(dfx ledger account-id)

dfx deploy --specified-id ryjl3-tyaaa-aaaaa-aaaba-cai ICP --argument "
  (variant {
    Init = record {
      minting_account = \"$MINTER_ACCOUNT_ID\";
      initial_values = vec {
        record {
          \"$DEFAULT_ACCOUNT_ID\";
          record {
            e8s = 10_000_000_000 : nat64;
          };
        };
      };
      send_whitelist = vec {};
      transfer_fee = opt record {
        e8s = 10_000 : nat64;
      };
      token_symbol = opt \"LICP\";
      token_name = opt \"Local ICP\";
    }
  })
"

dfx deploy icvault

dfx identity new user1
dfx identity use user1

dfx canister call icvault register

# returns the deposit address with balance for the user
dfx canister call query_detail

# here I'll be directly minting tokens to the user's deposits address
dfx identity use minter

dfx canister call ICP icrc1_transfer '(record{
  amount= x;
  to= #paste the account returned from calling the function
})'

dfx identity use user1

dfx canister call icvault deposit

dfx identity new receiver
dfx identity use receiver
export RECEIVER=$(dfx identity get-principal)

dfx identity use user1

dfx canister call icvault withdraw '(record{
  amount= x;
  to=record{
    owner= principal "${RECEIVER}";
    subaccount=null;
  }
})'
```