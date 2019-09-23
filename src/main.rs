#[macro_use]
extern crate intrusive_collections;

use std::cmp::min;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Copy, Clone, PartialEq, Debug)]
enum Side {
	Bid,
	Ask
}

#[derive(Copy, Clone, Debug)]
struct Order {
	side: Side,
	amount: i32,
	price: i32,
	timestamp: i32
}

impl Order {
	fn matches(&self, other: &Self) -> bool {
		return (self.side == Side::Bid && self.price >= other.price) || (self.side == Side::Ask && self.price <= other.price);
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


fn main() {
	let mut asks = BinaryHeap::<Order>::new();
	asks.push(Order{ side: Side::Ask, amount: 200, price: 2030, timestamp: 905 });
	asks.push(Order{ side: Side::Ask, amount: 100, price: 2030, timestamp: 901 });
	asks.push(Order{ side: Side::Ask, amount: 100, price: 2025, timestamp: 903 });
	println!("Top ask: {:?}", asks.peek().unwrap());

	let bids = BinaryHeap::<Order>::new();

	let (asks, bids) = match_orders(asks, bids, Order{ side: Side::Bid, amount: 250, price: 2035, timestamp: 908 });
	let (asks, bids) = match_orders(asks, bids, Order{ side: Side::Ask, amount: 250, price: 2035, timestamp: 908 });

	println!("Asks after: {:?}", asks.into_sorted_vec());
	println!("Bids after: {:?}", bids.into_sorted_vec());
}

fn match_orders(asks: BinaryHeap::<Order>, bids: BinaryHeap::<Order>, mut new_order: Order) -> (BinaryHeap::<Order>, BinaryHeap::<Order>) {
	// TODO: return trades
	// TODO: order executing strategies: LIMIT, MARKET, STOP
	// TODO: time in force - GTC, FOK, IOC
	let (mut same_side, mut other_side) = if new_order.side == Side::Bid {
		(bids, asks)
	} else {
		(asks, bids)
	};

	while new_order.amount > 0 && other_side.peek() != None && new_order.matches(&(other_side.peek().unwrap())) {
		let amount_delta = min(new_order.amount, other_side.peek().unwrap().amount);
		new_order.amount -= amount_delta;
		// if other order is filled, remove it
		if amount_delta == other_side.peek().unwrap().amount {
			let ask_to_delete = other_side.pop();
			println!("Filled! {:?}", ask_to_delete);
		} else { // otherwise, lower amount only
			other_side.peek_mut().unwrap().amount -= amount_delta;
		}
	}
	if new_order.amount > 0 {
		println!("Pushing to same side {:?}", new_order);
		same_side.push(new_order);
	} else {
		println!("Filled! {:?}", new_order);
	}

	if new_order.side == Side::Bid {
		(other_side, same_side)
	} else {
		(same_side, other_side)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_cross_order_bid() {
		let asks = BinaryHeap::from(vec![Order{ side: Side::Ask, amount: 10, price: 10, timestamp: 1 }]);
		let bids = BinaryHeap::<Order>::new();

		let (asks, bids) = match_orders(asks, bids, Order{ side: Side::Bid, amount: 10, price: 10, timestamp: 1 });

		assert_eq!(asks.into_sorted_vec(), []);
		assert_eq!(bids.into_sorted_vec(), []);
	}

	#[test]
	fn test_cross_order_ask() {
		let asks = BinaryHeap::<Order>::new();
		let bids = BinaryHeap::from(vec![Order{ side: Side::Bid, amount: 10, price: 10, timestamp: 1 }]);

		let (asks, bids) = match_orders(asks, bids, Order{ side: Side::Ask, amount: 10, price: 10, timestamp: 1 });

		assert_eq!(asks.into_sorted_vec(), []);
		assert_eq!(bids.into_sorted_vec(), []);
	}

	#[test]
	fn test_cheaper_ask_comes_in() {
		let asks = BinaryHeap::<Order>::new();
		let bids = BinaryHeap::from(vec![Order{ side: Side::Bid, amount: 10, price: 10, timestamp: 1 }]);

		let (asks, bids) = match_orders(asks, bids, Order{ side: Side::Ask, amount: 10, price: 5, timestamp: 1 });

		assert_eq!(asks.into_sorted_vec(), []);
		assert_eq!(bids.into_sorted_vec(), []);
	}

}
