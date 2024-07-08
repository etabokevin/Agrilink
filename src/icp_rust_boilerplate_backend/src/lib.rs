#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
// use ic_cdk::api::time;
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

// Storable and BoundedStorable implementations for Farmer
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

// Storable and BoundedStorable implementations for ProductRecord
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

// Product_bid Payload
#[derive(candid::CandidType, Deserialize, Serialize)]
struct ProductBidPayload {
    farmer_id: u64,
    consumer_address: String,
}

// Mark_Product_Sold Payload
#[derive(candid::CandidType, Deserialize, Serialize)]
struct MarkProductSoldPayload {
    farmer_id: u64,
    consumer_address: String,
}

// Withdraw_from_escrow Payload
#[derive(candid::CandidType, Deserialize, Serialize)]
struct WithdrawFromEscrowPayload {
    farmer_id: u64,
    amount: u64,
}

// Accessor Functions

#[ic_cdk::query]
fn get_product_description(farmer_id: u64) -> Result<String, String> {
    FARMERS_STORAGE.with(|storage| {
        storage.borrow().get(&farmer_id).map_or_else(
            || Err("Farmer not found".to_string()),
            |farmer| Ok(farmer.bio.clone()),
        )
    })
}

#[ic_cdk::query]
fn get_product_price(farmer_id: u64) -> Result<u64, String> {
    FARMERS_STORAGE.with(|storage| {
        storage.borrow().get(&farmer_id).map_or_else(
            || Err("Farmer not found".to_string()),
            |farmer| Ok(farmer.price),
        )
    })
}

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

#[ic_cdk::update]
fn add_product(payload: FarmerPayload) -> Result<Farmer, String> {
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

// Function for a consumer to bid on a product
#[ic_cdk::update]
fn product_bid(payload: ProductBidPayload) -> Result<(), String> {
    let mut farmer = FARMERS_STORAGE
        .with(|storage| storage.borrow_mut().get(&payload.farmer_id))
        .ok_or("Farmer not found".to_string())?;

    if farmer.consumer_address.is_none() {
        farmer.consumer_address = Some(payload.consumer_address);
        farmer.product_status = "Bid Placed".to_string();
        FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(payload.farmer_id, farmer));
        Ok(())
    } else {
        Err("Product already bid on".to_string())
    }
}

// Function for a farmer to accept a bid on their product
#[ic_cdk::update]
fn accept_bid(farmer_id: u64) -> Result<(), String> {
    let mut farmer = FARMERS_STORAGE
        .with(|storage| storage.borrow_mut().get(&farmer_id))
        .ok_or("Farmer not found".to_string())?;
    
    if farmer.consumer_address.is_some() {
        farmer.product_status = "Bid Accepted".to_string();
        FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(farmer_id, farmer));
        Ok(())
    } else {
        Err("No bid to accept".to_string())
    }
}

#[ic_cdk::update]
fn mark_product_sold(payload: MarkProductSoldPayload) -> Result<(), String> {
    // Retrieve and update the farmer within a single borrow scope
    let mut farmer = FARMERS_STORAGE
        .with(|storage| storage.borrow_mut().get(&payload.farmer_id))
        .ok_or("Farmer not found".to_string())?;

    if farmer.consumer_address.is_some() {
        farmer.is_sold = true;
        farmer.product_status = "Product Sold".to_string();
        FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(payload.farmer_id, farmer));
        Ok(())
    } else {
        Err("No consumer to sell to".to_string())
    }
}

#[ic_cdk::update]
fn dispute_product(farmer_id: u64) -> Result<(), String> {
    // Retrieve and update the farmer within a single borrow scope
    let mut farmer = FARMERS_STORAGE
        .with(|storage| storage.borrow_mut().get(&farmer_id))
        .ok_or("Farmer not found".to_string())?;

    farmer.dispute_status = true;
    farmer.product_status = "Dispute Raised".to_string();
    FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(farmer_id, farmer));
    Ok(())
}

#[ic_cdk::update]
fn resolve_dispute(farmer_id: u64, resolution: bool) -> Result<(), String> {
    // Retrieve the farmer within a single borrow scope
    let mut farmer = FARMERS_STORAGE
        .with(|storage| storage.borrow_mut().get(&farmer_id))
        .ok_or("Farmer not found".to_string())?;

    // Check for dispute status
    if !farmer.dispute_status {
        return Err("No dispute to resolve".to_string());
    }

    // Update the farmer's dispute status and product status
    farmer.dispute_status = false;
    farmer.product_status = if resolution {
        "Dispute Resolved - Funds to Farmer".to_string()
    } else {
        "Dispute Resolved - Funds to Consumer".to_string()
    };

    // Insert the updated farmer back into the storage
    FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(farmer_id, farmer));

    Ok(())
}

#[ic_cdk::update]
fn release_payment(farmer_id: u64) -> Result<(), String> {
    // Retrieve the farmer within a single borrow scope
    let mut farmer = FARMERS_STORAGE
        .with(|storage| storage.borrow_mut().get(&farmer_id))
        .ok_or("Farmer not found".to_string())?;

    // Check if the product is sold and no dispute is unresolved
    if farmer.is_sold && !farmer.dispute_status {
        farmer.escrow_balance = 0;
        let product_record = ProductRecord {
            id: farmer.id,
            farmer_address: farmer.address.clone(),
        };

        // Insert the product record into PRODUCTS_STORAGE
        PRODUCTS_STORAGE.with(|storage| storage.borrow_mut().insert(farmer.id, product_record));

        // Insert the updated farmer back into the FARMERS_STORAGE
        FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(farmer_id, farmer));

        Ok(())
    } else {
        Err("Product not sold or dispute unresolved".to_string())
    }
}

#[ic_cdk::update]
fn add_to_escrow(farmer_id: u64, amount: u64) -> Result<(), String> {
    // Retrieve and update the farmer within a single borrow scope
    let mut farmer = FARMERS_STORAGE
        .with(|storage| storage.borrow_mut().get(&farmer_id))
        .ok_or("Farmer not found".to_string())?;

    farmer.escrow_balance += amount;

    // Insert the updated farmer back into the storage
    FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(farmer_id, farmer));

    Ok(())
}

#[ic_cdk::update]
fn withdraw_from_escrow(payload: WithdrawFromEscrowPayload) -> Result<(), String> {
    // Retrieve and update the farmer within a single borrow scope
    let mut farmer = FARMERS_STORAGE
        .with(|storage| storage.borrow_mut().get(&payload.farmer_id))
        .ok_or("Farmer not found".to_string())?;

    if farmer.escrow_balance >= payload.amount {
        farmer.escrow_balance -= payload.amount;

        // Insert the updated farmer back into the storage
        FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(payload.farmer_id, farmer));

        Ok(())
    } else {
        Err("Insufficient funds in escrow".to_string())
    }
}

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

#[ic_cdk::update]
fn rate_farmer(farmer_id: u64, rating: u8) -> Result<(), String> {
    let mut farmer = FARMERS_STORAGE
        .with(|storage| storage.borrow_mut().get(&farmer_id))
        .ok_or("Farmer not found".to_string())?;

    farmer.rating = rating;
    FARMERS_STORAGE.with(|storage| storage.borrow_mut().insert(farmer_id, farmer));
    Ok(())
}

// Error types
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

// need this to generate candid
ic_cdk::export_candid!();
