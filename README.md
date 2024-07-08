# Decentralized Mobile Market Platform

This project is a decentralized platform built on the Internet Computer for facilitating direct transactions between farmers and consumers, bypassing traditional market intermediaries. The platform allows farmers to list products, manage bids, and handle payments securely within a decentralized framework, enhancing transparency and efficiency.

## Key Features

### Farmer Management
- **Add Farmer**: Allows users to create farmer profiles and list products.
- **Update Farmer Info**: Update farmer's bio, category, and price.
- **Get Farmer Info**: Retrieve farmer's product description, price, and status.

### Product Management
- **Add Product**: Allows farmers to list new products for sale.
- **Product Bid**: Enables consumers to place bids on products.
- **Accept Bid**: Allows farmers to accept bids placed by consumers.
- **Mark Product Sold**: Marks a product as sold once a transaction is completed.
- **Dispute Management**: Handle disputes raised by consumers or farmers.
- **Resolve Dispute**: Resolve disputes and update product status accordingly.
- **Release Payment**: Release payment from escrow to the farmer.
- **Add to Escrow**: Add funds to the escrow balance.
- **Withdraw from Escrow**: Withdraw funds from the escrow balance.

### Error Handling
- **Not Found**: Returns an error if a requested item is not found.
- **Unauthorized Access**: Returns an error if a user tries to perform an action without necessary permissions.
- **Invalid Bid**: Returns an error if a bid is invalid.
- **Invalid Product**: Returns an error if a product is invalid.
- **Dispute**: Returns an error if there is a dispute.
- **Already Resolved**: Returns an error if the dispute is already resolved.
- **Not Consumer**: Returns an error if the user is not a consumer.
- **Invalid Withdrawal**: Returns an error if the withdrawal is invalid.
- **Insufficient Escrow**: Returns an error if the escrow balance is insufficient.


## Requirements
* rustc 1.64 or higher
```bash
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
$ source "$HOME/.cargo/env"
```
* rust wasm32-unknown-unknown target
```bash
$ rustup target add wasm32-unknown-unknown
```
* candid-extractor
```bash
$ cargo install candid-extractor
```
* install `dfx`
```bash
$ DFX_VERSION=0.15.0 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
$ echo 'export PATH="$PATH:$HOME/bin"' >> "$HOME/.bashrc"
$ source ~/.bashrc
$ dfx start --background
```

If you want to start working on your project right away, you might want to try the following commands:

```bash
$ cd icp_rust_boilerplate/
$ dfx help
$ dfx canister --help
```

## Update dependencies

update the `dependencies` block in `/src/{canister_name}/Cargo.toml`:
```
[dependencies]
candid = "0.9.9"
ic-cdk = "0.11.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1.0"
ic-stable-structures = { git = "https://github.com/lwshang/stable-structures.git", branch = "lwshang/update_cdk"}
```

## did autogenerate

Add this script to the root directory of the project:
```
https://github.com/buildwithjuno/juno/blob/main/scripts/did.sh
```

Update line 16 with the name of your canister:
```
https://github.com/buildwithjuno/juno/blob/main/scripts/did.sh#L16
```

After this run this script to generate Candid.
Important note!

You should run this script each time you modify/add/remove exported functions of the canister.
Otherwise, you'll have to modify the candid file manually.

Also, you can add package json with this content:
```
{
    "scripts": {
        "generate": "./did.sh && dfx generate",
        "gen-deploy": "./did.sh && dfx generate && dfx deploy -y"
      }
}
```

and use commands `npm run generate` to generate candid or `npm run gen-deploy` to generate candid and to deploy a canister.

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
$ dfx start --background

# Deploys your canisters to the replica and generates your candid interface
$ dfx deploy
```