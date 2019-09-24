#[macro_use]
extern crate text_io;

use std::cmp::min;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::VecDeque;
use std::str::FromStr;

#[derive(Copy, Clone, PartialEq, Debug)]
enum Side {
    Bid,
    Ask,
}

#[derive(Copy, Clone, Debug)]
struct Trade {
    executing_order_id: i32,
    matched_order_id: i32,
    timestamp: u128,
    amount: i32,
    price: i32,
}

#[derive(Copy, Clone, Debug)]
struct Order {
    side: Side,
    amount: i32,
    price: i32,
    timestamp: i32,
}

impl Order {
    fn matches(&self, other: &Self) -> bool {
        return (self.side == Side::Bid && self.price >= other.price)
            || (self.side == Side::Ask && self.price <= other.price);
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
        let (side_int, amount, price, timestamp): (i32, i32, i32, i32);
        scan!(raw_str.bytes() => "{} {} {} {}", side_int, amount, price, timestamp);
        let side;
        if side_int == 4 {
            side = Side::Ask;
        } else if side_int == 8 {
            side = Side::Bid;
        } else {
            panic!("Invalid side")
        }
        Ok(Self {
            side: side,
            amount: amount,
            price: price,
            timestamp: timestamp,
        })
    }
}

fn main() {
    let mut asks = BinaryHeap::<Order>::new();
    let mut bids = BinaryHeap::<Order>::new();
    loop {
        let line: String = read!("{}\n");
        match Order::from_str(&line) {
            Ok(new_order) => {
                let trades = execute_limit_order(&mut asks, &mut bids, new_order);
                println!("Trades generated: {:?}", trades);
                for order in asks.iter() {
                    println!("Ask: {:?}", order);
                }
                for order in bids.iter() {
                    println!("Ask: {:?}", order);
                }
            }
            Err(_) => {
                println!("Couldn't parse input: '{}'", line);
            }
        }
    }
}

fn execute_limit_order(
    asks: &mut BinaryHeap<Order>,
    bids: &mut BinaryHeap<Order>,
    mut new_order: Order,
) -> (VecDeque<Trade>) {
    // TODO: order executing strategies: LIMIT, MARKET, STOP
    // TODO: time in force - GTC, FOK, IOC
    let (same_side, other_side) = if new_order.side == Side::Bid {
        (bids, asks)
    } else {
        (asks, bids)
    };
    let mut trades = VecDeque::<Trade>::new();

    while new_order.amount > 0
        && other_side.peek() != None
        && new_order.matches(&(other_side.peek().unwrap()))
    {
        let matched_order = other_side.peek().unwrap();
        let matched_amount = min(new_order.amount, matched_order.amount);
        let price = matched_order.price;
        new_order.amount -= matched_amount;
        // if other order is filled, remove it
        if matched_amount == matched_order.amount {
            let ask_to_delete = other_side.pop();
            println!("Filled! {:?}", ask_to_delete);
        } else {
            // otherwise, lower amount only
            other_side.peek_mut().unwrap().amount -= matched_amount;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("WTF");
        trades.push_back(Trade {
            executing_order_id: 1,
            matched_order_id: 1,
            timestamp: now.as_nanos(),
            amount: matched_amount,
            price: price,
        });
    }
    if new_order.amount > 0 {
        // IoC wouldn't add it
        println!("Pushing to same side {:?}", new_order);
        same_side.push(new_order);
    } else {
        println!("Filled! {:?}", new_order);
    }

    trades
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_order_bid() {
        let mut asks = BinaryHeap::from(vec![Order {
            side: Side::Ask,
            amount: 10,
            price: 10,
            timestamp: 1,
        }]);
        let mut bids = BinaryHeap::<Order>::new();

        let trades = execute_limit_order(
            &mut asks,
            &mut bids,
            Order {
                side: Side::Bid,
                amount: 10,
                price: 10,
                timestamp: 1,
            },
        );

        assert_eq!(asks.into_sorted_vec(), []);
        assert_eq!(bids.into_sorted_vec(), []);
        assert_has_one_trade(trades, 10, 10);
    }

    #[test]
    fn test_cross_order_ask() {
        let mut asks = BinaryHeap::<Order>::new();
        let mut bids = BinaryHeap::from(vec![Order {
            side: Side::Bid,
            amount: 10,
            price: 10,
            timestamp: 1,
        }]);

        let trades = execute_limit_order(
            &mut asks,
            &mut bids,
            Order {
                side: Side::Ask,
                amount: 10,
                price: 10,
                timestamp: 1,
            },
        );

        assert_eq!(asks.into_sorted_vec(), []);
        assert_eq!(bids.into_sorted_vec(), []);
        assert_has_one_trade(trades, 10, 10);
    }

    #[test]
    fn test_cheaper_ask_comes_in() {
        let mut asks = BinaryHeap::<Order>::new();
        let mut bids = BinaryHeap::from(vec![Order {
            side: Side::Bid,
            amount: 10,
            price: 10,
            timestamp: 1,
        }]);

        let trades = execute_limit_order(
            &mut asks,
            &mut bids,
            Order {
                side: Side::Ask,
                amount: 10,
                price: 5,
                timestamp: 1,
            },
        );

        assert_eq!(asks.into_sorted_vec(), []);
        assert_eq!(bids.into_sorted_vec(), []);
        assert_has_one_trade(trades, 10, 10);
    }

    fn assert_has_one_trade(trades: VecDeque<Trade>, amount: i32, price: i32) {
        assert_eq!(trades.len(), 1);
        let only_trade = trades.front().unwrap();
        assert_eq!(only_trade.amount, amount);
        assert_eq!(only_trade.price, price);
    }

    #[test]
    fn test_order_from_str_bid() {
        assert_eq!(
            Order::from_str("8 1 2 0"),
            Ok(Order {
                side: Side::Bid,
                amount: 1,
                price: 2,
                timestamp: 0
            })
        );
    }

    #[test]
    fn test_order_from_str_ask() {
        assert_eq!(
            Order::from_str("4 9 1 2"),
            Ok(Order {
                side: Side::Bid,
                amount: 9,
                price: 1,
                timestamp: 2
            })
        );
    }
}
