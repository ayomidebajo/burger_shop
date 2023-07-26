#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod burger_shop {

    use ink::{prelude::vec::Vec, env::call};
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
        list_of_items: Vec<FoodItem>,
        customer: AccountId,
        total_price: Balance,
        paid: bool,
        status: Status,
    }

    impl Order {
        fn new(list_of_items: Vec<FoodItem>, customer: AccountId) -> Self {
            let total_price = Order::total_price(&list_of_items);
            Self {
                list_of_items,
                customer,
                total_price,
                paid: false,
                status: Default::default(), // Default is getting ingredients in this case
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
                Self::CheeseBurger => 120,
                Self::VeggieBurger => 100,
                Self::ChickenBurger => 150,
            }
        }
    }

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

    // result type
    pub type Result<T> = core::result::Result<T, BurgerShopError>;

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
        #[ink(constructor)]
        pub fn new() -> Self {
            let shop_owner = Self::env().caller();
            let order_storage_vector: Vec<(u32, Order)> = Vec::new();
            let order_storage_mapping = Mapping::new();

            Self {
                orders: order_storage_vector,
                orders_mapping: order_storage_mapping,
                shop_owner,
            }
        }

        #[ink(message)]
        pub fn take_order(&self, customer_id: AccountId, list_of_items: Vec<FoodItem>) -> Order {
            let total_price = Order::total_price(&list_of_items);
            let mut order = Order::new(list_of_items, customer_id);
            order.total_price = total_price;

            order
        }

        #[ink(message)]
        pub fn accept_payment(&self, order: Order) -> Result<()> {
            let caller = Self::env().caller();
            // make sure the order hasn't been paid for already
            assert!(order.paid == false, "Can't pay for an order that is paid for already");

            
            // make sure the caller is the customer which already in the Order struct
            assert!(caller == order.customer, "You are not the customer!");

            // make sure the status hasn't been delivered
            assert!(order.status == Status::GettingIngredients, "invalid order, this is already in progress or already completed");

            // make payment
            match self.env().transfer(self.shop_owner, order.total_price) {
                Ok(_) => todo!(),
                Err(_) => todo!()
            }
            
        } 

        #[ink(message)]
        pub fn foo(&self) -> bool {
            true
        }
    }
}
