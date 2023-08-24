#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod burger_shop {
    extern crate alloc;
    // use alloc::fmt::format;
    use ink::prelude::format;
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
    #[derive(Encode, Decode, Debug, PartialEq, Clone)]
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
        pub fn new(list_of_items: Vec<FoodItem>, customer: AccountId, id: u32) -> Self {
            let total_price = Order::total_price(&list_of_items);
            Self {
                list_of_items,
                customer,
                total_price,
                paid: false,
                order_id: id,
            }
        }

        pub fn total_price(list_of_items: &Vec<FoodItem>) -> Balance {
            let mut total = 0;
            for item in list_of_items {
                total += item.price()
            }
            total
        }
    }

    // Food Item type, basically for each food item
    #[derive(Encode, Decode, Debug, PartialEq, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct FoodItem {
        pub burger_menu: BurgerMenu,
        pub amount: u32,
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
    #[derive(Encode, Decode, PartialEq, Debug, Clone)]
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

            let multiply: Balance = 1_000_000_000_000; // this equals to 1 Azero, so we doing some conversion
            let transfered_val = self.env().transferred_value();

            // assert the value sent == total price
            assert!(
                transfered_val
                    == order
                        .total_price
                        .checked_mul(multiply)
                        .expect("Overflow!!!"),
                "{}",
                format!("Please pay complete amount which is {}", order.total_price)
            );

            ink::env::debug_println!("Expected value: {}", order.total_price);
            ink::env::debug_println!(
                "Expected received payment without conversion: {}",
                transfered_val
            ); // we are printing the expected value as is

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

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        fn first_e2e_transfer_works(mut client: ink_e2e::Client<C, E>) {
            // initialize the contract
            let constructor = BurgerShopRef::new();
            let contract_acc_id =
                client.instantiate("burger_shop", ink_e2e::alice(), constructor, 1000, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use ink::env::DefaultEnvironment;

    use crate::{
        burger_shop::{BurgerShop, FoodItem},
        *,
    };
    // use crate::burger_shop::BurgerShop;

    #[test]
    fn first_test() {
        assert!(2 == 2);
    }

    #[ink::test]
    fn first_integration_test_works() {
        let shop = BurgerShop::new();
        assert_eq!(None, shop.get_orders());
    }

    #[ink::test]
    fn order_and_payment_works() {
        let mut shop = BurgerShop::new();
        // test customer acct
        let customer_account =
            ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

        // // set test tokens into acct
        // ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(customer_account.bob, 100);

        let initial_bal =
            ink::env::test::get_account_balance::<DefaultEnvironment>(customer_account.bob)
                .expect("no bal");

        assert!(initial_bal == 1000_u128);

        // set caller which is the customer_account in this case
        ink::env::test::set_callee::<ink::env::DefaultEnvironment>(customer_account.bob);

        // assert caller
        assert_eq!(
            ink::env::test::callee::<DefaultEnvironment>(),
            customer_account.bob
        );

        // make order
        let food_items = FoodItem {
            burger_menu: burger_shop::BurgerMenu::ChickenBurger,
            amount: 2,
        };

        ink::env::test::set_value_transferred::<DefaultEnvironment>(30);
        let bob_after = ink::env::test::get_account_balance::<DefaultEnvironment>(customer_account.bob);
        dbg!(bob_after);

        ink::env::test::set_caller::<DefaultEnvironment>(customer_account.alice);

        let alice_initial = ink::env::test::get_account_balance::<DefaultEnvironment>(customer_account.alice);

        dbg!(alice_initial.expect("err"));
        //    assert!(initial_bal == 970_u128);
        ink::env::test::set_value_transferred::<DefaultEnvironment>(30);
        assert_eq!(
            ink::env::test::callee::<DefaultEnvironment>(),
            customer_account.bob
        );
        let alice_after = ink::env::test::get_account_balance::<DefaultEnvironment>(customer_account.alice);
        dbg!(alice_after.expect("err"));
        

        // shop.take_order_and_payment(vec![food_items]).expect("something went wrong");
    }
}

// #[cfg(all(test, feature = "e2e-tests"))]
