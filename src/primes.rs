
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

use std::cmp::{min, max};


#[pyclass]
pub struct PrimeSieve {
    primes: Vec<usize>,
    index: usize
}


// O(n) memory required
fn sieve0(n: usize) -> Vec<usize> {
    let n = max(n, 4);
    let mut state = vec![true; n];
    state[0] = false; // not prime
    state[1] = false; // not prime
    state[2] = true; // not prime
    state[3] = true; // not prime

    let m = (n as f64).sqrt() as usize;
    for i in 2..=m {
        if state[i] {
            for j in (i * i..n).step_by(i) {
                state[j] = false;
            }
        }
    }
    state.iter().enumerate().filter(|(_, &s)| s).map(|(i, _)| i).collect::<Vec<_>>()
}


// returns all primes up to and including sqrt(n), using a segmented approach to reduce memory requirements
fn sieve(n: usize) -> Vec<usize> {
    let n = max(n, 4);
    let m = (n as f64).sqrt() as usize;
    let chunk_size = min(n, 1_000);
    let mut primes = sieve0(chunk_size);
    for m0 in (chunk_size..m).step_by(chunk_size) {
        let m1 = min(m0 + chunk_size, m);
        let mut state = vec![true; m1 - m0];
        for p in &primes {
            let s = match m0 % p {
                0 => p * (m0 / p),
                _ => p * (m0 / p + 1)
            };
            for i in (s..m1).step_by(*p) {
                state[i - m0] = false;
            }
        }
        primes.extend(
            state.iter().enumerate().filter(|(_, &s)| s).map(|(i, _)| m0 + i) //.collect::<Vec<_>>()
        );
    }
    primes
}


// brute force
fn seed_primes(n: usize) -> Vec<usize> {
    let mut primes = vec![2, 3];
    let mut c = *primes.last().unwrap();
    loop {
        c += 2;
        if c * c > n {
            break;
        }
        if is_prime(c, &primes) {
            primes.push(c);
        }
    }
    primes
}


fn extend_seed_primes(primes: &Vec<usize>, n: usize) -> Vec<usize> {
    let mut ext_primes = vec![];
    let mut c = *primes.last().unwrap();
    loop {
        c += 2;
        if c * c > n {
            break;
        }
        if is_prime(c, primes) && is_prime(c, &ext_primes) {
            ext_primes.push(c);
        }
    }
    ext_primes
}


#[pymethods]
impl PrimeSieve {
    #[new]
    fn new(n: usize) -> Self {
        PrimeSieve{ primes: sieve(n), index: 0 }
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<usize> {
        slf.index += 1;
        match slf.index {
            i if i <= slf.primes.len() => Some(slf.primes[i - 1]),
            _ => None
        }
    }
}


fn is_prime(n: usize, primes_below: &Vec::<usize>) -> bool {
    match n {
        0..=1 => false,
        2..=3 => true,
        _ => {
            for p in primes_below {
                if n % p == 0 {
                    return false;
                }
                // only need to go as far as sqrt(n)
                if p * p > n {
                    break;
                }
            }
            true
        }
    }
}



#[pyclass]
pub struct PrimeGenerator {
    found: Vec<usize>
}


#[pymethods]
impl PrimeGenerator {
    #[new]
    fn new() -> Self {
        PrimeGenerator{ found: vec![2, 3] }
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<usize> {
        let mut n = *slf.found.last().unwrap();
        loop {
            n += 2;
            if is_prime(n, &slf.found) {
                slf.found.push(n);
                return Some(n);
            }
        }
    }
}


#[pyfunction]
pub fn nth_prime(n: usize) -> PyResult<usize> {
    if n == 0 {
        return Err(PyValueError::new_err("n must be >0"));
    }
    let mut found = Vec::with_capacity(n);
    found.push(2);
    found.push(3);
    while n > found.len() {
        let mut c = *found.last().unwrap();
        loop {
            c += 2;
            if is_prime(c, &found) {
                found.push(c);
                break;
            }
        }
    }
    Ok(found[n-1])
}


#[pyclass]
pub struct PrimeRange {
    index: usize,
    n: usize,
    seed_primes: Vec<usize>,
}


#[pymethods]
impl PrimeRange {
    #[new]
    fn new(m: usize, n: usize) -> Self {
        // seed_primes is faster than sieve for larger n
        PrimeRange{ index: if m % 2 == 0 { m + 1 } else { m }, n, seed_primes: seed_primes(n)}
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<usize> {
        while slf.index <= slf.n && !is_prime(slf.index, &slf.seed_primes) {
            slf.index += 2;
        }
        let ret = slf.index;
        slf.index += 2;
        match ret {
            r if r > slf.n => None,
            _ => Some(ret)
        }
    }
}


#[pyfunction(name="is_prime")]
pub fn is_prime_py(n: usize) -> bool {
    // is_prime(n, &sieve(max(n, 10))) is a lot slower!

    let mut m = 1000000;
    if n < m {
        return is_prime(n, &seed_primes(n))
    }
    let mut primes = seed_primes(m);
    loop {
        for p in &primes {
            if n % p == 0 {
                return false;
            }
        }
        if m > n {
            break;
        }
        m *= 10;
        primes = extend_seed_primes(&primes, m);
    }
    true
}


#[pyfunction]
pub fn prime_factors(n: usize) -> PyResult<Vec<usize>> {
    if n == 0 {
        return Err(PyValueError::new_err("input must be >=1"));
    }
    let mut factors = vec![];
    let mut m = n;
    for p in seed_primes(n) {
        while m % p == 0 {
            m /= p;
            factors.push(p);
        }
    }
    if m > 1 {
        factors.push(m);
    }
    Ok(factors)
}


