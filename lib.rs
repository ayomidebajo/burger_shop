#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod burger_shop {

    use ink::prelude::string::String;
    use ink::prelude::vec;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use scale::{Decode, Encode};

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct BurgerShop {
        /// Stores a single `bool` value on the storage.
        orders: Vec<(u32, Order)>,
        orders_mapping: Mapping<u32, Order>,
    }

    // TODO: add logic for payment to the shop
    // TODO: add logic to calculate gas fees for user

    #[derive(Encode, Decode, Debug, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Order {
        burger_menu: String,
        customer: AccountId,
        price: u32,
        amount: u32,
        paid: bool,
        delivered: bool,
        status: Status,
        completed: bool,
    }

    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    #[derive(Encode, Decode, Debug, Clone)]
    pub enum Status {
        GettingIngredients,
        Preparing,
        SentForDelivery,
        Delivered,
    }

    impl Default for Status {
        fn default() -> Self {
            Status::GettingIngredients
        }
    }

    impl BurgerShop {
        #[ink(constructor)]
        pub fn new() -> Self {
            let list_of_orders = Mapping::new();
            Self {
                orders: vec![],
                orders_mapping: list_of_orders,
            }
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn get_orders(&self) -> Vec<(u32, Order)> {
            self.orders.clone()
        }

        #[ink(message)]
        pub fn new_order(&mut self, order: Order) {
            let count = self.orders.len() as u32;
            self.orders_mapping.insert(count, &order);
            self.orders.push((count, order));
        }

        #[ink(message)]
        pub fn get_single_order(&self, id: u32) -> (u32, Order) {
            let tuple_order = (id, self.orders_mapping.get(id).expect("Order not found!"));
            tuple_order
        }

        #[ink(message)]
        pub fn mark_completed(&mut self, id: u32) {
            let mut order = self.orders_mapping.get(id).expect("order not found!");
            if order.paid {
                order.completed = true
            }
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        // #[ink::test]
        // fn default_works() {
        //     let burger_shop = BurgerShop::default();
        //     assert_eq!(burger_shop.get(), false);
        // }

        // /// We test a simple use case of our contract.
        // #[ink::test]
        // fn it_works() {
        //     let mut burger_shop = BurgerShop::new(false);
        //     assert_eq!(burger_shop.get(), false);
        //     burger_shop.flip();
        //     assert_eq!(burger_shop.get(), true);
        // }
    }

    // / This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    // /
    // / When running these you need to make sure that you:
    // / - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    // / - Are running a Substrate node which contains `pallet-contracts` in the background
    // #[cfg(all(test, feature = "e2e-tests"))]
    // mod e2e_tests {
    //     /// Imports all the definitions from the outer scope so we can use them here.
    //     use super::*;

    //     /// A helper function used for calling contract messages.
    //     use ink_e2e::build_message;

    //     /// The End-to-End test `Result` type.
    //     type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    //     /// We test that we can upload and instantiate the contract using its default constructor.
    //     #[ink_e2e::test]
    //     async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    //         // Given
    //         let constructor = BurgerShopRef::default();

    //         // When
    //         let contract_account_id = client
    //             .instantiate("burger_shop", &ink_e2e::alice(), constructor, 0, None)
    //             .await
    //             .expect("instantiate failed")
    //             .account_id;

    //         // Then
    //         let get = build_message::<BurgerShopRef>(contract_account_id.clone())
    //             .call(|burger_shop| burger_shop.get());
    //         let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
    //         assert!(matches!(get_result.return_value(), false));

    //         Ok(())
    //     }

    //     /// We test that we can read and write a value from the on-chain contract contract.
    //     #[ink_e2e::test]
    //     async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    //         // Given
    //         let constructor = BurgerShopRef::new(false);
    //         let contract_account_id = client
    //             .instantiate("burger_shop", &ink_e2e::bob(), constructor, 0, None)
    //             .await
    //             .expect("instantiate failed")
    //             .account_id;

    //         let get = build_message::<BurgerShopRef>(contract_account_id.clone())
    //             .call(|burger_shop| burger_shop.get());
    //         let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
    //         assert!(matches!(get_result.return_value(), false));

    //         // When
    //         let flip = build_message::<BurgerShopRef>(contract_account_id.clone())
    //             .call(|burger_shop| burger_shop.flip());
    //         let _flip_result = client
    //             .call(&ink_e2e::bob(), flip, 0, None)
    //             .await
    //             .expect("flip failed");

    //         // Then
    //         let get = build_message::<BurgerShopRef>(contract_account_id.clone())
    //             .call(|burger_shop| burger_shop.get());
    //         let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
    //         assert!(matches!(get_result.return_value(), true));

    //         Ok(())
    //     }
    // }
}
