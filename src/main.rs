extern crate num_cpus;
extern crate num_format;
extern crate rand;

use std::fmt::{Display, Formatter, Error};
use rand::thread_rng;
use rand::seq::SliceRandom;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::{thread, env};
use std::time::Duration;
use num_format::{Locale, ToFormattedString};
use std::collections::HashMap;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    let iterations: u64 = if args.len() == 1 {
        1_000_000_000
    } else if args.len() == 2 {
        match args[1].parse::<u64>() {
            Ok(n) => n,
            Err(_) => {
                eprintln!("Argument must be an integer");
                exit(1);
            }
        }
    } else {
        eprintln!("Usage: idiots-delight [simulations]");
        exit(1);
    };

    println!("Running {} simulations", iterations.to_formatted_string(&Locale::en));
    println!();

    let counter = Arc::new(AtomicU64::new(0));
    let done = Arc::new(AtomicBool::new(false));
    let win_map = Arc::new(Mutex::new(HashMap::<i8, u64>::new()));

    let mut locks = Vec::<Arc<Mutex<usize>>>::new();

    for _ in 0..num_cpus::get() {
        let counter_bg = counter.clone();
        let win_map_bg = win_map.clone();

        let lock = Arc::new(Mutex::new(0));
        locks.push(lock.clone());

        thread::spawn(move || {
            let _guard = lock.lock().unwrap();

            let mut deck = gen_deck();
            let mut hand: [*const Card; 52] = [&deck[0] as *const Card; 52];

            let mut thread_win_map = HashMap::<i8, u64>::new();

            let mut count: u64;
            count = counter_bg.fetch_add(1, Ordering::SeqCst) + 1;

            while count < iterations {
                deck.shuffle(&mut thread_rng());

                if count % 1_000_000 == 0 {
                    println!("Attempt {}...", count.to_formatted_string(&Locale::en));
                }

                let hand_size = play_game(&deck, &mut hand);
                if let Some(num) = thread_win_map.get_mut(&hand_size) {
                    *num += 1;
                } else {
                    thread_win_map.insert(hand_size, 1);
                }

                count = counter_bg.fetch_add(1, Ordering::SeqCst) + 1;
            }

            let mut win_lock = win_map_bg.lock().unwrap();
            for (k, v) in thread_win_map {
                if let Some(num) = win_lock.get_mut(&k) {
                    *num += v;
                } else {
                    win_lock.insert(k, v);
                }
            }
        });
    }

    {
        let counter_bg = counter.clone();
        let done_bg = done.clone();

        thread::spawn(move || {
            let mut count = 0;

            let mut counts: [u64; 5] = Default::default();

            while !done_bg.load(Ordering::SeqCst) {
                let start = counter_bg.load(Ordering::SeqCst);
                thread::sleep(Duration::from_secs(1));
                let finish = counter_bg.load(Ordering::SeqCst);
                counts[count] = finish - start;
                if count == counts.len() - 1 {
                    count = 0;
                    let mut avg: f64 = 0.0;
                    for val in counts.iter() {
                        avg += *val as f64;
                    }
                    avg /= 10.0;
                    println!("Games / second: {}", avg);
                } else {
                    count += 1;
                }
            }
        });
    }

    // Wait for all threads to finish
    thread::sleep(Duration::from_millis(10));
    for lock in locks {
        let _ = lock.lock().unwrap();
    }
    done.store(true, Ordering::SeqCst);

    println!();
    println!();
    println!();

    let win_lock = win_map.lock().unwrap();
    let mut values: Vec<(&i8, &u64)> = win_lock.iter().collect();
    values.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
    for (k, v) in values {
        println!("Cards Left: {: >2} | Count: {}", k + 1, v);
    }
}

fn play_game<'a>(deck: &'a [Card; 52], hand: &'a mut [*const Card; 52]) -> i8 {
    let mut deck_index: usize = 0;
    let mut hand_index: i8 = -1;

    while deck_index < 52 {
        if hand_index < 3 {
            hand_index += 1;
            if hand_index >= 52 {
                break;
            }
            hand[hand_index as usize] = &deck[deck_index] as *const Card;
            deck_index += 1;
            if hand_index < 3 {
                continue;
            }
        }
        if unsafe { (*hand[hand_index as usize]).value == (*hand[(hand_index - 3) as usize]).value } {
            hand_index -= 4;
            continue;
        }
        if unsafe { (*hand[hand_index as usize]).suit == (*hand[(hand_index - 3) as usize]).suit } {
            hand[(hand_index - 2) as usize] = hand[hand_index as usize];
            hand_index -= 2;
            continue;
        }

        if deck_index == 52 {
            break;
        }

        hand_index += 1;
        if hand_index >= 52 {
            break;
        }
        hand[hand_index as usize] = &deck[deck_index] as *const Card;
        deck_index += 1;
    }

    return hand_index;
}

fn gen_deck() -> [Card; 52] {
    return [
        // Spades
        Card { value: CardValue::Ace, suit: CardSuit::Spades },
        Card { value: CardValue::Two, suit: CardSuit::Spades },
        Card { value: CardValue::Three, suit: CardSuit::Spades },
        Card { value: CardValue::Four, suit: CardSuit::Spades },
        Card { value: CardValue::Five, suit: CardSuit::Spades },
        Card { value: CardValue::Six, suit: CardSuit::Spades },
        Card { value: CardValue::Seven, suit: CardSuit::Spades },
        Card { value: CardValue::Eight, suit: CardSuit::Spades },
        Card { value: CardValue::Nine, suit: CardSuit::Spades },
        Card { value: CardValue::Ten, suit: CardSuit::Spades },
        Card { value: CardValue::Jack, suit: CardSuit::Spades },
        Card { value: CardValue::Queen, suit: CardSuit::Spades },
        Card { value: CardValue::King, suit: CardSuit::Spades },
        // Clubs
        Card { value: CardValue::Ace, suit: CardSuit::Clubs },
        Card { value: CardValue::Two, suit: CardSuit::Clubs },
        Card { value: CardValue::Three, suit: CardSuit::Clubs },
        Card { value: CardValue::Four, suit: CardSuit::Clubs },
        Card { value: CardValue::Five, suit: CardSuit::Clubs },
        Card { value: CardValue::Six, suit: CardSuit::Clubs },
        Card { value: CardValue::Seven, suit: CardSuit::Clubs },
        Card { value: CardValue::Eight, suit: CardSuit::Clubs },
        Card { value: CardValue::Nine, suit: CardSuit::Clubs },
        Card { value: CardValue::Ten, suit: CardSuit::Clubs },
        Card { value: CardValue::Jack, suit: CardSuit::Clubs },
        Card { value: CardValue::Queen, suit: CardSuit::Clubs },
        Card { value: CardValue::King, suit: CardSuit::Clubs },
        // Hearts
        Card { value: CardValue::Ace, suit: CardSuit::Hearts },
        Card { value: CardValue::Two, suit: CardSuit::Hearts },
        Card { value: CardValue::Three, suit: CardSuit::Hearts },
        Card { value: CardValue::Four, suit: CardSuit::Hearts },
        Card { value: CardValue::Five, suit: CardSuit::Hearts },
        Card { value: CardValue::Six, suit: CardSuit::Hearts },
        Card { value: CardValue::Seven, suit: CardSuit::Hearts },
        Card { value: CardValue::Eight, suit: CardSuit::Hearts },
        Card { value: CardValue::Nine, suit: CardSuit::Hearts },
        Card { value: CardValue::Ten, suit: CardSuit::Hearts },
        Card { value: CardValue::Jack, suit: CardSuit::Hearts },
        Card { value: CardValue::Queen, suit: CardSuit::Hearts },
        Card { value: CardValue::King, suit: CardSuit::Hearts },
        // Diamonds
        Card { value: CardValue::Ace, suit: CardSuit::Diamonds },
        Card { value: CardValue::Two, suit: CardSuit::Diamonds },
        Card { value: CardValue::Three, suit: CardSuit::Diamonds },
        Card { value: CardValue::Four, suit: CardSuit::Diamonds },
        Card { value: CardValue::Five, suit: CardSuit::Diamonds },
        Card { value: CardValue::Six, suit: CardSuit::Diamonds },
        Card { value: CardValue::Seven, suit: CardSuit::Diamonds },
        Card { value: CardValue::Eight, suit: CardSuit::Diamonds },
        Card { value: CardValue::Nine, suit: CardSuit::Diamonds },
        Card { value: CardValue::Ten, suit: CardSuit::Diamonds },
        Card { value: CardValue::Jack, suit: CardSuit::Diamonds },
        Card { value: CardValue::Queen, suit: CardSuit::Diamonds },
        Card { value: CardValue::King, suit: CardSuit::Diamonds },
    ];
}

#[derive(Debug, PartialEq)]
enum CardValue {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

#[derive(Debug, PartialEq)]
enum CardSuit {
    Spades,
    Clubs,
    Hearts,
    Diamonds,
}

struct Card {
    value: CardValue,
    suit: CardSuit,
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        return write!(f, "{:?} of {:?}", self.value, self.suit);
    }
}
