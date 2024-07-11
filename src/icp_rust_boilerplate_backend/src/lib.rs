#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// Farmer Struct
#[derive(candid::CandidType, Serialize, Deserialize, Clone, Default, Debug)]
struct Farmer {
    id: u64,
    address: String,
    name: String,
    bio: String,
    category: String,
    price: u64,
    escrow_balance: u64,
    dispute_status: bool,
    rating: u8,
    product_status: String,
    consumer_address: Option<String>,
    is_sold: bool,
}

// ProductRecord Struct
#[derive(candid::CandidType, Serialize, Deserialize, Clone, Default, Debug)]
struct ProductRecord {
    id: u64,
    farmer_address: String,
}

// Implementing Storable and BoundedStorable for Farmer
impl Storable for Farmer {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Farmer {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Implementing Storable and BoundedStorable for ProductRecord
impl Storable for ProductRecord {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for ProductRecord {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static FARMERS_STORAGE: RefCell<StableBTreeMap<u64, Farmer, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static PRODUCTS_STORAGE: RefCell<StableBTreeMap<u64, ProductRecord, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));
}

// Farmer Payload
#[derive(candid::CandidType, Deserialize, Serialize)]
struct FarmerPayload {
    address: String,
    name: String,
    bio: String,
    category: String,
    price: u64,
    product_status: String,
}

// ProductBid Payload
#[derive(candid::CandidType, Deserialize, Serialize)]
struct ProductBidPayload {
    farmer_id: u64,
    consumer_address: String,
}

// MarkProductSold Payload
#[derive(candid::CandidType, Deserialize, Serialize)]
struct MarkProductSoldPayload {
    farmer_id: u64,
    consumer_address: String,
}

// WithdrawFromEscrow Payload
#[derive(candid::CandidType, Deserialize, Serialize)]
struct WithdrawFromEscrowPayload {
    farmer_id: u64,
    amount: u64,
}

// Accessor Functions

/// Retrieves the bio description of a farmer's product.
#[ic_cdk::query]
fn get_product_description(farmer_id: u64) -> Result<String, String> {
    FARMERS_STORAGE.with(|storage| {
        storage.borrow().get(&farmer_id).map_or_else(
            || Err("Farmer not found".to_string()),
            |farmer| Ok(farmer.bio.clone()),
        )
    })
}

/// Retrieves the price of a farmer's product.
#[ic_cdk::query]
fn get_product_price(farmer_id: u64) -> Result<u64, String> {
    FARMERS_STORAGE.with(|storage| {
        storage.borrow().get(&farmer_id).map_or_else(
            || Err("Farmer not found".to_string()),
            |farmer| Ok(farmer.price),
        )
    })
}

/// Retrieves the status of a farmer's product.
#[ic_cdk::query]
fn get_product_status(farmer_id: u64) -> Result<String, String> {
    FARMERS_STORAGE.with(|storage| {
        storage.borrow().get(&farmer_id).map_or_else(
            || Err("Farmer not found".to_string()),
            |farmer| Ok(farmer.product_status.clone()),
        )
    })
}

// Public Entry Functions

/// Adds a new product entry for a farmer.
#[ic_cdk::update]
fn add_product(payload: FarmerPayload) -> Result<Farmer, String> {
    // Validate input payload
    if payload.address.is_empty()
        || payload.name.is_empty()
        || payload.bio.is_empty()
        || payload.category.is_empty()
        || payload.product_status.is_empty()
    {
        return Err("All fields must be provided and non-empty".to_string());
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let farmer = Farmer {
        id,
        address: payload.address,
        name: payload.name,
        bio: payload.bio,
        category: payload.category,
        price: payload.price,
        escrow_balance: 0,
        dispute_status: false,
        rating: 0,
        product_status: payload.product_status,
        consumer_address: None,
        is_sold: false,
    };

    FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(id, farmer.clone()));

    Ok(farmer)
}

/// Allows a consumer to place a bid on a farmer's product.
#[ic_cdk::update]
fn product_bid(payload: ProductBidPayload) -> Result<(), String> {
    // Validate input payload
    if payload.consumer_address.is_empty() {
        return Err("Consumer address must be provided and non-empty".to_string());
    }

    let mut farmer = FARMERS_STORAGE
        .with(|storage| storage.borrow_mut().get(&payload.farmer_id))
        .ok_or("Farmer not found".to_string())?;

    // Check if the product already has a consumer bid
    if farmer.consumer_address.is_none() {
        farmer.consumer_address = Some(payload.consumer_address);
        farmer.product_status = "Bid Placed".to_string();
        FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(payload.farmer_id, farmer));
        Ok(())
    } else {
        Err("Product already has a bid placed".to_string())
    }
}

/// Allows a farmer to accept a bid on their product.
#[ic_cdk::update]
fn accept_bid(farmer_id: u64) -> Result<(), String> {
    let mut farmer = FARMERS_STORAGE
        .with(|storage| storage.borrow_mut().get(&farmer_id))
        .ok_or("Farmer not found".to_string())?;

    // Check if a consumer has placed a bid on the product
    if let Some(_) = farmer.consumer_address {
        farmer.product_status = "Bid Accepted".to_string();
        FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(farmer_id, farmer));
        Ok(())
    } else {
        Err("No bid placed on the product".to_string())
    }
}

/// Marks a product as sold, updating its status and consumer details.
#[ic_cdk::update]
fn mark_product_sold(payload: MarkProductSoldPayload) -> Result<(), String> {
    let mut farmer = FARMERS_STORAGE
        .with(|storage| storage.borrow_mut().get(&payload.farmer_id))
        .ok_or("Farmer not found".to_string())?;

    // Check if a consumer has placed a bid and mark the product as sold
    if let Some(_) = farmer.consumer_address {
        farmer.is_sold = true;
        farmer.product_status = "Product Sold".to_string();
        FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(payload.farmer_id, farmer));
        Ok(())
    } else {
        Err("No consumer has placed a bid on the product".to_string())
    }
}

/// Raises a dispute for a product, updating its status accordingly.
#[ic_cdk::update]
fn dispute_product(farmer_id: u64) -> Result<(), String> {
    let mut farmer = FARMERS_STORAGE
        .with(|storage| storage.borrow_mut().get(&farmer_id))
        .ok_or("Farmer not found".to_string())?;

    farmer.dispute_status = true;
    farmer.product_status = "Dispute Raised".to_string();
    FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(farmer_id, farmer));
    Ok(())
}

/// Resolves a dispute for a product based on the resolution provided.
#[ic_cdk::update]
fn resolve_dispute(farmer_id: u64, resolution: bool) -> Result<(), String> {
    let mut farmer = FARMERS_STORAGE
        .with(|storage| storage.borrow_mut().get(&farmer_id))
        .ok_or("Farmer not found".to_string())?;

    // Check if there is an active dispute to resolve
    if !farmer.dispute_status {
        return Err("No active dispute to resolve".to_string());
    }

    // Update product status based on resolution
    farmer.dispute_status = false;
    farmer.product_status = if resolution {
        "Dispute Resolved - Funds to Farmer".to_string()
    } else {
        "Dispute Resolved - Funds to Consumer".to_string()
    };

    FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(farmer_id, farmer));

    Ok(())
}

/// Releases the escrowed payment for a sold product to the farmer.
#[ic_cdk::update]
fn release_payment(farmer_id: u64) -> Result<(), String> {
    let mut farmer = FARMERS_STORAGE
        .with(|storage| storage.borrow_mut().get(&farmer_id))
        .ok_or("Farmer not found".to_string())?;

    // Check if the product is sold and there are no active disputes
    if farmer.is_sold && !farmer.dispute_status {
        farmer.escrow_balance = 0;
        let product_record = ProductRecord {
            id: farmer.id,
            farmer_address: farmer.address.clone(),
        };

        // Store product record in PRODUCTS_STORAGE
        PRODUCTS_STORAGE.with(|storage| storage.borrow_mut().insert(farmer.id, product_record));

        // Update farmer status in FARMERS_STORAGE
        FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(farmer_id, farmer));

        Ok(())
    } else {
        Err("Product is not sold or has an unresolved dispute".to_string())
    }
}

/// Adds funds to the escrow for a farmer's product.
#[ic_cdk::update]
fn add_to_escrow(farmer_id: u64, amount: u64) -> Result<(), String> {
    let mut farmer = FARMERS_STORAGE
        .with(|storage| storage.borrow_mut().get(&farmer_id))
        .ok_or("Farmer not found".to_string())?;

    farmer.escrow_balance += amount;

    FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(farmer_id, farmer));

    Ok(())
}

/// Withdraws funds from the escrow for a farmer's product.
#[ic_cdk::update]
fn withdraw_from_escrow(payload: WithdrawFromEscrowPayload) -> Result<(), String> {
    let mut farmer = FARMERS_STORAGE
        .with(|storage| storage.borrow_mut().get(&payload.farmer_id))
        .ok_or("Farmer not found".to_string())?;

    if farmer.escrow_balance >= payload.amount {
        farmer.escrow_balance -= payload.amount;
        FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(payload.farmer_id, farmer));
        Ok(())
    } else {
        Err("Insufficient funds in escrow".to_string())
    }
}

/// Updates the category of a farmer's product.
#[ic_cdk::update]
fn update_product_category(farmer_id: u64, category: String) -> Result<(), String> {
    FARMERS_STORAGE.with(|storage| {
        if let Some(mut farmer) = storage.borrow_mut().get(&farmer_id) {
            farmer.category = category;
            storage.borrow_mut().insert(farmer_id, farmer.clone());
            Ok(())
        } else {
            Err("Farmer not found".to_string())
        }
    })
}

/// Updates the description of a farmer's product.
#[ic_cdk::update]
fn update_product_description(farmer_id: u64, bio: String) -> Result<(), String> {
    FARMERS_STORAGE.with(|storage| {
        if let Some(mut farmer) = storage.borrow_mut().get(&farmer_id) {
            farmer.bio = bio;
            storage.borrow_mut().insert(farmer_id, farmer.clone());
            Ok(())
        } else {
            Err("Farmer not found".to_string())
        }
    })
}

/// Updates the price of a farmer's product.
#[ic_cdk::update]
fn update_product_price(farmer_id: u64, price: u64) -> Result<(), String> {
    FARMERS_STORAGE.with(|storage| {
        if let Some(mut farmer) = storage.borrow_mut().get(&farmer_id) {
            farmer.price = price;
            storage.borrow_mut().insert(farmer_id, farmer.clone());
            Ok(())
        } else {
            Err("Farmer not found".to_string())
        }
    })
}

/// Updates the status of a farmer's product.
#[ic_cdk::update]
fn update_product_status(farmer_id: u64, status: String) -> Result<(), String> {
    FARMERS_STORAGE.with(|storage| {
        if let Some(mut farmer) = storage.borrow_mut().get(&farmer_id) {
            farmer.product_status = status;
            storage.borrow_mut().insert(farmer_id, farmer.clone());
            Ok(())
        } else {
            Err("Farmer not found".to_string())
        }
    })
}

/// Rates a farmer based on a numeric rating.
#[ic_cdk::update]
fn rate_farmer(farmer_id: u64, rating: u8) -> Result<(), String> {
    let mut farmer = FARMERS_STORAGE
        .with(|storage| storage.borrow_mut().get(&farmer_id))
        .ok_or("Farmer not found".to_string())?;

    farmer.rating = rating;
    FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(farmer_id, farmer));

    Ok(())
}

// Error types for the system
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    EInvalidBid,
    EInvalidProduct,
    EDispute,
    EAlreadyResolved,
    ENotConsumer,
    EInvalidWithdrawal,
    EInsufficientEscrow,
}

// Exporting the Candid interface for the canister
ic_cdk::export_candid!();
