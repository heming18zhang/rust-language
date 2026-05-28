# Rust W3 Exercises (`rust-w3`)

Welcome to the `rust-w3` subdirectory! This repository contains a collection of fundamental algorithmic challenges, competitive programming problems, and foundational data structure practices written in idiomatic Rust. 

The focus of this module is on mastering core Rust concepts such as standard I/O streams, standard library collections, ownership/borrowing, and algorithm efficiency.

## 🚀 Projects & Exercises Covered

This directory includes solutions and implementations for the following technical exercises:

1. **Social Media Rights Tool:** A simple check tracking the legality of native vs. manual content reposting structures.
2. **Array Cyclic Rotation:** Efficient $O(n)$ time and $O(1)$ space algorithms to shift vectors both left and right using modulo arithmetic safely without memory overflows.
3. **Stream Statistics (Second Largest Number):** Memory-optimized stream-parsing logic to track primary and unique secondary maximums without loading complete files into memory. Includes multi-approach implementations using sorting and `BTreeSet` filtering.
4. **Two-Way Merge Sort:** Implementation of a classic data structures dual-pointer merging pipeline to seamlessly blend separate descending/ascending lists into a single sorted vector.
5. **Radix Converter (Custom Number Bases):** Mathematical base conversion engine mapping inputs up to Base-36 utilizing safe standard-library operations (`char::to_digit` and `char::from_digit`).
6. **Optimized Bubble Sort:** Standard sorting setup augmented with short-circuiting logic ("early stop") to hit optimal linear $O(n)$ time when digesting pre-sorted sequences.
7. **Word Counter & Frequency Tracker:** Token stream processor mapping string slices to occurrences utilizing the efficient `HashMap::entry` API.
8. **Sorted Map Utilities:** Implementations highlighting performance trade-offs between sorting transient vectors vs. deploying self-balancing `BTreeMap` structures.

## 🛠️ Key Rust Concepts Explored

* **Standard Input Streams:** Harnessing `io::stdin().lock().lines()` combined with `.by_ref()` to safely stream multi-line test inputs without destroying data iterators prematurely.
* **Safe Memory Slicing:** Avoiding common `index out of bounds` errors by calibrating loops around zero-indexed boundaries (`0..len`).
* **The Entry API & De-referencing:** Deploying `*map.entry(key).or_insert(0) += 1` to bypass immutable constraints when mutating values in place.
* **Collections Optimization:** Practical analytical comparisons charting search/insert trade-offs across `Vec`, `HashMap`, `BTreeMap`, and `BTreeSet`.

## 💻 Getting Started

### Prerequisites
Make sure you have Rust and Cargo installed on your system. If not, get it via [rustup](https://rustup.rs/):
```bash
curl --proto '=https' --tlsv1.2 -sSf [https://sh.rustup.rs](https://sh.rustup.rs) | sh
