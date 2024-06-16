#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod real_estate {
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct RealEstate {
        avg_block_time: u8,
        decimals: u8,
        tax: u8,
        rental_limit_months: u8,
        rental_limit_blocks: u64,
        total_supply: u64,
        total_supply2: u64,
        rent_per_30_day: u64,
        accumulated: u64,
        blocks_per_30_day: u64,
        rental_begin: u64,
        occupied_until: u64,
        tax_deduct: u64,
        name: String,
        symbol: String,
        gov: AccountId,
        main_property_owner: AccountId,
        tenant: AccountId,
        stakeholders: Vec<AccountId>,
        revenues: Mapping<AccountId, u64>,
        shares: Mapping<AccountId, u64>,
        allowed: Mapping<(AccountId, AccountId), u64>,
        rent_paid_until: Mapping<AccountId, u64>,
        shares_offered: Mapping<AccountId, u64>,
        share_sell_price: Mapping<AccountId, u64>,
    }

    #[ink(event)]
    pub struct ShareTransfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        shares: u64,
    }

    #[ink(event)]
    pub struct Seizure {
        #[ink(topic)]
        seized_from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        shares: u64,
    }

    #[ink(event)]
    pub struct ChangedTax {
        new_tax: u64,
    }

    #[ink(event)]
    pub struct MainPropertyOwner {
        new_main_property_owner: AccountId,
    }

    #[ink(event)]
    pub struct NewStakeHolder {
        stakeholder_added: AccountId,
    }

    #[ink(event)]
    pub struct CurrentlyEligibleToPayRent {
        tenant: AccountId,
    }

    #[ink(event)]
    pub struct PrePayRentLimit {
        months: u8,
    }

    #[ink(event)]
    pub struct AvgBlockTimeChangedTo {
        avg_block_time: u8,
    }

    #[ink(event)]
    pub struct RentPer30DaySetTo {
        rent_per_30_day: u64,
    }

    #[ink(event)]
    pub struct StakeHolderBanned {
        banned: AccountId,
    }

    #[ink(event)]
    pub struct RevenuesDistributed {
        shareholder: AccountId,
        gained: u64,
        total: u64,
    }

    #[ink(event)]
    pub struct Withdrawal {
        shareholder: AccountId,
        withdrawn: u64,
    }

    #[ink(event)]
    pub struct Rental {
        date: u64,
        renter: AccountId,
        rent_paid: u64,
        tax: u64,
        distributable_revenue: u64,
        rented_from: u64,
        rented_until: u64,
    }

    #[ink(event)]
    pub struct SharesOffered {
        seller: AccountId,
        amount_shares: u64,
        price_per_share: u64,
    }

    #[ink(event)]
    pub struct SharesSold {
        seller: AccountId,
        buyer: AccountId,
        shares_sold: u64,
        price_per_share: u64,
    }

    impl RealEstate {
        #[ink(constructor)]
        pub fn new(property_id: String, property_symbol: String, main_property_owner: AccountId, tax: u8, avg_block_time: u8) -> Self {
            let gov = Self::env().caller();
            let total_supply = 100;
            let total_supply2 = total_supply.pow(2);
            let blocks_per_30_day = 60 * 60 * 24 * 30 / avg_block_time as u64;

            let mut stakeholders = Vec::new();
            stakeholders.push(gov);
            stakeholders.push(main_property_owner);

            let mut allowed = Mapping::new();
            allowed.insert(&(main_property_owner, gov), &u64::MAX);

            let mut shares = Mapping::new();
            shares.insert(&main_property_owner, &total_supply);

            Self {
                avg_block_time,
                decimals: 0,
                tax,
                rental_limit_months: 12,
                rental_limit_blocks: blocks_per_30_day * 12,
                total_supply,
                total_supply2,
                rent_per_30_day: 0,
                accumulated: 0,
                blocks_per_30_day,
                rental_begin: 0,
                occupied_until: 0,
                tax_deduct: 0,
                name: property_id,
                symbol: property_symbol,
                gov,
                main_property_owner,
                tenant: Default::default(),
                stakeholders,
                revenues: Mapping::new(),
                shares,
                allowed,
                rent_paid_until: Mapping::new(),
                shares_offered: Mapping::new(),
                share_sell_price: Mapping::new(),
            }
        }

        #[ink(message)]
        pub fn show_shares_of(&self, owner: AccountId) -> u64 {
            self.shares.get(&owner).unwrap_or(0)
        }

        #[ink(message)]
        pub fn is_stakeholder(&self, address: AccountId) -> (bool, u64) {
            for (index, stakeholder) in self.stakeholders.iter().enumerate() {
                if *stakeholder == address {
                    return (true, index as u64);
                }
            }
            (false, 0)
        }

        #[ink(message)]
        pub fn current_tenant_check(&self, tenant_check: AccountId) -> (bool, u64) {
            assert!(self.occupied_until == self.rent_paid_until.get(&self.tenant).unwrap_or(0));
            let rent_paid_until = self.rent_paid_until.get(&tenant_check).unwrap_or(0);
            if rent_paid_until > Self::env().block_number() {
                let days_remaining = (rent_paid_until - Self::env().block_number()) * self.avg_block_time as u64 / 86400;
                (true, days_remaining)
            } else {
                (false, 0)
            }
        }

        #[ink(message)]
        pub fn add_stakeholder(&mut self, stakeholder: AccountId) {
            self.only_gov();
            if !self.is_stakeholder(stakeholder).0 {
                self.stakeholders.push(stakeholder);
                self.allowed.insert(&(stakeholder, self.gov), &u64::MAX);
                self.env().emit_event(NewStakeHolder { stakeholder_added: stakeholder });
            }
        }

        #[ink(message)]
        pub fn ban_stakeholder(&mut self, stakeholder: AccountId) {
            self.only_gov();
            if self.is_stakeholder(stakeholder).0 {
                let index = self.is_stakeholder(stakeholder).1 as usize;
                self.stakeholders.remove(index);
                let shares = self.shares.get(&stakeholder).unwrap_or(0);
                self.seizure_from(stakeholder, self.gov, shares);
                self.env().emit_event(StakeHolderBanned { banned: stakeholder });
            }
        }

        #[ink(message)]
        pub fn set_tax(&mut self, tax: u8) {
            self.only_gov();
            assert!(tax <= 100, "Valid tax rate (0% - 100%) required");
            self.tax = tax;
            self.env().emit_event(ChangedTax { new_tax: tax as u64 });
        }

        #[ink(message)]
        pub fn set_avg_block_time(&mut self, seconds_per_block: u8) {
            self.only_gov();
            assert!(seconds_per_block > 0, "Please enter a value above 0");
            self.avg_block_time = seconds_per_block;
            self.blocks_per_30_day = 60 * 60 * 24 * 30 / self.avg_block_time as u64;
            self.env().emit_event(AvgBlockTimeChangedTo { avg_block_time: self.avg_block_time });
        }

        #[ink(message)]
        pub fn distribute(&mut self) {
            self.only_gov();
            let mut accumulated = self.accumulated;
            for stakeholder in &self.stakeholders {
                let shares = self.show_shares_of(*stakeholder);
                let eth_to_receive = (accumulated / self.total_supply) * shares;
                accumulated -= eth_to_receive;
                let revenue = self.revenues.get(stakeholder).unwrap_or(0) + eth_to_receive;
                self.revenues.insert(stakeholder, &revenue);
                self.env().emit_event(RevenuesDistributed {
                    shareholder: *stakeholder,
                    gained: eth_to_receive,
                    total: revenue,
                });
            }
            self.accumulated = accumulated;
        }

        #[ink(message)]
        pub fn seizure_from(&mut self, from: AccountId, to: AccountId, value: u64) -> bool {
            let allowance = self.allowed.get(&(from, self.env().caller())).unwrap_or(0);
            let from_shares = self.shares.get(&from).unwrap_or(0);
            assert!(from_shares >= value && allowance >= value);

            self.shares.insert(&to, &(self.shares.get(&to).unwrap_or(0) + value));
            self.shares.insert(&from, &(from_shares - value));

            if allowance < u64::MAX {
                self.allowed.insert(&(from, self.env().caller()), &(allowance - value));
            }#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod real_estate {
    use ink_storage::collections::HashMap as StorageHashMap;

    #[ink(storage)]
    pub struct RealEstate {
        avg_block_time: u8,
        decimals: u8,
        tax: u8,
        rental_limit_months: u8,
        rental_limit_blocks: u64,
        total_supply: u64,
        total_supply2: u64,
        rent_per_30_day: u128,
        accumulated: u128,
        blocks_per_30_day: u64,
        rental_begin: u64,
        occupied_until: u64,
        tax_deduct: u128,
        name: String,
        symbol: String,
        gov: AccountId,
        main_property_owner: AccountId,
        tenant: AccountId,
        stakeholders: Vec<AccountId>,
        revenues: StorageHashMap<AccountId, u128>,
        shares: StorageHashMap<AccountId, u64>,
        allowed: StorageHashMap<(AccountId, AccountId), u64>,
        rent_paid_until: StorageHashMap<AccountId, u64>,
        shares_offered: StorageHashMap<AccountId, u64>,
        share_sell_price: StorageHashMap<AccountId, u128>,
    }

    #[ink(event)]
    pub struct ShareTransfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        shares: u64,
    }

    #[ink(event)]
    pub struct Seizure {
        #[ink(topic)]
        seized_from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        shares: u64,
    }

    #[ink(event)]
    pub struct ChangedTax {
        new_tax: u8,
    }

    #[ink(event)]
    pub struct MainPropertyOwner {
        new_main_property_owner: AccountId,
    }

    #[ink(event)]
    pub struct NewStakeholder {
        stakeholder_added: AccountId,
    }

    #[ink(event)]
    pub struct CurrentlyEligibleToPayRent {
        tenant: AccountId,
    }

    #[ink(event)]
    pub struct PrePayRentLimit {
        months: u8,
    }

    #[ink(event)]
    pub struct AvgBlockTimeChangedTo {
        seconds: u8,
    }

    #[ink(event)]
    pub struct RentPer30DaySetTo {
        weis: u128,
    }

    #[ink(event)]
    pub struct StakeholderBanned {
        banned: AccountId,
    }

    #[ink(event)]
    pub struct RevenuesDistributed {
        shareholder: AccountId,
        gained: u128,
        total: u128,
    }

    #[ink(event)]
    pub struct Withdrawal {
        shareholder: AccountId,
        withdrawn: u128,
    }

    #[ink(event)]
    pub struct Rental {
        date: u64,
        renter: AccountId,
        rent_paid: u128,
        tax: u128,
        distributable_revenue: u128,
        rented_from: u64,
        rented_until: u64,
    }

    #[ink(event)]
    pub struct SharesOffered {
        seller: AccountId,
        amount_shares: u64,
        price_per_share: u128,
    }

    #[ink(event)]
    pub struct SharesSold {
        seller: AccountId,
        buyer: AccountId,
        shares_sold: u64,
        price_per_share: u128,
    }

    impl RealEstate {
        #[ink(constructor)]
        pub fn new(
            property_id: String,
            property_symbol: String,
            main_property_owner: AccountId,
            tax: u8,
            avg_block_time: u8,
        ) -> Self {
            let mut shares = StorageHashMap::new();
            shares.insert(main_property_owner, 100);

            let mut stakeholders = Vec::new();
            let gov = Self::env().caller();
            stakeholders.push(gov);
            stakeholders.push(main_property_owner);

            let mut allowed = StorageHashMap::new();
            allowed.insert((main_property_owner, gov), u64::MAX);

            let blocks_per_30_day = 60 * 60 * 24 * 30 / avg_block_time as u64;
            let rental_limit_blocks = 12 * blocks_per_30_day;

            Self {
                avg_block_time,
                decimals: 0,
                tax,
                rental_limit_months: 12,
                rental_limit_blocks,
                total_supply: 100,
                total_supply2: 100 * 100,
                rent_per_30_day: 0,
                accumulated: 0,
                blocks_per_30_day,
                rental_begin: 0,
                occupied_until: 0,
                tax_deduct: 0,
                name: property_id,
                symbol: property_symbol,
                gov,
                main_property_owner,
                tenant: Default::default(),
                stakeholders,
                revenues: StorageHashMap::new(),
                shares,
                allowed,
                rent_paid_until: StorageHashMap::new(),
                shares_offered: StorageHashMap::new(),
                share_sell_price: StorageHashMap::new(),
            }
        }

        #[ink(message)]
        pub fn show_shares_of(&self, owner: AccountId) -> u64 {
            *self.shares.get(&owner).unwrap_or(&0)
        }

        #[ink(message)]
        pub fn is_stakeholder(&self, address: AccountId) -> (bool, usize) {
            for (index, stakeholder) in self.stakeholders.iter().enumerate() {
                if *stakeholder == address {
                    return (true, index);
                }
            }
            (false, 0)
        }

        #[ink(message)]
        pub fn current_tenant_check(&self, tenant_check: AccountId) -> (bool, u64) {
            let rent_paid_until = *self.rent_paid_until.get(&tenant_check).unwrap_or(&0);
            if self.occupied_until == rent_paid_until {
                if rent_paid_until > self.env().block_number() {
                    let days_remaining = (rent_paid_until - self.env().block_number())
                        * self.avg_block_time as u64
                        / 86400;
                    return (true, days_remaining);
                }
            }
            (false, 0)
        }

        #[ink(message)]
        #[ink(keep_attr = "only_gov")]
        pub fn add_stakeholder(&mut self, stakeholder: AccountId) {
            let (is_stakeholder, _) = self.is_stakeholder(stakeholder);
            if !is_stakeholder {
                self.stakeholders.push(stakeholder);
                self.allowed.insert((stakeholder, self.gov), u64::MAX);
                self.env().emit_event(NewStakeholder {
                    stakeholder_added: stakeholder,
                });
            }
        }

        #[ink(message)]
        #[ink(keep_attr = "only_gov")]
        pub fn ban_stakeholder(&mut self, stakeholder: AccountId) {
            let (is_stakeholder, index) = self.is_stakeholder(stakeholder);
            if is_stakeholder {
                self.stakeholders.swap_remove(index);
                let shares = *self.shares.get(&stakeholder).unwrap_or(&0);
                self.seizure_from(stakeholder, self.gov, shares);
                self.env().emit_event(StakeholderBanned { banned: stakeholder });
            }
        }

        #[ink(message)]
        #[ink(keep_attr = "only_gov")]
        pub fn set_tax(&mut self, x: u8) {
            assert!(x <= 100, "Valid tax rate (0% - 100%) required");
            self.tax = x;
            self.env().emit_event(ChangedTax { new_tax: x });
        }

        #[ink(message)]
        #[ink(keep_attr = "only_gov")]
        pub fn set_avg_block_time(&mut self, s_per_block: u8) {
            assert!(s_per_block > 0, "Please enter a value above 0");
            self.avg_block_time = s_per_block;
            self.blocks_per_30_day = (60 * 60 * 24 * 30) / s_per_block as u64;
            self.env().emit_event(AvgBlockTimeChangedTo {
                seconds: s_per_block,
            });
        }

        #[ink(message)]
        #[ink(keep_attr = "only_gov")]
        pub fn distribute(&mut self) {
            let accumulated = self.accumulated;
            for stakeholder in &self.stakeholders {
                let shares = self.show_shares_of(*stakeholder);
                let eth_to_receive = (accumulated / self.total_supply) * shares as u128;
                self.accumulated -= eth_to_receive;
                *self.revenues.entry(*stakeholder).or_insert(0) += eth_to_receive;
                self.env().emit_event(RevenuesDistributed {
                    shareholder: *stakeholder,
                    gained: eth_to_receive,
                    total: *self.revenues.get(stakeholder).unwrap_or(&0),
                });
            }
        }

        #[ink(message)]
        pub fn seizure_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: u64,
        ) -> bool {
            let allowance = *self.allowed.get(&(from, self.env().caller())).unwrap_or(&0);
            let from_balance = *self.shares.get(&from).unwrap_or(&0);

            if from_balance >= value && allowance >= value {
                *self.shares.entry(from).or_insert(0) -= value;
                *self.shares.entry(to).or_insert(0)*self.shares.entry(to).or_insert(0) += value;
                self.allowed.insert((from, self.env().caller()), allowance - value);
                self.env().emit_event(Seizure {
                    seized_from: Some(from),
                    to: Some(to),
                    shares: value,
                });
                return true;
            }
            false
        }

        #[ink(message)]
        #[ink(keep_attr = "only_gov")]
        pub fn set_rent_per_30_day(&mut self, rent: u128) {
            self.rent_per_30_day = rent;
            self.env().emit_event(RentPer30DaySetTo { weis: rent });
        }

        #[ink(message)]
        pub fn pay_rent(&mut self) {
            let caller = self.env().caller();
            let rent_paid_until = *self.rent_paid_until.get(&caller).unwrap_or(&0);
            let current_block = self.env().block_number();
            assert!(current_block < rent_paid_until + self.rental_limit_blocks, "Rent cannot be prepaid for more than the rental limit");

            let rent_duration = rent_paid_until.saturating_sub(current_block);
            let rent_to_pay = self.rent_per_30_day * rent_duration as u128 / self.blocks_per_30_day as u128;
            assert!(self.env().transferred_balance() >= rent_to_pay, "Insufficient rent payment");

            let tax_amount = rent_to_pay * self.tax as u128 / 100;
            let distributable_revenue = rent_to_pay - tax_amount;
            self.accumulated += distributable_revenue;
            self.tenant = caller;
            self.rental_begin = current_block;
            self.occupied_until = rent_paid_until + rent_duration;

            self.env().emit_event(Rental {
                date: current_block,
                renter: caller,
                rent_paid: rent_to_pay,
                tax: tax_amount,
                distributable_revenue,
                rented_from: current_block,
                rented_until: self.occupied_until,
            });
        }

        #[ink(message)]
        pub fn offer_shares_for_sale(&mut self, amount_shares: u64, price_per_share: u128) {
            let caller = self.env().caller();
            let caller_shares = self.show_shares_of(caller);
            assert!(caller_shares >= amount_shares, "Insufficient shares to sell");

            self.shares_offered.insert(caller, amount_shares);
            self.share_sell_price.insert(caller, price_per_share);

            self.env().emit_event(SharesOffered {
                seller: caller,
                amount_shares,
                price_per_share,
            });
        }

        #[ink(message)]
        pub fn buy_shares(&mut self, seller: AccountId, amount_shares: u64) {
            let buyer = self.env().caller();
            let sell_price = *self.share_sell_price.get(&seller).unwrap_or(&0);
            let total_price = sell_price * amount_shares as u128;
            assert!(self.env().transferred_balance() >= total_price, "Insufficient payment");

            let seller_shares = self.show_shares_of(seller);
            assert!(seller_shares >= amount_shares, "Seller does not have enough shares");

            *self.shares.entry(seller).or_insert(0) -= amount_shares;
            *self.shares.entry(buyer).or_insert(0) += amount_shares;

            self.env().emit_event(SharesSold {
                seller,
                buyer,
                shares_sold: amount_shares,
                price_per_share: sell_price,
            });
        }

        #[ink(message)]
        pub fn withdraw_revenue(&mut self) {
            let caller = self.env().caller();
            let revenue = *self.revenues.get(&caller).unwrap_or(&0);
            assert!(revenue > 0, "No revenue to withdraw");

            self.revenues.insert(caller, 0);
            self.env().transfer(caller, revenue).expect("Transfer failed");

            self.env().emit_event(Withdrawal {
                shareholder: caller,
                withdrawn: revenue,
            });
        }
    }
}

