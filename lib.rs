#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod burger_shop {

    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use scale::{Decode, Encode};

    // this is the main contract, this is what gets instantiated
    #[ink(storage)]
    pub struct BurgerShop {
        orders: Vec<(u32, Order)>,
        orders_mapping: Mapping<u32, Order>,
    }

    // The order type
    #[derive(Encode, Decode, Debug, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Order {
        list_of_items: Vec<FoodItem>,
        customer: AccountId,
        total_price: Balance,
        paid: bool,
        order_id: u32,
    }

    impl Order {
        fn new(list_of_items: Vec<FoodItem>, customer: AccountId, id: u32) -> Self {
            let total_price = Order::total_price(&list_of_items);
            Self {
                list_of_items,
                customer,
                total_price,
                paid: false,
                order_id: id, // Default is "getting ingredients" in this case
            }
        }

        fn total_price(list_of_items: &Vec<FoodItem>) -> Balance {
            let mut total = 0;
            for item in list_of_items {
                total += item.price()
            }
            total
        }
    }

    // Food Item type, basically for each food item
    #[derive(Encode, Decode, Debug, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct FoodItem {
        burger_menu: BurgerMenu,
        amount: u32,
    }

    impl FoodItem {
        fn price(&self) -> Balance {
            match self.burger_menu {
                BurgerMenu::CheeseBurger => BurgerMenu::CheeseBurger.price() * self.amount as u128,
                BurgerMenu::ChickenBurger => {
                    BurgerMenu::ChickenBurger.price() * self.amount as u128
                }
                BurgerMenu::VeggieBurger => BurgerMenu::VeggieBurger.price() * self.amount as u128,
            }
        }
    }

    // Burger Type
    #[derive(Encode, Decode, Debug, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum BurgerMenu {
        CheeseBurger,
        ChickenBurger,
        VeggieBurger,
    }
    impl BurgerMenu {
        fn price(&self) -> Balance {
            match self {
                Self::CheeseBurger => 12,
                Self::VeggieBurger => 10,
                Self::ChickenBurger => 15,
            }
        }
    }

    // For catching errors that happens during shop operations
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum BurgerShopError {
        /// Errors types for different errors.
        PaymentError,
        OrderNotCompleted,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    /// Event when shop owner get all orders in storage
    #[ink(event)]
    pub struct GetAllOrders {
        #[ink(topic)]
        orders: Vec<(u32, Order)>,
    }

    /// Event when shop owner gets a single order
    #[ink(event)]
    pub struct GetSingleOrder {
        #[ink(topic)]
        single_order: Order,
    }

    /// Event when the shop_owner creates his shop
    #[ink(event)]
    pub struct CreatedShopAndStorage {
        #[ink(topic)]
        orders: Vec<(u32, Order)>, // this only contains a vector because `Mapping` doesn't implement "encode" trait, this means you can't encode or decode it for operational purposes
    }

    // result type
    pub type Result<T> = core::result::Result<T, BurgerShopError>;

    /// Implementing the contract
    impl BurgerShop {
        #[ink(constructor)]
        /// Instantiates the contract
        pub fn new() -> Self {
            let order_storage_vector: Vec<(u32, Order)> = Vec::new();
            let order_storage_mapping = Mapping::new();

            Self {
                orders: order_storage_vector,
                orders_mapping: order_storage_mapping,
            }
        }

        /// takes the order and makes the payment, we aren't implementing cart feature here for simplicity purposes, ideally the cart feature should be implemented in the frontend
        #[ink(message, payable)]
        pub fn take_order_and_payment(&mut self, list_of_items: Vec<FoodItem>) -> Result<Order> {
            // Gets the caller account id
            let caller = Self::env().caller();

            // this is assertion is opinionated, if you don't want to limit the shop owner from creating an order, you can remove this line
            assert!(
                caller != self.env().account_id(),
                "You are not the customer!"
            );

            // assert the order contains at least 1 item
            for item in &list_of_items {
                assert!(item.amount > 0, "Can't take an empty order")
            }

            // our own local id, you can change this to a hash if you want, but remember to make the neccessary type changes too!
            let id = self.orders.len() as u32;

            // Calculate and set order price
            let total_price = Order::total_price(&list_of_items);
            let mut order = Order::new(list_of_items, caller, id);
            order.total_price = total_price;

            assert!(
                order.paid == false,
                "Can't pay for an order that is paid for already"
            );

            let multiply: Balance = 1_000_000_000_000;
            let transfered_val = self.env().transferred_value();

            // assert the value sent == total price
            assert!(
                transfered_val
                    == order
                        .total_price
                        .checked_mul(multiply)
                        .expect("Overflow!!!"),
                "Please pay complete amount"
            );
            ink::env::debug_println!("received payment: {}", transfered_val);

            // make payment
            match self
                .env()
                .transfer(self.env().account_id(), order.total_price)
            {
                Ok(_) => {
                    // get current length of the list orders in storage, this will act as our unique id
                    let id = self.orders.len() as u32;
                    // mark order as paid
                    order.paid = true;

                    // Emit event
                    self.env().emit_event(Transfer {
                        from: Some(order.customer),
                        to: Some(self.env().account_id()),
                        value: order.total_price,
                    });

                    // Push to storage
                    self.orders_mapping.insert(id, &order);
                    self.orders.push((id, order.clone()));
                    Ok(order)
                }
                Err(_) => Err(BurgerShopError::PaymentError),
            }
        }

        #[ink(message)]
        /// gets a single order from storage
        pub fn get_single_order(&self, id: u32) -> Order {
            // get single order
            let single_order = self.orders_mapping.get(id).expect("Oh no, Order not found");

            single_order
        }

        #[ink(message)]
        /// gets the orders in storage
        pub fn get_orders(&self) -> Option<Vec<(u32, Order)>> {
            // Get all orders
            let get_all_orders = &self.orders;

            if get_all_orders.len() > 0 {
                Some(get_all_orders.to_vec()) // converts ref to an owned/new vector
            } else {
                None
            }
        }
    }
}
