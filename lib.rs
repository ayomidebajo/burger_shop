#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod burger_shop {

    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use scale::{Decode, Encode};

    // this is the main contract, this is what gets instantiated
    #[ink(storage)]
    pub struct BurgerShop {
        orders: Vec<(u32, Order)>,
        orders_mapping: Mapping<u32, Order>,
        shop_owner: AccountId,
    }

    #[derive(Encode, Decode, Debug, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Order {
        burger_menu: BurgerMenu,
        customer: AccountId,
        price: u32,
        amount: u32,
        paid: bool,
        delivered: bool,
        status: Status,
        completed: bool,
    }

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



    impl Order {
        pub fn new(
            burger_menu: BurgerMenu,
            customer: AccountId,
            price: u32,
            amount: u32,
            paid: bool,
            delivered: bool,
            status: Status,
            completed: bool,
        ) -> Self {

            let mut new_order = Order {
                burger_menu,
                customer,
                price,
                amount,
                paid,
                delivered,
                status,
                completed,
            };

           new_order.set_price();

           new_order
        }

        pub fn get_burger_menu(&self) -> BurgerMenu {
            self.burger_menu.clone()
        }

        fn set_price(&mut self)  {
            match self.burger_menu {
                BurgerMenu::CheeseBurger => self.price = 100,
                BurgerMenu::ChickenBurger => self.price = 150,
                BurgerMenu::VeggieBurger => self.price = 120,
            }
        }

        pub fn get_price(&self) -> u32 {
            self.price
        }
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
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

    // result type
    pub type Result<T> = core::result::Result<T, Error>;

    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    #[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
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
        #[ink(constructor)] // this is the constructor, it gets called when the contract is instantiated
        pub fn new() -> Self {
            let list_of_orders = Mapping::new();
            let new_vec = Vec::new();
            Self {
                orders: new_vec,
                orders_mapping: list_of_orders,
                shop_owner: Self::env().caller(),
            }
        }

        /// A message that can be called on instantiated contracts.
        /// This one gets the value of the `message` state value.
        #[ink(message)] // this get after the contract is instantiated, it can be called multiple times
        pub fn get_orders(&self) -> Vec<(u32, Order)> {
            // ensure the call is from the shop owner
            let caller = self.env().caller();
            assert!(caller == self.shop_owner, "You are not the shop owner!");
            self.orders.clone()
        }

        #[ink(message)]
        pub fn new_order(&mut self, order: Order) {
            let count = self.orders.len() as u32;
            // ensure order is from the customer
            let caller = self.env().caller();
            assert!(caller == order.customer, "You are not the customer!");

            // ensure that the order is not completed
            assert!(
                order.completed == false,
                "Cant add an already completed order!"
            );

            self.orders_mapping.insert(count, &order);

            self.orders.push((count, order));
        }

        #[ink(message)]
        pub fn get_single_order(&self, id: u32) -> (u32, Order) {
            let tuple_order = (id, self.orders_mapping.get(id).expect("Order not found!"));
            tuple_order
        }

        #[ink(message)]
        pub fn mark_completed(&mut self, id: u32) -> Result<()> {
            let mut order = self.orders_mapping.get(id).expect("order not found!");

            assert!(order.completed == false, "Order already completed!");
            assert!(order.paid == true, "Order not paid!");
            assert!(order.delivered == true, "Order not delivered!");
            assert!(order.status == Status::Delivered, "Order not delivered!");

            if order.paid && order.delivered && order.status == Status::Delivered {
                order.completed = true;
                Ok(())
            } else {
                Err(Error::OrderNotCompleted)
            }
        }

        #[ink(message)]
        pub fn make_payment(&mut self, id: u32) -> Result<()> {
            let mut order = self.orders_mapping.get(id).expect("order not found!");
            assert!(order.paid == false, "Order already paid!");

            // ensure that sender is the caller
            let caller = self.env().caller();
            assert!(caller == order.customer, "You are not the customer!");

            // ensure that the order is not completed
            assert!(order.completed == false, "Order already completed!");

            // ensure that the order is not delivered
            assert!(order.delivered == false, "Order already delivered!");

            // make payment
            match self.env().transfer(self.shop_owner, order.price.into()) {
                Ok(_) => {
                    order.paid = true;
                    order.status = Status::Preparing;
                    Ok(())
                }
                Err(_) => Err(Error::PaymentError),
            }
        }

        #[ink(message)]
        pub fn change_status(&mut self, id: u32, status: Status) {
            let mut order = self.orders_mapping.get(id).expect("order not found!");

            // ensure caller is the shop owner
            let caller = self.env().caller();
            assert!(caller == self.shop_owner, "You are not the shop owner!");

            // ensure that the order is not completed
            assert!(order.completed == false, "Order already completed!");

            order.status = status;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn it_works() {
            let mut burger_shop = BurgerShop::new();
            let order = Order {
                burger_menu: BurgerMenu::CheeseBurger,
                customer: AccountId::from([0x01; 32]),
                price: 100,
                amount: 1,
                paid: false,
                delivered: false,
                status: Status::GettingIngredients,
                completed: false,
            };

            burger_shop.new_order(order.clone());

            let orders = burger_shop.get_orders();
            assert_eq!(orders.len(), 1);
        }

        #[ink::test]
        fn test_make_payment() {
            let mut burger_shop = BurgerShop::new();
            let order = Order {
                burger_menu: BurgerMenu::CheeseBurger,
                customer: AccountId::from([0x01; 32]),
                price: 100,
                amount: 1,
                paid: false,
                delivered: false,
                status: Status::GettingIngredients,
                completed: false,
            };

            burger_shop.new_order(order.clone());

            burger_shop.make_payment(0).unwrap();

            let orders = burger_shop.get_orders();
            assert_eq!(orders.len(), 1);
            assert_eq!(orders[0].1.paid, true);
        }
        #[ink::test]
        fn test_change_status() {
            let mut burger_shop = BurgerShop::new();
            let order = Order {
                burger_menu: BurgerMenu::CheeseBurger,
                customer: AccountId::from([0x01; 32]),
                price: 100,
                amount: 1,
                paid: false,
                delivered: false,
                status: Status::GettingIngredients,
                completed: false,
            };

            burger_shop.new_order(order.clone());
            let orders = burger_shop.get_orders();
            assert_eq!(orders.len(), 1);
            burger_shop.change_status(0, Status::Preparing);

            assert_eq!(orders[0].1.status, Status::Preparing);
        }

        #[ink::test]
        fn test_mark_completed() {
            let mut burger_shop = BurgerShop::new();
            let order = Order {
                burger_menu: BurgerMenu::CheeseBurger,
                customer: AccountId::from([0x01; 32]),
                price: 100,
                amount: 1,
                paid: false,
                delivered: false,
                status: Status::GettingIngredients,
                completed: false,
            };

            burger_shop.new_order(order.clone());

            burger_shop.make_payment(0).unwrap();
            burger_shop.change_status(0, Status::Preparing);
            burger_shop.change_status(0, Status::SentForDelivery);
            burger_shop.change_status(0, Status::Delivered);
            burger_shop.mark_completed(0).unwrap();

            let orders = burger_shop.get_orders();
            assert_eq!(orders.len(), 1);
            assert_eq!(orders[0].1.completed, true);
        }
    }
}
