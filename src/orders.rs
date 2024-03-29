use std::cmp::Ordering;
use std::str::FromStr;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Copy, Clone, Debug)]
pub struct Trade {
    pub executing_order_id: i32,
    pub matched_order_id: i32,
    pub timestamp: u128,
    pub amount: i32,
    pub price: i32,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Strategy {
    Limit,
    Market,
}

impl Strategy {
    fn matches(&self, this_order: &Order, other: &Order) -> bool {
        if *self == Strategy::Limit {
            return (this_order.side == Side::Bid && this_order.price >= other.price)
                || (this_order.side == Side::Ask && this_order.price <= other.price);
        } else if *self == Strategy::Market {
            return true;
        } else {
            panic!("Noooo");
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Order {
    pub side: Side,
    pub amount: i32,
    pub price: i32,
    pub timestamp: i32,
    strategy: Strategy,
}

impl Order {
    pub fn matches(&self, other: &Self) -> bool {
        return self.strategy.matches(self, other);
    }
}

impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        let multiplier;
        if self.side == Side::Ask {
            multiplier = -1;
        } else {
            multiplier = 1;
        }
        (self.price, self.timestamp).cmp(&((other.price * multiplier), other.timestamp))
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.timestamp == other.timestamp
    }
}

impl Eq for Order {}

impl FromStr for Order {
    type Err = std::num::ParseIntError;

    fn from_str(raw_str: &str) -> Result<Self, Self::Err> {
        let (side_int, amount, price, timestamp, strategy_int): (i32, i32, i32, i32, i32);
        scan!(raw_str.bytes() => "{} {} {} {} {}", side_int, amount, price, timestamp, strategy_int);
        let side;
        if side_int == 4 {
            side = Side::Ask;
        } else if side_int == 8 {
            side = Side::Bid;
        } else {
            panic!("Invalid side");
        }
        let strategy;
        if strategy_int == 0 {
            strategy = Strategy::Limit;
        } else if strategy_int == 1 {
            strategy = Strategy::Market;
        } else {
            panic!("Invalid strategy");
        }

        Ok(Self {
            side: side,
            amount: amount,
            price: price,
            timestamp: timestamp,
            strategy: strategy,
        })
    }
}

pub fn order_from_str(raw_str: &str) -> Result<Order, <Order as FromStr>::Err> {
    return Order::from_str(raw_str);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_from_str_bid() {
        assert_eq!(
            order_from_str("8 1 2 0 0"),
            Ok(Order {
                side: Side::Bid,
                amount: 1,
                price: 2,
                timestamp: 0,
                strategy: Strategy::Limit
            })
        );
    }

    #[test]
    fn test_order_from_str_ask() {
        assert_eq!(
            order_from_str("4 9 1 2 0"),
            Ok(Order {
                side: Side::Bid,
                amount: 9,
                price: 1,
                timestamp: 2,
                strategy: Strategy::Limit
            })
        );
    }

    #[test]
    fn test_order_matches() {
        let executing_order = order_from_str("8 1 1 5 0").unwrap();
        let other_order = order_from_str("4 1 1 3 0").unwrap();
        assert!(executing_order.matches(&other_order));
    }
}
