#[macro_use]
extern crate text_io;

use std::cmp::min;
use std::collections::BinaryHeap;
use std::collections::VecDeque;

mod orders;

use orders::{order_from_str, Order, Side, Trade};

fn main() {
    let mut asks = BinaryHeap::<Order>::new();
    let mut bids = BinaryHeap::<Order>::new();
    loop {
        let line: String = read!("{}\n");
        match order_from_str(&line) {
            Ok(new_order) => {
                let trades = execute_order(&mut asks, &mut bids, new_order);
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

fn execute_order(
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
    // move this part out of executing strategy function
    // have different strategies for GTC, FOK or IOC
    // GTC - pass trades through, add order (as below)
    // FOK - if not filled, discard trades (how to undo changes in orders?)
    // - "order validation" could do this before executing strategy.
    // IoC - pass trades through, cancel order if amount > 0
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
        let mut asks = BinaryHeap::from(vec![order_from_str("4 10 10 1 0").unwrap()]);
        let mut bids = BinaryHeap::<Order>::new();

        let trades = execute_order(&mut asks, &mut bids, order_from_str("8 10 10 3 0").unwrap());

        assert_eq!(asks.into_sorted_vec(), []);
        assert_eq!(bids.into_sorted_vec(), []);
        assert_has_one_trade(trades, 10, 10);
    }

    #[test]
    fn test_cross_order_ask() {
        let mut asks = BinaryHeap::<Order>::new();
        let mut bids = BinaryHeap::from(vec![order_from_str("8 10 10 1 0").unwrap()]);

        let trades = execute_order(&mut asks, &mut bids, order_from_str("4 10 10 5 0").unwrap());

        assert_eq!(asks.into_sorted_vec(), []);
        assert_eq!(bids.into_sorted_vec(), []);
        assert_has_one_trade(trades, 10, 10);
    }

    #[test]
    fn test_cheaper_ask_comes_in() {
        let mut asks = BinaryHeap::<Order>::new();
        let mut bids = BinaryHeap::from(vec![order_from_str("8 10 10 1 0").unwrap()]);

        let trades = execute_order(&mut asks, &mut bids, order_from_str("4 10 5 3 0").unwrap());

        assert_eq!(asks.into_sorted_vec(), []);
        assert_eq!(bids.into_sorted_vec(), []);
        assert_has_one_trade(trades, 10, 10);
    }

    #[test]
    fn test_market_order_buys_all() {
        let mut asks = BinaryHeap::from(vec![
            order_from_str("4 10 10 1 0").unwrap(),
            order_from_str("4 10 20 1 0").unwrap(),
        ]);
        let mut bids = BinaryHeap::<Order>::new();

        let trades = execute_order(&mut asks, &mut bids, order_from_str("8 20 0 3 1").unwrap());

        assert_eq!(asks.into_sorted_vec(), []);
        assert_eq!(bids.into_sorted_vec(), []);
        assert_eq!(trades.get(0).unwrap().amount, 10);
        assert_eq!(trades.get(0).unwrap().price, 10);
        assert_eq!(trades.get(1).unwrap().amount, 10);
        assert_eq!(trades.get(1).unwrap().price, 20);
    }

    fn assert_has_one_trade(trades: VecDeque<Trade>, amount: i32, price: i32) {
        assert_eq!(trades.len(), 1);
        let only_trade = trades.front().unwrap();
        assert_eq!(only_trade.amount, amount);
        assert_eq!(only_trade.price, price);
    }
}
